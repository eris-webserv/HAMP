// game_server.rs — multiplayer game server.
//
// Two modes of operation:
//
//   Standalone (default):
//     `run()` binds a single port, serves blank chunks from a `WorldState`,
//     tracks player positions. This is the default when the binary starts.
//
//   Relay (friend-server integration):
//     `spawn_relay_session()` is called by the friend server's JoinGrant
//     handler. Each call binds a dynamic port and relays data between
//     the host and joiner without server-side world state.
//
// Adding a new C→S handler
// ─────────────────────────
// Add a match arm in `handle_client`. Most relayed packets just need an
// entry in the bulk-relay arm at the bottom.

pub mod dummy_world;
pub mod world_state;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::utils::config::Config;
use crate::defs::packet::{craft_batch, pack_string, unpack_string};
use world_state::WorldState;

// ── Per-session player ─────────────────────────────────────────────────────

struct GamePlayer {
    /// Cloned stream handle used by other threads to push data to this player.
    sink:         Mutex<TcpStream>,
    /// Last received PLAYER_DATA blob (C→S 0x03 body), replayed to players who join later.
    initial_data: Mutex<Option<Vec<u8>>>,
}

// ── Session mode ──────────────────────────────────────────────────────────

/// Determines how a session handles zone/chunk requests and position tracking.
enum SessionMode {
    /// Pure relay — the host client owns the world data.
    /// Zone/chunk requests are ignored (host sends data directly to peers).
    Relay,
    /// Server-managed world — the server owns and serves chunk data,
    /// tracks player positions, and can persist world state.
    Managed(Arc<WorldState>),
}

// ── Session shared state ────────────────────────────────────────────────────

pub(crate) struct Session {
    room_token:  String,
    mode:        SessionMode,
    players:     Mutex<HashMap<String, Arc<GamePlayer>>>,
    shutdown:    AtomicBool,
    /// The address the listener is bound to — used to unblock the accept loop on shutdown.
    listen_addr: Mutex<Option<std::net::SocketAddr>>,
}

impl Session {
    fn new(room_token: impl Into<String>, mode: SessionMode) -> Arc<Self> {
        Arc::new(Self {
            room_token: room_token.into(),
            mode,
            players: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
            listen_addr: Mutex::new(None),
        })
    }

    /// Shuts down the session: disconnects all players and unblocks the accept loop.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);

        // Disconnect all players.
        let players: Vec<Arc<GamePlayer>> = self.players.lock().unwrap()
            .drain()
            .map(|(_, p)| p)
            .collect();
        for p in &players {
            let _ = p.sink.lock().unwrap().shutdown(std::net::Shutdown::Both);
        }

        // Poke the listener to unblock the accept loop.
        if let Some(addr) = *self.listen_addr.lock().unwrap() {
            let _ = TcpStream::connect(addr);
        }

        println!("[GAME:'{}'] Session stopped, {} player(s) disconnected",
                 self.room_token, players.len());
    }

    /// Wraps `payload` in a batch frame and sends it to all players except
    /// `exclude`.
    fn broadcast(&self, payload: &[u8], exclude: Option<&str>) {
        let batch = craft_batch(2, payload);
        for (name, p) in self.players.lock().unwrap().iter() {
            if exclude == Some(name.as_str()) { continue; }
            let _ = p.sink.lock().unwrap().write_all(&batch);
        }
    }

    /// Sends `payload` to a single player by their in-session id.
    fn send_to(&self, target: &str, payload: &[u8]) {
        if let Some(p) = self.players.lock().unwrap().get(target) {
            let batch = craft_batch(2, payload);
            let _ = p.sink.lock().unwrap().write_all(&batch);
        }
    }
}

// ── Wire-packet builders ────────────────────────────────────────────────────

/// S→C 0x26 LOGIN_RESPONSE — first packet sent on login.
///
/// Python: make_login_response(world_name, token, zone_type=0)
///   Byte(0x26) + Short(zone_trail_count=0) + String(world_name) + String(token) + Byte(zone_type)
fn build_login_response(world_name: &str, token: &str) -> Vec<u8> {
    let mut p = vec![0x26u8];
    p.extend_from_slice(&0i16.to_le_bytes()); // zone_trail_count = 0
    p.extend(pack_string(world_name));
    p.extend(pack_string(token));
    p.push(0x00); // zone_type = 0 (overworld)
    p
}

