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

// ── Inventory / basket wire helpers ───────────────────────────────────────

/// Skips past an `InventoryItem::UnpackFromWeb` blob and returns the new offset.
///
/// Wire layout:
///   Short(short_prop_count) + [String(key) + Short(value)] × count
///   Short(string_prop_count) + [String(key) + String(value)] × count
///   Short(long_prop_count)   + [String(key) + Long(value)]  × count
fn skip_inventory_item(data: &[u8], mut off: usize) -> usize {
    // Short properties
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o; // key
        off += 2; // short value
    }
    // String properties
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o; // key
        let (_, o) = unpack_string(data, off); off = o; // value
    }
    // Long properties
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o; // key
        off += 4; // long value (u32)
    }
    off
}

/// Skips past a `BasketContents` blob (slot_count × [index + quantity + InventoryItem]).
/// Returns the new offset past the last slot.
fn skip_basket_contents(data: &[u8], mut off: usize) -> usize {
    if off + 2 > data.len() { return off; }
    let slot_count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..slot_count {
        off += 2 + 2; // index + quantity
        off = skip_inventory_item(data, off);
    }
    off
}

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
    /// The host player's username (relay mode only). The host client owns
    /// the world data; chunk/container requests from guests are relayed to it.
    host:        Mutex<Option<String>>,
    /// Pending chunk requests: chunk_key → list of requesting player usernames.
    pending_chunks:     Mutex<HashMap<String, Vec<String>>>,
    /// Pending container requests: basket_id → list of requesting player usernames.
    pending_containers: Mutex<HashMap<String, Vec<String>>>,
}

