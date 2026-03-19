// game_server.rs — per-session multiplayer game server.
//
// Each active game session listens on its own dynamically allocated port.
// The friend server spawns a session when a host accepts a join request via
// JOIN_GRANT (0x2B) and uses the returned port in the JumpToGame (0x25) packet
// sent to both the host and the joiner.
//
// Adding a new C→S handler
// ─────────────────────────
// Add a match arm in `handle_client`.  Most relayed packets just need an entry
// in the bulk-relay arm at the bottom.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::packet::{craft_batch, pack_string, unpack_string};

// ── Per-session player ─────────────────────────────────────────────────────

struct GamePlayer {
    /// Cloned stream handle used by other threads to push data to this player.
    sink:         Mutex<TcpStream>,
    /// Last received PLAYER_DATA blob (C→S 0x03 body), replayed to players who join later.
    initial_data: Mutex<Option<Vec<u8>>>,
}

// ── Session shared state ────────────────────────────────────────────────────

struct Session {
    room_token: String,
    players:    Mutex<HashMap<String, Arc<GamePlayer>>>,
}

impl Session {
    fn new(room_token: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            room_token: room_token.into(),
            players:    Mutex::new(HashMap::new()),
        })
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

/// S→C 0x02 LOGIN_SUCCESS
///
/// RE from GameServerReceiver::OnReceive case 2:
///   GetString()  server_name
///   GetByte()    is_host
///   GetByte()    ignored
///   GetString()  validator_code
///   GetShort()   validator_variation
///   GetShort()   n_others   → if is_host && n_others > 0: n_others × GetString()
fn build_login_success(server_name: &str) -> Vec<u8> {
    let mut p = vec![0x02u8];
    p.extend(pack_string(server_name)); // server_name
    p.push(0x00);                       // is_host = false
    p.push(0x00);                       // ignored
    p.extend(pack_string(""));          // validator_code = ""
    p.extend_from_slice(&0i16.to_le_bytes()); // validator_variation = 0
    p.extend_from_slice(&0i16.to_le_bytes()); // n_others = 0
    p
}

/// S→C 0x05 FULLY_IN_GAME
///
/// RE from GameServerReceiver::OnReceive case 5:
///   GetShort()  n_ids → n_ids × GetLong() unique_id
///   GetShort()  daynight  (time × 1000 as i16; 12000 = noon)
///   GetShort()  n_perks → n_perks × GetString() perk_name
///   GetByte()   is_moderator
///   GetByte()   max_companions
///   GetByte()   last_byte  (0 → client requests zone via C→S 0x0A)
///   GetByte()   pvp
///   GetByte()   ignored
fn build_fully_in_game() -> Vec<u8> {
    let mut p = vec![0x05u8];
    p.extend_from_slice(&0i16.to_le_bytes());     // n_ids = 0
    p.extend_from_slice(&12000i16.to_le_bytes()); // daynight = noon
    p.extend_from_slice(&0i16.to_le_bytes());     // n_perks = 0
    p.push(0x00); // is_moderator
    p.push(0x00); // max_companions
    p.push(0x00); // last_byte = 0 → client will send REQ_ZONE_DATA
    p.push(0x00); // pvp
    p.push(0x00); // ignored
    p
}