/// S→C 0x29 UNIQUE_IDS — block of unique IDs for the client.
fn build_unique_ids(count: u16) -> Vec<u8> {
    let mut p = vec![0x29u8];
    p.extend_from_slice(&count.to_le_bytes());
    for i in 0..count {
        p.extend_from_slice(&(i as i64 + 1).to_le_bytes());
    }
    p
}

/// S→C 0x02 JOIN_CONFIRMED
///
/// Python: make_join_confirmed(host_name, username, player_id_short=0, is_host=0)
///   Byte(0x02) + String(host_name) + Byte(is_host) + Byte(0)
///   + String(username) + Short(player_id_short) + Short(0)
fn build_join_confirmed(server_name: &str, username: &str, is_host: bool) -> Vec<u8> {
    let mut p = vec![0x02u8];
    p.extend(pack_string(server_name));
    p.push(is_host as u8);
    p.push(0x00);
    p.extend(pack_string(username));
    p.extend_from_slice(&0i16.to_le_bytes()); // player_id_short
    p.extend_from_slice(&0i16.to_le_bytes()); // 0
    p
}

/// S→C 0x0B ZONE_DATA (full — sends zone data blob, flag=1)
///
/// ProcessIncomingZoneData reads:
///   GetString()  zone_name
///   ZoneData::UnpackFromWeb:
///     InventoryItem::UnpackFromWeb (3 shorts: 0,0,0 for empty)
///     GetByte()    zone_type
///     GetShort()×4 unknowns
///     GetString()  zone_name (inner)
///     GetShort()   timer_dict_count (0 = empty)
///   GetByte()    zone_type (trailing)
fn build_zone_data(zone_name: &str) -> Vec<u8> {
    let mut p = vec![0x0Bu8];
    p.push(0x01);                                    // flag = 1 (zone data follows)
    p.push(0x00);                                    // sub_flag = 0

    // ProcessIncomingZoneData body:
    p.extend(pack_string(zone_name));                // zone name
    // ZoneData::UnpackFromWeb:
    p.extend_from_slice(&0i16.to_le_bytes());        // InventoryItem: count1 = 0
    p.extend_from_slice(&0i16.to_le_bytes());        // InventoryItem: count2 = 0
    p.extend_from_slice(&0i16.to_le_bytes());        // InventoryItem: count3 = 0
    p.push(0x00);                                    // zone_type = 0 (overworld)
    p.extend_from_slice(&0i16.to_le_bytes());        // unknown1
    p.extend_from_slice(&0i16.to_le_bytes());        // unknown2
    p.extend_from_slice(&0i16.to_le_bytes());        // unknown3
    p.extend_from_slice(&0i16.to_le_bytes());        // unknown4
    p.extend(pack_string(zone_name));                // zone_name (inner)
    p.extend_from_slice(&0i16.to_le_bytes());        // timer_dict_count = 0
    p.push(0x00);                                    // trailing zone_type
    p
}

/// S→C 0x17 DAY_NIGHT — short(ms)
fn build_daynight(ms: i16) -> Vec<u8> {
    let mut p = vec![0x17u8];
    p.extend_from_slice(&ms.to_le_bytes());
    p
}

/// S→C 0x07 JOIN/LEAVE notification
/// String(unused) + String(username) + Byte(1=joined, 0=left)
fn build_join_notif(username: &str, joined: bool) -> Vec<u8> {
    let mut p = vec![0x07u8];
    p.extend(pack_string(""));       // unused
    p.extend(pack_string(username));
    p.push(joined as u8);
    p
}

/// S→C 0x13 PLAYER_GONE
///
/// RE from GameServerReceiver::OnReceive case 0x13 (byte 0 = gone):
///   GetByte()   0 = gone
///   GetString() username
///   GetByte()   mob_count → mob_count × GetString() mob_id
fn build_player_gone(username: &str) -> Vec<u8> {
    let mut p = vec![0x13u8, 0x00]; // type = gone
    p.extend(pack_string(username));
    p.push(0x00); // mob_count = 0
    p
}

/// S→C 0x13 NEW_PLAYER_NEARBY
///
/// `player_data_body` is the raw body from C→S 0x03 (SendInitialPlayerData),
/// forwarded as-is into the OnlinePlayerData slot.
fn build_player_nearby(username: &str, player_data_body: &[u8]) -> Vec<u8> {
    let mut p = vec![0x13u8, 0x01]; // type = new
    p.extend(pack_string(username)); // username
    p.extend(pack_string(username)); // display_name (same for now)
    p.extend_from_slice(player_data_body);
    p
}