impl Session {
    fn new(room_token: impl Into<String>, mode: SessionMode) -> Arc<Self> {
        Arc::new(Self {
            room_token: room_token.into(),
            mode,
            players: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
            listen_addr: Mutex::new(None),
            host: Mutex::new(None),
            pending_chunks: Mutex::new(HashMap::new()),
            pending_containers: Mutex::new(HashMap::new()),
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
    let mut read_buf = [0u8; 65536];
    let mut accum: Vec<u8> = Vec::new();

    println!("[GAME:'{}'] {} connected", session.room_token, addr);

    'outer: loop {
        if session.shutdown.load(Ordering::Relaxed) { break; }
        let n = match stream.read(&mut read_buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        accum.extend_from_slice(&read_buf[..n]);

        // Consume complete framed packets from the accumulation buffer.
        while !accum.is_empty() {
            // Handshake probe: single 0x66 byte (no length prefix).
            if accum[0] == 0x66 {
                let _ = stream.write_all(&craft_batch(0, &[0x09, 0x01]));
                accum.remove(0);
                continue;
            }

            if accum.len() < 2 { break; }
            let total_len = u16::from_le_bytes([accum[0], accum[1]]) as usize;
            if accum.len() < total_len { break; } // wait for more data

            let data: Vec<u8> = accum.drain(..total_len).collect();

            if data.len() < 10 { continue; }
            let pid = data[9];
            let data = data.as_slice();

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

                // In relay mode, the room_token is the host's username.
                // Only the player whose name matches becomes host.
                let is_host = match session.mode {
                    SessionMode::Relay => {
                        let mut host = session.host.lock().unwrap();
                        if host.is_none() && uid.eq_ignore_ascii_case(&world) {
                            *host = Some(uid.clone());
                            true
                        } else {
                            false
                        }
                    }
                    SessionMode::Managed(_) => false,
                };

                // In relay mode, guests must wait for the host to connect
                // before proceeding — the host's client serves all world data.
                // Wait up to 5 seconds, then disconnect if no host appears.
                if matches!(session.mode, SessionMode::Relay) && !is_host {
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                    loop {
                        if session.host.lock().unwrap().is_some() { break; }
                        if std::time::Instant::now() >= deadline {
                            println!("[GAME:'{}'] Guest '{}' timed out waiting for host",
                                     world, uid);
                            session.players.lock().unwrap().remove(&uid);
                            break 'outer;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }

                let is_host_flag = is_host;

                if is_host_flag {
                    println!("[GAME:'{}'] {} → HOST player_id='{}'", world, addr, uid);
                } else {
                    println!("[GAME:'{}'] {} → GUEST player_id='{}'", world, addr, uid);
                }

                // 1. S→C 0x26: login response
                let _ = stream.write_all(&craft_batch(2, &build_login_response(&world, &uid)));

                // 2. S→C 0x29: unique IDs
                let _ = stream.write_all(&craft_batch(2, &build_unique_ids(16)));

                // 3. S→C 0x02: join confirmed (is_host=true for host, false for guests)
                let _ = stream.write_all(&craft_batch(2, &build_join_confirmed(&world, &uid, is_host_flag)));

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
            // C→S body: pos(8) + zone(Str) + appearance(1) + rest
            // Must transform to S→C 0x13 body (OnlinePlayerData):
            //   pos(8) + target_pos(8) + rotation(8) + appearance(1) + zone(Str) + extra(Str) + rest
            0x03 => {
                if let Some(ref uid) = player_id {
                    let raw = &data[10..];
                    if raw.len() >= 8 {
                        let mut off = 0usize;
                        let pos_bytes = &raw[off..off + 8]; off += 8;
                        let (zone_name, new_off) = unpack_string(raw, off); off = new_off;
                        let appearance = if off < raw.len() { raw[off] } else { 0 }; off += 1;
                        let rest = if off < raw.len() { &raw[off..] } else { &[] };

                        // Build OnlinePlayerData blob
                        let mut opd = Vec::new();
                        opd.extend_from_slice(pos_bytes);                    // position
                        opd.extend_from_slice(pos_bytes);                    // target (same)
                        opd.extend_from_slice(&[0, 0, 0, 0, 0, 0, 100, 0]); // rotation: quat (0,0,0,100) as 4×i16 LE
                        opd.push(appearance);
                        opd.extend(pack_string(&zone_name));
                        opd.extend(pack_string(""));                         // extra string
                        opd.extend_from_slice(rest);

                        // 1. Store this player's initial_data (transformed).
                        if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                            *p.initial_data.lock().unwrap() = Some(opd.clone());
                        }

                        // 2. Broadcast NEW_PLAYER_NEARBY (0x13 type=1) to all other players.
                        session.broadcast(&build_player_nearby(uid, &opd), Some(uid.as_str()));

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
            }

            // ── REQ_ZONE_DATA (0x0A) ──────────────────────────────────────
            // C→S: [zone_name: Str] [type: u8] [if type 2|3: packed_position]
            //
            // Zone data is already sent during login (0x26 handler).
            // This handles subsequent zone change requests (e.g. entering a cave).
            0x0A => {
                if let Some(ref uid) = player_id {
                    let (zone_name, off) = unpack_string(data, 10);
                    let zone_type = if data.len() > off { data[off] } else { 0 };

                    match session.mode {
                        SessionMode::Managed(ref ws) => {
                            let zone = if zone_name.is_empty() {
                                ws.default_zone.clone()
                            } else {
                                zone_name
                            };
                            let _ = stream.write_all(&craft_batch(2, &build_zone_data(&zone)));
                        }
                        SessionMode::Relay => {
                            let host = session.host.lock().unwrap().clone();
                            let is_host = host.as_ref().map(|h| h == uid).unwrap_or(false);
                            if is_host {
                                // Host gets zone data directly.
                                let zone = if zone_name.is_empty() { "overworld".to_string() } else { zone_name };
                                let _ = stream.write_all(&craft_batch(2, &build_zone_data(&zone)));
                            } else if let Some(ref hname) = host {
                                // Guest → relay to host.
                                let mut relay = vec![0x0Au8];
                                relay.extend(pack_string(uid));
                                relay.extend(pack_string(&zone_name));
                                relay.push(zone_type);
                                if data.len() > off + 1 {
                                    relay.extend_from_slice(&data[off + 1..]);
                                }
                                session.send_to(hname, &relay);
                            }
                        }
                    }
                }
            }

            // ── REQ_CHUNK (0x0C) ──────────────────────────────────────────
            0x0C => {
                if let Some(ref uid) = player_id {
                    let (zone_name, off) = unpack_string(data, 10);
                    if data.len() >= off + 4 {
                        let x = i16::from_le_bytes([data[off], data[off + 1]]);
                        let z = i16::from_le_bytes([data[off + 2], data[off + 3]]);

                        match session.mode {
                            SessionMode::Managed(ref world) => {
                                let wire = world.get_chunk_wire(x, z);
                                let _ = stream.write_all(&craft_batch(2, &wire));
                            }
                            SessionMode::Relay => {
                                let host = session.host.lock().unwrap().clone();
                                let is_host = host.as_ref().map(|h| h == uid).unwrap_or(false);
                                if is_host {
                                    // Host reads chunks locally (ShouldSaveLocally=true).
                                    // This shouldn't normally happen.
                                } else if host.is_some() {
                                    // Guest → relay request to host.
                                    // S→C 0x0C to host: String(requester) + String(zone) + Short(x) + Short(z)
                                    let chunk_key = format!("{}_{}_{}",  zone_name, x, z);
                                    session.pending_chunks.lock().unwrap()
                                        .entry(chunk_key).or_default().push(uid.clone());

                                    let mut relay = vec![0x0Cu8];
                                    relay.extend(pack_string(uid));
                                    relay.extend(pack_string(&zone_name));
                                    relay.extend_from_slice(&x.to_le_bytes());
                                    relay.extend_from_slice(&z.to_le_bytes());
                                    session.send_to(host.as_ref().unwrap(), &relay);
                                }
                            }
                        }
                    }
                }
            }

            // ── HOST CHUNK RESPONSE (0x0D) — relay mode only ─────────────
            // The host's client sends: 0x0D + String(requester) + PackForWeb body + bandit data
            // We strip the requester, wrap the PackForWeb body in the outer envelope
            // the guest expects, and forward it.
            0x0D if matches!(session.mode, SessionMode::Relay) => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (requester, off) = unpack_string(data, 10);
                        let chunk_data = &data[off..];

                        if chunk_data.len() >= 4 {
                            // PackForWeb starts: Short(cx) + Short(cz) + String(zone) + ...
                            let cx = i16::from_le_bytes([chunk_data[0], chunk_data[1]]);
                            let cz = i16::from_le_bytes([chunk_data[2], chunk_data[3]]);
                            let (zone_from_host, _) = unpack_string(chunk_data, 4);

                            // Build outer wrapper the guest expects:
                            // 0x0D + String(zone) + Short(cx) + Short(cz) + Byte(0=new) + String("")
                            // + PackForWeb body + bandit data
                            let mut out = vec![0x0Du8];
                            out.extend(pack_string(&zone_from_host));
                            out.extend_from_slice(&cx.to_le_bytes());
                            out.extend_from_slice(&cz.to_le_bytes());
                            out.push(0x00); // flag = 0 (new chunk)
                            out.extend(pack_string("")); // checkpoint
                            out.extend_from_slice(chunk_data); // PackForWeb body + bandit data

                            // Route to whoever requested this chunk.
                            let chunk_key = format!("{}_{}_{}",  zone_from_host, cx, cz);
                            let targets = session.pending_chunks.lock().unwrap()
                                .remove(&chunk_key)
                                .unwrap_or_default();
                            if targets.is_empty() {
                                // Maybe requester name from the packet itself
                                if !requester.is_empty() {
                                    session.send_to(&requester, &out);
                                }
                            } else {
                                for target in &targets {
                                    session.send_to(target, &out);
                                }
                            }
                        }
                    }
                }
            }

            // ── HOST ZONE RESPONSE (0x0B) — relay mode only ──────────────
            // The host sends: 0x0B + String(requester) + String(zone) + Byte(type) + ...
            // We transform to guest format: 0x0B + Byte(1) + Byte(0) + String(zone) + rest
            0x0B if matches!(session.mode, SessionMode::Relay) => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (requester, off) = unpack_string(data, 10);
                        let (zone_name, zoff) = unpack_string(data, off);
                        if zoff < data.len() {
                            let type_flag = data[zoff];
                            // Skip optional position data if present
                            let pos_skip = if (type_flag & 0xFE) == 2 { 12 } else { 0 };
                            let zone_data_start = zoff + 1 + pos_skip;

                            let mut out = vec![0x0Bu8, 0x01, 0x00]; // flag=1, sub=0
                            out.extend(pack_string(&zone_name));
                            if zone_data_start < data.len() {
                                out.extend_from_slice(&data[zone_data_start..]);
                            }

                            if !requester.is_empty() {
                                session.send_to(&requester, &out);
                            }
                        }
                    }
                }
            }

            // ── REQ_CONTAINER (0x1A) ──────────────────────────────────────
            // C→S: [validator:Str][basket_id:u32][type:u8][chunk:Str][shorts×4]
            // Relay mode: forward to host as 0x1C + String(requester) + Long(basket_id)
            //
            // The host client also sends 0x1A when opening its own containers
            // (despite ShouldSaveLocally=true). We must relay 0x1C back to the
            // host so it loads from disk and responds with 0x1B.
            0x1A => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    if data.len() >= off + 4 {
                        let basket_id = u32::from_le_bytes([
                            data[off], data[off+1], data[off+2], data[off+3]
                        ]);

                        if matches!(session.mode, SessionMode::Relay) {
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                let bk = basket_id.to_string();
                                session.pending_containers.lock().unwrap()
                                    .entry(bk).or_default().push(uid.clone());

                                // Relay 0x1C to the host (works for both host-self
                                // and guest requests — host loads from disk either way).
                                let mut relay = vec![0x1Cu8];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&basket_id.to_le_bytes());
                                session.send_to(hname, &relay);
                            }
                        }
                        // Managed mode: containers not yet implemented
                    }
                }
            }

            // ── HOST CONTAINER RESPONSE (0x1B) — relay mode only ─────────
            // Host sends: 0x1B + String(requester) + Long(basket_id) + BasketContents
            // Guest expects: 0x1B + Long(basket_id) + BasketContents
            0x1B if matches!(session.mode, SessionMode::Relay) => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (_requester, off) = unpack_string(data, 10);
                        if data.len() >= off + 4 {
                            let basket_id = u32::from_le_bytes([
                                data[off], data[off+1], data[off+2], data[off+3]
                            ]);

                            let mut out = vec![0x1Bu8];
                            out.extend_from_slice(&data[off..]); // basket_id + contents

                            let bk = basket_id.to_string();
                            let targets = session.pending_containers.lock().unwrap()
                                .remove(&bk)
                                .unwrap_or_default();
                            for target in &targets {
                                session.send_to(target, &out);
                            }
                        }
                    }
                }
            }

            // ── CLOSE_BASKET (0x1E) — relay to host in relay mode ────────
            // C→S: [validator:Str][basket_id:u32][BasketContents][item_name:Str][chunk:Str][shorts×4]
            // S→C: [basket_id:u32][BasketContents][item_name:Str] (broadcast, no trailing chunk/shorts)
            0x1E => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    // Parse through to find where item_name ends (exclude trailing chunk+shorts)
                    let basket_start = off;
                    let after_basket_id = off + 4;
                    let after_contents = skip_basket_contents(data, after_basket_id);
                    let (_item_name, after_item_name) = unpack_string(data, after_contents);

                    // Build S→C: basket_id + BasketContents + item_name (no trailing fields)
                    let mut pkt = vec![0x1Eu8];
                    pkt.extend_from_slice(&data[basket_start..after_item_name]);
                    session.broadcast(&pkt, Some(uid.as_str()));

                    // In relay mode, also forward save to host
                    if matches!(session.mode, SessionMode::Relay) {
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            if uid != hname {
                                // Send to host for persistence:
                                // [0x1E] + String(requester) + basket_id + BasketContents + item_name
                                let mut relay = vec![0x1Eu8];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&data[basket_start..after_item_name]);
                                session.send_to(hname, &relay);
                            }
                        }
                    }
                }
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

                    // Update stored initial_data zone so late-joiners see the
                    // correct zone.  Layout: pos(8)+pos(8)+rot(8)+appearance(1)+String(zone)+…
                    if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                        let mut guard = p.initial_data.lock().unwrap();
                        if let Some(ref od) = *guard {
                            let z_off = 8 + 8 + 8 + 1; // 25
                            if od.len() > z_off {
                                let (_, after_zone) = unpack_string(od, z_off);
                                let mut new_od = od[..z_off].to_vec();
                                new_od.extend(pack_string(&zone_name));
                                if after_zone < od.len() {
                                    new_od.extend_from_slice(&od[after_zone..]);
                                }
                                *guard = Some(new_od);
                            }
                        }
                    }

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

            // ── END_TELEPORT (0x16) → update position + broadcast ─────────
            // C→S: raw position Short×4 (8 bytes) at data[10..]
            // S→C: String(username) + Short×4
            0x16 => {
                if let Some(ref uid) = player_id {
                    // Update stored initial_data position so late-joiners see
                    // the correct location.  Layout: pos(8)+pos(8)+rest…
                    let new_pos = &data[10..];
                    if new_pos.len() >= 8 {
                        if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                            let mut guard = p.initial_data.lock().unwrap();
                            if let Some(ref od) = *guard {
                                if od.len() >= 16 {
                                    let mut new_od = Vec::with_capacity(od.len());
                                    new_od.extend_from_slice(&new_pos[..8]); // pos slot 1
                                    new_od.extend_from_slice(&new_pos[..8]); // pos slot 2
                                    new_od.extend_from_slice(&od[16..]);     // rest unchanged
                                    *guard = Some(new_od);
                                }
                            }
                        }
                    }

                    let mut pkt = vec![0x16u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Bulk broadcast-relay: [pid][player_id][body] ──────────────
            0x09 | 0x18 | 0x19 | 0x23 |
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
        } // match pid
        } // while (frame loop)
    } // 'outer loop

    // Disconnect cleanup: remove player and notify others via 0x13 type=gone + 0x07 leave.
    if let Some(ref uid) = player_id {
        session.players.lock().unwrap().remove(uid.as_str());
        session.broadcast(&build_player_gone(uid), None);
        session.broadcast(&build_join_notif(uid, false), None);

        // Remove from world state tracking.
        if let SessionMode::Managed(ref world) = session.mode {
            world.players.write().unwrap().remove(uid.as_str());
        }

        // In relay mode, if the host disconnects, shut down the entire session.
        if matches!(session.mode, SessionMode::Relay) {
            let is_host = session.host.lock().unwrap()
                .as_ref()
                .map(|h| h == uid)
                .unwrap_or(false);
            if is_host {
                println!("[GAME:'{}'] Host '{}' left — shutting down relay session", session.room_token, uid);
                session.stop();
            }
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
            *session.listen_addr.lock().unwrap() = listener.local_addr().ok();
            let accept_session = Arc::clone(&session);
            println!("[GAME] Relay session '{}' → port {}", room_token, port);
            std::thread::spawn(move || {
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
                println!("[GAME] Relay session '{}' listener closed (port {})", accept_session.room_token, port);
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