/// S→C 0x0B ZONE_ASSIGNMENT
///
/// RE from GameServerReceiver::OnReceive case 0x0B:
///   GetByte()  status   (0 → UnknownZoneGotoSpawn(true, false))
///   GetByte()  is_host
fn build_zone_assignment() -> Vec<u8> {
    vec![0x0Bu8, 0x00, 0x00]
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
/// RE from GameServerReceiver::OnReceive case 0x13 (byte 1 = new):
///   GetByte()   1 = new
///   GetString() username
///   GetString() display_name
///   OnlinePlayerData::Unpack(reader)
///     UnpackPosition(at)
///     UnpackPosition(to)
///     UnpackRotation(rot)
///     GetByte()   is_dead
///     GetString() currently_using
///     GetString() sitting_in_chair
///     GetLong()   level
///     3 × InventoryItem::UnpackFromWeb
///     GetLong()   hp_max
///     GetLong()   hp
///     GetLong()   hp_regen
///     GetShort()  creature_count → creature_count × GetString() creature_name
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
            // C→S: [WorldName: Str] [Token: Str]
            // RE from GameServerSender$$SendLoginAttempt.
            //
            // Response sequence:
            //   S→C 0x02  LOGIN_SUCCESS   (client auto-calls SendInitialPlayerData)
            //   → client sends C→S 0x03
            //   S→C 0x05  FULLY_IN_GAME   (last_byte=0 → client sends REQ_ZONE_DATA)
            //   → client sends C→S 0x0A
            //   S→C 0x0B  ZONE_ASSIGNMENT (status=0 → UnknownZoneGotoSpawn)
            0x26 => {
                if player_id.is_some() { continue; } // ignore repeated logins

                let (_world, off) = unpack_string(data, 10);
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

                println!("[GAME:'{}'] {} → player_id='{}'", world, addr, uid);

                // S→C 0x02: login success
                let _ = stream.write_all(&craft_batch(2, &build_login_success(&world)));
                // 0x05 and 0x0B come after we receive C→S 0x03 (PLAYER_DATA).
            }

            // ── PLAYER_DATA (0x03) ─────────────────────────────────────────
            // Store and broadcast to others; replay existing players to newcomer.
            // Then send 0x05 FULLY_IN_GAME so the client requests zone data.
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

                    // 4. Send FULLY_IN_GAME (0x05) → client will send C→S 0x0A (REQ_ZONE_DATA).
                    let _ = stream.write_all(&craft_batch(2, &build_fully_in_game()));
                }
            }

            // ── REQ_ZONE_DATA (0x0A) ──────────────────────────────────────
            // C→S: [zone_name: Str] [type: u8] [if type 2|3: packed_position]
            // Respond with 0x0B status=0 is_host=0 → UnknownZoneGotoSpawn(true, false)
            // which lets the client pick its own spawn point.
            0x0A => {
                let _ = stream.write_all(&craft_batch(2, &build_zone_assignment()));
            }

            // ── REQ_CHUNK (0x0C) ──────────────────────────────────────────
            // We don't serve chunks — ignore silently.
            0x0C => {}

            // ── POSITION (0x11) → relay as S→C 0x11 ─────────────────────
            // RE from GameServerReceiver::OnReceive case 0x11:
            //   GetString()      username
            //   UnpackPosition() at
            //   UnpackPosition() to
            //   UnpackRotation() rot
            0x11 => {
                if let Some(ref uid) = player_id {
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

            // ── CHAT (0x06) → relay as S→C 0x1B ──────────────────────────
            0x06 => {
                if let Some(ref uid) = player_id {
                    let (msg, _) = unpack_string(data, 10);
                    let mut pkt = vec![0x1Bu8];
                    pkt.extend(pack_string(uid));
                    pkt.extend(pack_string(&msg));
                    session.broadcast(&pkt, Some(uid.as_str()));
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

            // ── Bulk broadcast-relay: [pid][player_id][body] ──────────────
            // These packets are relayed verbatim to all other players.
            0x09 | 0x16 | 0x18 | 0x19 | 0x20 | 0x21 | 0x22 | 0x23 |
            0x46 | 0x47 | 0x48 | 0x4A | 0x4B | 0x4E | 0x4F | 0x50 |
            0x51 | 0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x57 | 0x58 |
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

    // Disconnect cleanup: remove player and notify others via 0x13 type=gone.
    if let Some(ref uid) = player_id {
        session.players.lock().unwrap().remove(uid.as_str());
        session.broadcast(&build_player_gone(uid), None);
        println!("[GAME:'{}'] '{}' disconnected ({})", session.room_token, uid, addr);
    }
}

// ── Session spawner ────────────────────────────────────────────────────────

/// Scans `[cfg.game_port, cfg.game_port_max]` for a free TCP port, binds it,
/// starts a session listener thread, and returns the port number.
///
/// Returns `None` if no port in the range is available.
pub fn spawn_session(room_token: String, cfg: &Config) -> Option<u16> {
    for port in cfg.game_port..=cfg.game_port_max {
        let addr = format!("{}:{}", cfg.host, port);
        if let Ok(listener) = TcpListener::bind(&addr) {
            let session = Session::new(room_token.clone());
            println!("[GAME] Session '{}' → port {}", room_token, port);
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