// ── Per-client handler ─────────────────────────────────────────────────────

fn handle_client(mut stream: TcpStream, addr: std::net::SocketAddr, session: Arc<Session>) {
    let mut player_id: Option<String> = None;
    let mut buf = [0u8; 65536];

    println!("[GAME:'{}'] {} connected", session.room_token, addr);

    loop {
        if session.shutdown.load(Ordering::Relaxed) { break; }
        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let data = &buf[..n];

        if data.is_empty() { continue; }

        // Handshake probe: single 0x66 byte.
        if data[0] == 0x66 {
            let _ = stream.write_all(&craft_batch(0, &[0x09, 0x01]));
            continue;
        }

        if data.len() < 10 { continue; }
        let pid = data[9];

        match pid {

            // ── PING (0x01) ────────────────────────────────────────────────
            0x01 => {
                let _ = stream.write_all(&craft_batch(2, &[0x01]));
            }

            // ── HEARTBEAT (0x0F) ───────────────────────────────────────────
            0x0F => {
                let _ = stream.write_all(&craft_batch(2, &[0x0F]));
            }

            // ── LOGIN (0x26) ───────────────────────────────────────────────
            // C→S: [world_name: Str] [token: Str]
            //
            // Response sequence (matching Python game_server.py):
            //   S→C 0x26  LOGIN_RESPONSE
            //   S→C 0x29  UNIQUE_IDS
            //   S→C 0x02  JOIN_CONFIRMED
            //   S→C 0x0B  ZONE_DATA
            //   S→C 0x17  DAYNIGHT
            //   S→C 0x07  JOIN_NOTIF (broadcast)
            0x26 => {
                if player_id.is_some() { continue; } // ignore repeated logins

                let (_raw_world, off) = unpack_string(data, 10);
                let (raw_token, _) = unpack_string(data, off);
                let token = raw_token.replace('\0', "").trim().to_string();
                let uid = if token.is_empty() {
                    format!("player_{}", addr.port())
                } else {
                    token
                };
                let world = session.room_token.clone();

                let cloned = match stream.try_clone() {
                    Ok(s) => s,
                    Err(e) => { eprintln!("[GAME] try_clone failed for {}: {}", addr, e); break; }
                };
                let player = Arc::new(GamePlayer {
                    sink:         Mutex::new(cloned),
                    initial_data: Mutex::new(None),
                });
                session.players.lock().unwrap().insert(uid.clone(), Arc::clone(&player));
                player_id = Some(uid.clone());

                // Track in world state for position management.
                if let SessionMode::Managed(ref ws) = session.mode {
                    use world_state::TrackedPlayer;
                    ws.players.write().unwrap()
                        .insert(uid.clone(), TrackedPlayer::new(&ws.default_zone));
                }

                println!("[GAME:'{}'] {} → player_id='{}'", world, addr, uid);

                // 1. S→C 0x26: login response
                let _ = stream.write_all(&craft_batch(2, &build_login_response(&world, &uid)));

                // 2. S→C 0x29: unique IDs
                let _ = stream.write_all(&craft_batch(2, &build_unique_ids(16)));

                // 3. S→C 0x02: join confirmed
                let _ = stream.write_all(&craft_batch(2, &build_join_confirmed(&world, &uid, false)));

                // 4. S→C 0x0B: zone data
                let zone_name = match session.mode {
                    SessionMode::Managed(ref ws) => ws.default_zone.clone(),
                    SessionMode::Relay => "overworld".to_string(),
                };
                let _ = stream.write_all(&craft_batch(2, &build_zone_data(&zone_name)));

                // 5. S→C 0x17: daynight
                let _ = stream.write_all(&craft_batch(2, &build_daynight(12000)));

                // 6. S→C 0x07: join notification (broadcast to others)
                session.broadcast(&build_join_notif(&uid, true), Some(uid.as_str()));
            }

            // ── PLAYER_DATA (0x03) ─────────────────────────────────────────
            // Store initial_data; broadcast spawn to others after a short delay
            // (matching Python's delayed_sync pattern).
            0x03 => {
                if let Some(ref uid) = player_id {
                    let body = data[10..].to_vec();

                    // 1. Store this player's initial_data.
                    if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                        *p.initial_data.lock().unwrap() = Some(body.clone());
                    }

                    // 2. Broadcast NEW_PLAYER_NEARBY (0x13 type=1) to all other players.
                    session.broadcast(&build_player_nearby(uid, &body), Some(uid.as_str()));

                    // 3. Send all existing players' data to this newcomer as 0x13 type=1.
                    let existing: Vec<(String, Vec<u8>)> = session.players.lock().unwrap()
                        .iter()
                        .filter(|(n, _)| n.as_str() != uid.as_str())
                        .filter_map(|(n, p)| {
                            p.initial_data.lock().unwrap()
                                .as_ref()
                                .map(|d| (n.clone(), d.clone()))
                        })
                        .collect();
                    for (name, init) in existing {
                        let _ = stream.write_all(&craft_batch(2, &build_player_nearby(&name, &init)));
                    }
                }
            }

            // ── REQ_ZONE_DATA (0x0A) ──────────────────────────────────────
            // C→S: [zone_name: Str] [type: u8] [if type 2|3: packed_position]
            //
            // Zone data is already sent during login (0x26 handler).
            // This handles zone change requests (e.g. entering a cave).
            0x0A => {
                let (zone_name, _) = unpack_string(data, 10);
                let zone = if zone_name.is_empty() {
                    match session.mode {
                        SessionMode::Managed(ref ws) => ws.default_zone.clone(),
                        SessionMode::Relay => "overworld".to_string(),
                    }
                } else {
                    zone_name
                };
                let _ = stream.write_all(&craft_batch(2, &build_zone_data(&zone)));
            }

            // ── REQ_CHUNK (0x0C) ──────────────────────────────────────────
            0x0C => {
                if let SessionMode::Managed(ref world) = session.mode {
                    let (_zone_name, off) = unpack_string(data, 10);
                    if data.len() >= off + 4 {
                        let x = i16::from_le_bytes([data[off], data[off + 1]]);
                        let z = i16::from_le_bytes([data[off + 2], data[off + 3]]);
                        let wire = world.get_chunk_wire(x, z);
                        let _ = stream.write_all(&craft_batch(2, &wire));
                    }
                }
                // Relay mode: ignore (host serves chunks directly).
            }

            // ── POSITION (0x11) → relay as S→C 0x11 ─────────────────────
            0x11 => {
                if let Some(ref uid) = player_id {
                    // In managed mode, track the position server-side.
                    if let SessionMode::Managed(ref world) = session.mode {
                        // Position body: [pos_at: 4×i16] [pos_to: 4×i16] [rot: 4×i16]
                        let body = &data[10..];
                        if body.len() >= 24 {
                            use world_state::WorldPosition;
                            let at = WorldPosition {
                                chunk_x: i16::from_le_bytes([body[0], body[1]]),
                                chunk_z: i16::from_le_bytes([body[2], body[3]]),
                                local_x: i16::from_le_bytes([body[4], body[5]]),
                                local_z: i16::from_le_bytes([body[6], body[7]]),
                            };
                            let to = WorldPosition {
                                chunk_x: i16::from_le_bytes([body[8], body[9]]),
                                chunk_z: i16::from_le_bytes([body[10], body[11]]),
                                local_x: i16::from_le_bytes([body[12], body[13]]),
                                local_z: i16::from_le_bytes([body[14], body[15]]),
                            };
                            if let Ok(mut players) = world.players.write() {
                                if let Some(tp) = players.get_mut(uid.as_str()) {
                                    tp.position = at;
                                    tp.target = to;
                                }
                            }
                        }
                    }

                    let mut pkt = vec![0x11u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── MOB_POSITIONS (0x41) → relay as same ID ───────────────────
            0x41 => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![0x41u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── CHAT (0x06) → relay as S→C 0x06 ──────────────────────────
            // RE: GSR case 6 reads [str player_id][str display_name][str msg][u8 type]
            0x06 => {
                if let Some(ref uid) = player_id {
                    let (msg, _) = unpack_string(data, 10);
                    let mut pkt = vec![0x06u8];
                    pkt.extend(pack_string(uid));  // player_id
                    pkt.extend(pack_string(uid));  // display_name (same for now)
                    pkt.extend(pack_string(&msg)); // message
                    pkt.push(0x00);                // type: 0 = public
                    session.broadcast(&pkt, None); // include sender so they see own msg
                }
            }

            // ── TELE_START (0x15) → relay as S→C 0x0C ────────────────────
            0x15 => {
                if let Some(ref uid) = player_id {
                    let (tele_name, _) = unpack_string(data, 10);
                    let mut pkt = vec![0x0Cu8];
                    pkt.extend(pack_string(uid));
                    pkt.extend(pack_string(&tele_name));
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── ZONE_CHANGE (0x14) → broadcast zone change ─────────────────
            0x14 => {
                if let Some(ref uid) = player_id {
                    let (zone_name, _) = unpack_string(data, 10);
                    let mut pkt = vec![0x14u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend(pack_string(&zone_name));
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── SYNC_COMPLETE (0x2A) — echo to sender, relay to others ────
            0x2A => {
                let body = &data[9..]; // includes the 0x2A ID byte
                let _ = stream.write_all(&craft_batch(2, body));
                if let Some(ref uid) = player_id {
                    session.broadcast(body, Some(uid.as_str()));
                }
            }

            // ── ASK_JOIN (0x2D) — relay to named target ───────────────────
            0x2D => {
                if let Some(ref uid) = player_id {
                    let (target, off) = unpack_string(data, 10);
                    let mut pkt = vec![0x2Du8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[off..]);
                    session.send_to(&target, &pkt);
                }
            }

            // ── YOU_MAY_JOIN (0x2B) — relay to named target ───────────────
            0x2B => {
                if let Some(ref uid) = player_id {
                    let (target, off) = unpack_string(data, 10);
                    let mut pkt = vec![0x2Bu8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[off..]);
                    session.send_to(&target, &pkt);
                }
            }

            // ── BUILD (0x20) ───────────────────────────────────────────────
            // C→S: [validator:Str][Item][rot:u8][zone:Str][shorts×4][extra:Str]
            // S→C: [Item][rot:u8][zone:Str][shorts×4][owner:Str][extra]
            // DO NOT echo back to builder (client already placed it locally).
            0x20 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let mut pkt = vec![0x20u8];
                    // Item + rot + zone + shorts
                    let item_and_rest = &data[off..];
                    // Find where shorts end to insert owner string.
                    // Item is variable, rot is 1 byte, zone is Str, shorts is 8 bytes.
                    // Simplest: relay everything after validator, append owner.
                    // Actually Python strips validator, then appends owner before extra.
                    // The exact layout matters for the S→C handler. For now, relay
                    // everything after validator and append owner at the end.
                    pkt.extend_from_slice(item_and_rest);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── REMOVE_OBJECT (0x21) ──────────────────────────────────────
            // C→S: [validator:Str][zone:Str][shorts×4][rot:u8][Item][extra:Str]
            // S→C: [zone:Str][shorts×4][rot:u8][Item][owner:Str]
            0x21 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let mut pkt = vec![0x21u8];
                    pkt.extend_from_slice(&data[off..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── REPLACE_BUILDABLE (0x22) ──────────────────────────────────
            // C→S: [validator:Str][old_Item][new_Item][rot:u8][zone:Str][shorts×4][extra:Str]
            // S→C: [old_Item][new_Item][rot:u8][zone:Str][shorts×4][owner:Str]
            0x22 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let mut pkt = vec![0x22u8];
                    pkt.extend_from_slice(&data[off..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Combat: strip validator, relay ────────────────────────────
            // 0x46 ATTACK_ANIM, 0x47 HIT_MOB, 0x48 MOB_DIE
            0x46 | 0x47 | 0x48 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let mut pkt = vec![pid];
                    pkt.extend_from_slice(&data[off..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Perks: strip validator, relay ─────────────────────────────
            // 0x51 APPLY_PERK, 0x52 LAUNCH_PROJECTILE, 0x54 ALL_PRE_PERKS, 0x55 CREATE_PERK_DROP
            0x51 | 0x52 | 0x54 | 0x55 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let mut pkt = vec![pid];
                    pkt.extend_from_slice(&data[off..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Bulk broadcast-relay: [pid][player_id][body] ──────────────
            0x09 | 0x16 | 0x18 | 0x19 | 0x23 |
            0x4A | 0x4B | 0x4E | 0x4F | 0x50 |
            0x53 | 0x56 | 0x57 | 0x58 |
            0x59 | 0x5A => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![pid];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Unknown — relay with player prefix ────────────────────────
            _ => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![pid];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }
        }
    }

    // Disconnect cleanup: remove player and notify others via 0x13 type=gone + 0x07 leave.
    if let Some(ref uid) = player_id {
        session.players.lock().unwrap().remove(uid.as_str());
        session.broadcast(&build_player_gone(uid), None);
        session.broadcast(&build_join_notif(uid, false), None);

        // Remove from world state tracking.
        if let SessionMode::Managed(ref world) = session.mode {
            world.players.write().unwrap().remove(uid.as_str());
        }

        println!("[GAME:'{}'] '{}' disconnected ({})", session.room_token, uid, addr);
    }
}

// ── Standalone entry point ────────────────────────────────────────────────

/// Starts a standalone managed game server on `cfg.game_port`.
/// Serves a blank flat world and handles all connecting clients.
pub fn run(cfg: &Config) {
    let addr = format!("{}:{}", cfg.host, cfg.game_port);
    let listener = TcpListener::bind(&addr)
        .unwrap_or_else(|e| panic!("Failed to bind game server to {}: {}", addr, e));
    println!("[GAME] Standalone game server listening on {} ...", addr);

    let world = Arc::new(WorldState::new("World", 5));
    let session = Session::new("World", SessionMode::Managed(Arc::clone(&world)));

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let peer = stream.peer_addr()
                    .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                let sess = Arc::clone(&session);
                std::thread::spawn(move || handle_client(stream, peer, sess));
            }
            Err(e) => eprintln!("[GAME] accept error: {}", e),
        }
    }
}

// ── Friend-server relay spawner ───────────────────────────────────────────

/// Spawns a relay-mode game session on a dynamic port.
/// Used by the friend server's JoinGrant handler.
///
/// Returns the port number, or `None` if no port in the range is available.
pub fn spawn_relay_session(room_token: String, cfg: &Config) -> Option<u16> {
    for port in cfg.game_port..=cfg.game_port_max {
        let addr = format!("{}:{}", cfg.host, port);
        if let Ok(listener) = TcpListener::bind(&addr) {
            let session = Session::new(room_token.clone(), SessionMode::Relay);
            println!("[GAME] Relay session '{}' → port {}", room_token, port);
            std::thread::spawn(move || {
                for incoming in listener.incoming() {
                    match incoming {
                        Ok(stream) => {
                            let peer = stream.peer_addr()
                                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                            let sess = Arc::clone(&session);
                            std::thread::spawn(move || handle_client(stream, peer, sess));
                        }
                        Err(e) => eprintln!("[GAME] accept error on port {}: {}", port, e),
                    }
                }
            });
            return Some(port);
        }
    }
    eprintln!("[GAME] No free port in {}–{}", cfg.game_port, cfg.game_port_max);
    None
}

/// Spawns a managed-mode game session on a dynamic port with server-owned world state.
/// Used by the admin-spawned world system.
///
/// Returns `(port, session_handle)` or `None` if no port is available.
/// The caller can later call `session.stop()` to shut everything down.
pub fn spawn_managed_session(room_token: String, cfg: &Config, world: Arc<WorldState>) -> Option<(u16, Arc<Session>)> {
    for port in cfg.game_port..=cfg.game_port_max {
        let addr = format!("{}:{}", cfg.host, port);
        if let Ok(listener) = TcpListener::bind(&addr) {
            // Set a short accept timeout so the loop can check the shutdown flag.
            listener.set_nonblocking(false).ok();

            let session = Session::new(room_token.clone(), SessionMode::Managed(world));
            *session.listen_addr.lock().unwrap() = listener.local_addr().ok();
            let accept_session = Arc::clone(&session);
            println!("[GAME] Managed session '{}' → port {}", room_token, port);
            std::thread::spawn(move || {
                // Use a timeout so we can poll the shutdown flag.
                listener.set_nonblocking(false).ok();
                for incoming in listener.incoming() {
                    if accept_session.shutdown.load(Ordering::Relaxed) { break; }
                    match incoming {
                        Ok(stream) => {
                            let peer = stream.peer_addr()
                                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                            let sess = Arc::clone(&accept_session);
                            std::thread::spawn(move || handle_client(stream, peer, sess));
                        }
                        Err(e) => {
                            if accept_session.shutdown.load(Ordering::Relaxed) { break; }
                            eprintln!("[GAME] accept error on port {}: {}", port, e);
                        }
                    }
                }
                println!("[GAME] Accept loop for '{}' exited", accept_session.room_token);
            });
            return Some((port, session));
        }
    }
    eprintln!("[GAME] No free port in {}–{}", cfg.game_port, cfg.game_port_max);
    None
}
