// game_server.rs — multiplayer game server.
//
// Two modes of operation:
//
//   Standalone (`run()`):
//     Binds a single port, serves world chunks from a `WorldState`,
//     tracks player positions.
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

pub mod baskets;
pub mod generator;
pub mod packets_client;
pub mod packets_server;
pub mod persist;
pub mod registry_client;
pub mod world_state;

use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

/// Global allocator for unique object IDs.
///
/// Each player receives a contiguous block of IDs.  Starting at 1_000_000
/// keeps us well above any hardcoded sentinel values the client might use.
/// IDs are 64-bit so overflow is not a practical concern.
static NEXT_UNIQUE_ID: AtomicI64 = AtomicI64::new(1_000_000);

fn pkt_name(pid: u8) -> &'static str {
    match pid {
        0x01 => "PING",
        0x02 => "JOIN",
        0x03 => "PLAYER_DATA",
        0x06 => "CHAT",
        0x09 => "GUARD_DIE_NOTIF",
        0x0A => "REQ_ZONE",
        0x0C => "REQ_CHUNK",
        0x0F => "HEARTBEAT",
        0x11 => "POSITION",
        0x14 => "CHANGE_ZONE",
        0x15 => "START_TELEPORT",
        0x16 => "END_TELEPORT",
        0x18 => "CHANGE_EQUIP",
        0x19 => "UPDATE_CREATURES",
        0x1A => "REQ_CONTAINER",
        0x1B => "CONTAINER_RESP",
        0x1E => "CLOSE_BASKET",
        0x20 => "BUILD_FURNITURE",
        0x21 => "REMOVE_OBJECT",
        0x22 => "REPLACE_BUILDABLE",
        0x23 => "CHANGE_LAND_USER",
        0x26 => "LOGIN",
        0x27 => "CLAIM_OBJECT",
        0x28 => "RELEASE_INTERACTING",
        0x29 => "REQ_MORE_IDS",
        0x2A => "UNIQUE_ID_SEND",
        0x2B => "USED_UNIQUE_ID",
        0x2D => "MUSIC_BOX_NOTE",
        0x2E => "REQ_TELE_PAGE",
        0x2F => "REQ_TELEPORTERS",
        0x30 => "TELE_SCREENSHOT",
        0x31 => "REQ_TELE_SCREENSHOT",
        0x33 => "EDIT_TELEPORTER",
        0x34 => "NEW_TELE_SEARCH",
        0x35 => "MINIGAME_CHALLENGE",
        0x36 => "MINIGAME_RESPONSE",
        0x37 => "BEGIN_MINIGAME",
        0x38 => "EXIT_MINIGAME",
        0x39 => "POOL_CUE_POS",
        0x3A => "POOL_SHOOT",
        0x3B => "POOL_SYNC_READY",
        0x3C => "POOL_PLACE_BALL",
        0x3D => "POOL_PLAY_AGAIN",
        0x3E => "FINISH_SIT",
        0x3F => "CLAIM_MOBS",
        0x40 => "DELOAD_MOB",
        0x41 => "MOB_POSITIONS",
        0x46 => "ATTACK_ANIM",
        0x47 => "HIT_MOB",
        0x48 => "MOB_DIE",
        0x4A => "CREATURE_STATS",
        0x4B => "INCREASE_HP",
        0x4C => "SHOW_EXP",
        0x4E => "COMPANION_EQUIP",
        0x4F => "RENAME_COMPANION",
        0x50 => "DESTROY_COMPANION",
        0x51 => "APPLY_PERK",
        0x52 => "LAUNCH_PERK",
        0x53 => "QUICK_TAG",
        0x54 => "ALL_PERKS",
        0x55 => "CREATE_PERK_DROP",
        0x56 => "RESPAWN",
        0x57 => "BACK_TO_BREEDER",
        0x58 => "MOB_TARGET_SYNC",
        0x59 => "CREATE_MOB",
        0x5A => "BANDIT_FLAG_DEST",
        _    => "UNKNOWN",
    }
}

use crate::utils::config::Config;
#[allow(unused_imports)]
use crate::defs::packet::{craft_batch, pack_string, unpack_string, write_payload, ServerPacket, LOG_PACKETS};
#[allow(unused_imports)]
use packets_client::{skip_basket_contents, GameClientPacket};
#[allow(unused_imports)]
use packets_server::{
    BasketUpdateBroadcast, BasketUpdateToHost, BeginMinigameRelay, ChatBroadcast,
    ChunkForGuest, ChunkRelayToHost, ContainerContents, ContainerRelayToHost,
    DayNight, HeartbeatReply, JoinConfirmed, JoinNotif, LoginResponse, SessionInit,
    MinigameChallengeRelay, MinigameResponseRelay, NamedRelayPacket, NoPrefixPacket,
    PlayerGone, PlayerNearby, PlayerPrefixPacket, Pong, PositionUpdate,
    ReleaseInteractingObject, SetInteractingObject,
    UniqueIds, ZoneChangeBroadcast, ZoneData, ZoneForGuest, ZoneRelayToHost, InteriorInfo,
};
use world_state::WorldState;
use generator::WorldTemplate;

// ── Per-session player ─────────────────────────────────────────────────────

struct GamePlayer {
    /// Cloned stream handle used by other threads to push data to this player.
    sink:         Mutex<TcpStream>,
    /// Last received PLAYER_DATA blob (C→S 0x03 body), replayed to players who join later.
    initial_data: Mutex<Option<Vec<u8>>>,
    /// Current zone (e.g. "overworld", "cave_zone"). Used to filter visibility —
    /// players only see other players in the same zone.
    zone:         Mutex<String>,
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
    /// Whether player-vs-player combat is enabled. Sent to clients as packet
    /// 0x05 during login so `CombatControl$HitAllowed` allows player damage.
    pvp_enabled: bool,
    /// Print hex dumps of every C→S packet to stdout when true.
    log_packets: bool,
    /// The address the listener is bound to — used to unblock the accept loop on shutdown.
    listen_addr: Mutex<Option<std::net::SocketAddr>>,
    /// The host player's username (relay mode only). The host client owns
    /// the world data; chunk/container requests from guests are relayed to it.
    host:        Mutex<Option<String>>,
    /// Pending chunk requests: chunk_key → list of requesting player usernames.
    pending_chunks:     Mutex<HashMap<String, Vec<String>>>,
    /// Pending container requests: basket_id → list of requesting player usernames.
    pending_containers: Mutex<HashMap<String, Vec<String>>>,
    /// Basket locking: basket_id → username of the player currently holding it open.
    open_baskets: Mutex<HashMap<i64, (String, String)>>,
}

impl Session {
    fn new(room_token: impl Into<String>, mode: SessionMode, pvp_enabled: bool, log_packets: bool) -> Arc<Self> {
        Arc::new(Self {
            room_token: room_token.into(),
            mode,
            pvp_enabled,
            log_packets,
            players: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
            listen_addr: Mutex::new(None),
            host: Mutex::new(None),
            pending_chunks: Mutex::new(HashMap::new()),
            pending_containers: Mutex::new(HashMap::new()),
            open_baskets: Mutex::new(HashMap::new()),
        })
    }

    pub(crate) fn player_count(&self) -> usize {
        self.players.lock().unwrap().len()
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

    /// Serialises `pkt` and sends it to all players except `exclude`.
    fn broadcast(&self, pkt: &impl ServerPacket, exclude: Option<&str>) {
        let payload = pkt.to_payload();
        for (name, p) in self.players.lock().unwrap().iter() {
            if exclude == Some(name.as_str()) { continue; }
            let _ = write_payload(&mut *p.sink.lock().unwrap(), 2, &payload);
        }
    }

    /// Like `broadcast`, but only sends to players in the same zone as `zone`.
    fn broadcast_zone(&self, pkt: &impl ServerPacket, zone: &str, exclude: Option<&str>) {
        let payload = pkt.to_payload();
        for (name, p) in self.players.lock().unwrap().iter() {
            if exclude == Some(name.as_str()) { continue; }
            if *p.zone.lock().unwrap() != zone { continue; }
            let _ = write_payload(&mut *p.sink.lock().unwrap(), 2, &payload);
        }
    }

    /// Serialises `pkt` and sends it to a single player by username.
    fn send_to(&self, target: &str, pkt: &impl ServerPacket) {
        if let Some(p) = self.players.lock().unwrap().get(target) {
            let _ = write_payload(&mut *p.sink.lock().unwrap(), 2, &pkt.to_payload());
        }
    }
}


/// Advance past one InventoryItem in `data` starting at `off`.
/// Returns `(item_bytes, new_off)` or `None` if the data is truncated.
///
/// Wire format:
///   i16(n_shorts)   + n_shorts  × [pack_string(key) + i16(val)]
///   i16(n_strings)  + n_strings × [pack_string(key) + pack_string(val)]
///   i16(n_ints)     + n_ints    × [pack_string(key) + i32(val)]
fn read_inventory_item(data: &[u8], start: usize) -> Option<(Vec<u8>, usize)> {
    let mut off = start;
    macro_rules! need { ($n:expr) => { if off + $n > data.len() { return None; } } }
    macro_rules! u16 { () => {{ need!(2); let v = u16::from_le_bytes([data[off], data[off+1]]); off += 2; v as usize }} }
    macro_rules! skip_str { () => {{ let l = u16!(); need!(l); off += l; }} }

    let n_shorts  = u16!();
    for _ in 0..n_shorts  { skip_str!(); need!(2); off += 2; }
    let n_strings = u16!();
    for _ in 0..n_strings { skip_str!(); skip_str!(); }
    let n_ints    = u16!();
    for _ in 0..n_ints    { skip_str!(); need!(4); off += 4; }
    Some((data[start..off].to_vec(), off))
}

/// Extracts `(shack_id, item_id)` from wire-format InventoryItem bytes.
///
/// Looks for int key `"shack_id = *long* "` and string key `"item_id"`.
/// Returns None if either key is absent (not a shack item).
fn parse_shack_info(data: &[u8]) -> Option<(i32, String)> {
    let mut off = 0usize;
    macro_rules! need { ($n:expr) => { if off + $n > data.len() { return None; } } }
    macro_rules! read_u16 { () => {{ need!(2); let v = u16::from_le_bytes([data[off], data[off+1]]); off += 2; v as usize }} }
    macro_rules! read_str { () => {{ let l = read_u16!(); need!(l); let s = String::from_utf8_lossy(&data[off..off+l]).into_owned(); off += l; s }} }

    let n_shorts = read_u16!();
    for _ in 0..n_shorts {
        let _ = read_str!();
        need!(2); off += 2;
    }
    let n_strings = read_u16!();
    let mut item_id: Option<String> = None;
    for _ in 0..n_strings {
        let key = read_str!();
        let val = read_str!();
        if key == "item_id" { item_id = Some(val); }
    }
    let n_ints = read_u16!();
    let mut shack_id: Option<i32> = None;
    for _ in 0..n_ints {
        let key = read_str!();
        need!(4);
        let val = i32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        off += 4;
        if key == "shack_id = *long* " { shack_id = Some(val); }
    }
    Some((shack_id?, item_id?))
}

/// Rewrite the `currently_using` string inside a stored OPD blob.
///
/// OPD layout: pos(8) + pos(8) + rot(8) + is_dead(1) = 25-byte fixed header,
/// followed immediately by pack_string(currently_using).
///
/// Pass `""` to clear (player not using anything), or a container key of the form
/// `zone/chunkX/chunkZ/innerX/innerZ` to mark the player as using a basket.
/// The client's `GameServerInterface.AnyoneUsing` compares nearby players'
/// `currently_using` against this key to show the "in use" popup.
fn opd_with_using(opd: &[u8], using_str: &str) -> Vec<u8> {
    const HDR: usize = 25;
    if opd.len() < HDR + 2 { return opd.to_vec(); }
    let old_len = u16::from_le_bytes([opd[HDR], opd[HDR + 1]]) as usize;
    let after_old = HDR + 2 + old_len;
    let mut out = Vec::with_capacity(after_old + using_str.len() + (opd.len().saturating_sub(after_old)));
    out.extend_from_slice(&opd[..HDR]);
    out.extend(pack_string(using_str));
    if after_old < opd.len() { out.extend_from_slice(&opd[after_old..]); }
    out
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
                let _ = write_payload(&mut stream, 0, &[0x09, 0x01]);
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

            if session.log_packets {
                use crate::defs::packet::to_hex_upper;
                let uid = player_id.as_deref().unwrap_or("?");
                println!("[C→S] [GAME:'{}'] {}({}) | 0x{:02X} ({}) | {}",
                    session.room_token, uid, addr, pid, pkt_name(pid), to_hex_upper(data));
            }

        match pid {

            // ── PING (0x01) ────────────────────────────────────────────────
            0x01 => {
                let _ = write_payload(&mut stream, 2, &[0x01]);
            }

            // ── HEARTBEAT (0x0F) ───────────────────────────────────────────
            0x0F => {
                let _ = write_payload(&mut stream, 2, &[0x0F]);
            }

            // ── LOGIN (0x26) ───────────────────────────────────────────────
            // C→S: [world_name: Str] [token: Str]
            //
            // Response sequence (matching Python game_server.py):
            //   S→C 0x26  LOGIN_RESPONSE
            //   S→C 0x2A  UNIQUE_IDS
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
                    zone:         Mutex::new("overworld".to_string()),
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
                let _ = write_payload(&mut stream, 2, &LoginResponse { world_name: &world, player_uid: &uid }.to_payload());

                // 2. S→C 0x02: join confirmed (is_host=true for host, false for guests)
                let _ = write_payload(&mut stream, 2, &JoinConfirmed { server_name: &world, username: &uid, is_host: is_host_flag }.to_payload());

                // 3. S→C 0x05: session init — delivers unique IDs and session state.
                //    unique IDs go here (uid_count field), not as standalone 0x2A.
                //    skip_saved_pos=1 prevents the client from overriding our spawn.
                const INITIAL_ID_BLOCK: u16 = 64;
                let id_start = NEXT_UNIQUE_ID.fetch_add(INITIAL_ID_BLOCK as i64, Ordering::Relaxed);
                let _ = write_payload(&mut stream, 2, &SessionInit {
                    daynight_ms:    12000,
                    client_is_mod:  false,
                    max_companions: 3,
                    pvp_enabled:    session.pvp_enabled,
                    uid_start:      id_start,
                    uid_count:      INITIAL_ID_BLOCK,
                }.to_payload());

                // 4. S→C 0x0B: zone data
                let zone_name = match session.mode {
                    SessionMode::Managed(ref ws) => ws.default_zone.clone(),
                    SessionMode::Relay => "overworld".to_string(),
                };
                let _ = write_payload(&mut stream, 2, &ZoneData { zone_name: &zone_name, interior: None }.to_payload());

                // 6. S→C 0x07: join notification (broadcast to others)
                session.broadcast(&JoinNotif { username: &uid, joined: true }, Some(uid.as_str()));
            }

            // ── PLAYER_DATA (0x03) ─────────────────────────────────────────
            // C→S 0x03 (SendInitialPlayerData):
            //   Pos(8) + Str(zone) + u8(body_slot) + i32(level) + Item×3
            //   + i32(hp_max) + i32(hp) + i32(hp_regen) + i16(creature_count)
            //   + creature_count × Str(name) + [host-only trailing data]
            //
            // Must transform to OnlinePlayerData for S→C 0x13 type=1:
            //   Pos(at) + Pos(to) + Rot(rot) + u8(is_dead)
            //   + Str(currently_using) + Str(sitting_in_chair)
            //   + i32(level) + Item×3 + i32(hp_max) + i32(hp) + i32(hp_regen)
            //   + i16(creature_count) + creature_count × Str(name)
            //
            // Player sync is delayed 2 seconds (matching Python) so both
            // clients have time to finish loading the zone before receiving
            // 0x13 NEW_PLAYER_NEARBY packets.
            0x03 => {
                if let Some(ref uid) = player_id {
                    let raw = &data[10..];
                    if raw.len() >= 8 {
                        let mut off = 0usize;
                        let pos_bytes = &raw[off..off + 8]; off += 8;
                        let (zone_name_raw, new_off) = unpack_string(raw, off); off = new_off;
                        let body_slot = if off < raw.len() { raw[off] } else { 0 }; off += 1;

                        let rest = if off < raw.len() { &raw[off..] } else { &[] };

                        // In managed mode: validate the saved zone against the world's
                        // zone list, then send ZoneData to teleport the client into it
                        // if it differs from the default (which was sent at login).
                        // Unknown zones (e.g. from a different world) fall back to default.
                        let zone_name = if let SessionMode::Managed(ref ws) = session.mode {
                            let known = !zone_name_raw.is_empty()
                                && ws.generator.template().zones.iter()
                                    .any(|z| z.name == zone_name_raw);
                            let effective = if known {
                                zone_name_raw.clone()
                            } else {
                                ws.default_zone.clone()
                            };
                            if effective != ws.default_zone {
                                let _ = write_payload(&mut stream, 2,
                                    &ZoneData { zone_name: &effective, interior: None }.to_payload());
                            }
                            effective
                        } else {
                            zone_name_raw
                        };

                        // Update server-side TrackedPlayer with the position from 0x03.
                        if let SessionMode::Managed(ref ws) = session.mode {
                            use world_state::WorldPosition;
                            let pos = WorldPosition {
                                chunk_x: i16::from_le_bytes([pos_bytes[0], pos_bytes[1]]),
                                chunk_z: i16::from_le_bytes([pos_bytes[2], pos_bytes[3]]),
                                local_x: i16::from_le_bytes([pos_bytes[4], pos_bytes[5]]),
                                local_z: i16::from_le_bytes([pos_bytes[6], pos_bytes[7]]),
                            };
                            let mut players = ws.players.write().unwrap();
                            if let Some(tp) = players.get_mut(uid.as_str()) {
                                tp.zone     = zone_name.clone();
                                tp.position = pos;
                                tp.target   = pos;
                            }
                        }

                        // Build OnlinePlayerData blob.
                        // OPD: at(8) + to(8) + rot(8) + is_dead(1) + Str(currently_using)
                        //      + Str(sitting_in_chair) + level + items×3 + hp stats + creatures
                        // `rest` = level + items + hp + creatures + host-only trailing data.
                        // The trailing host-only data is harmless — the client stops
                        // reading after creature names and ignores extra bytes.
                        let mut opd = Vec::new();
                        opd.extend_from_slice(pos_bytes);                    // at position
                        opd.extend_from_slice(pos_bytes);                    // to position (same)
                        opd.extend_from_slice(&[0, 0, 0, 0, 0, 0, 100, 0]); // rotation: identity quat ×100
                        opd.push(body_slot);                                 // is_dead byte (0 = alive)
                        opd.extend(pack_string(""));                         // currently_using (empty = not using anything)
                        opd.extend(pack_string(""));                         // sitting_in_chair
                        opd.extend_from_slice(rest);                         // level + items + hp + creatures

                        // Store initial_data + zone immediately.
                        if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                            *p.initial_data.lock().unwrap() = Some(opd.clone());
                            *p.zone.lock().unwrap() = zone_name.clone();
                        }

                        // Delayed sync: wait 2s then broadcast + reciprocal sync.
                        let sess = Arc::clone(&session);
                        let uid_owned = uid.clone();
                        let opd_owned = opd;
                        let zone_owned = zone_name.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(1));

                            // 1. Broadcast this player to everyone in the same zone.
                            let spawn_pkt = PlayerNearby { username: &uid_owned, display: &uid_owned, opd: &opd_owned }.to_payload();
                            sess.broadcast_zone(&spawn_pkt, &zone_owned, Some(uid_owned.as_str()));

                            // 2. Send existing same-zone players' data to this newcomer.
                            let existing: Vec<(String, Vec<u8>)> = sess.players.lock().unwrap()
                                .iter()
                                .filter(|(n, _)| n.as_str() != uid_owned.as_str())
                                .filter(|(_, p)| *p.zone.lock().unwrap() == zone_owned)
                                .filter_map(|(n, p)| {
                                    p.initial_data.lock().unwrap()
                                        .as_ref()
                                        .map(|d| (n.clone(), d.clone()))
                                })
                                .collect();
                            for (name, init) in &existing {
                                sess.send_to(&uid_owned, &PlayerNearby { username: name, display: name, opd: init }.to_payload());
                            }
                            // For each existing player that has a basket open, send
                            // 0x27 so the newcomer's AnyoneUsing (field+48) is set.
                            let open: Vec<(String, String)> = sess.open_baskets.lock().unwrap()
                                .values()
                                .filter(|(holder, _)| holder != &uid_owned)
                                .cloned()
                                .collect();
                            for (holder, key) in &open {
                                sess.send_to(&uid_owned, &SetInteractingObject {
                                    player:     holder,
                                    object_key: key,
                                }.to_payload());
                            }

                            // 3. In relay mode, explicitly notify the host about
                            //    this guest so the host's client spawns them.
                            if matches!(sess.mode, SessionMode::Relay) {
                                let host = sess.host.lock().unwrap().clone();
                                if let Some(ref hname) = host {
                                    if uid_owned != *hname {
                                        sess.send_to(hname, &spawn_pkt);
                                        println!("[GAME:'{}'] Synced guest '{}' to host '{}'",
                                                 sess.room_token, uid_owned, hname);
                                    }
                                }
                            }
                        });
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
                            let zone = if zone_name.is_empty() { ws.default_zone.clone() } else { zone_name };
                            let zones = ws.zones.read().unwrap();
                            let interior = zones.get(&zone)
                                .and_then(|e| e.interior.as_ref())
                                .map(|id| InteriorInfo {
                                    item_bytes: &id.item_bytes,
                                    rotation: id.rotation,
                                    cx: id.cx, cz: id.cz,
                                    tx: id.tx, tz: id.tz,
                                    outer_zone: &id.outer_zone,
                                });
                            let payload = ZoneData { zone_name: &zone, interior }.to_payload();
                            let _ = write_payload(&mut stream, 2, &payload);
                        }
                        SessionMode::Relay => {
                            let host = session.host.lock().unwrap().clone();
                            let is_host = host.as_ref().map(|h| h == uid).unwrap_or(false);
                            if is_host {
                                // Host gets zone data directly.
                                let zone = if zone_name.is_empty() { "overworld".to_string() } else { zone_name };
                                let _ = write_payload(&mut stream, 2, &ZoneData { zone_name: &zone, interior: None }.to_payload());
                            } else if let Some(ref hname) = host {
                                // Guest → relay to host.
                                // Host expects: Str(zone_name) + Str(requester) + u8(type) [+ Pos if type 2|3]
                                let mut relay = vec![0x0Au8];
                                relay.extend(pack_string(&zone_name));
                                relay.extend(pack_string(uid));
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
                    // C→S: [str zone][i16 x][i16 z][u8 dimension][str sub_zone]
                    let (zone_name, off) = unpack_string(data, 10);
                    if data.len() >= off + 4 {
                        let x = i16::from_le_bytes([data[off], data[off + 1]]);
                        let z = i16::from_le_bytes([data[off + 2], data[off + 3]]);
                        // Skip dimension byte (off+4), read sub_zone at off+5
                        let (sub_zone, _) = if data.len() > off + 5 {
                            unpack_string(data, off + 5)
                        } else {
                            (String::new(), off + 5)
                        };

                        match session.mode {
                            SessionMode::Managed(ref world) => {
                                let wire = world.get_chunk_wire(x, z);
                                let _ = write_payload(&mut stream, 2, &wire);
                            }
                            SessionMode::Relay => {
                                let host = session.host.lock().unwrap().clone();
                                let is_host = host.as_ref().map(|h| h == uid).unwrap_or(false);
                                if is_host {
                                    // Host reads chunks locally (ShouldSaveLocally=true),
                                    // but the client still sends 0x0C to the server.
                                    // Send back an empty chunk so it doesn't stall.
                                    let empty = world_state::Chunk::blank(x, z, &zone_name).to_wire();
                                    let _ = write_payload(&mut stream, 2, &empty);
                                } else if host.is_some() {
                                    // Guest → relay request to host.
                                    // Host expects: Str(requester) + Str(zone) + Str(sub_zone) + i16(x) + i16(z)
                                    let chunk_key = format!("{}_{}_{}_{}",  zone_name, sub_zone, x, z);
                                    session.pending_chunks.lock().unwrap()
                                        .entry(chunk_key).or_default().push(uid.clone());

                                    let mut relay = vec![0x0Cu8];
                                    relay.extend(pack_string(uid));
                                    relay.extend(pack_string(&zone_name));
                                    relay.extend(pack_string(&sub_zone));
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
            // C→S: [validator:Str][basket_id:u32][type:u8][zone:Str]
            //       [chunkX:i16][chunkZ:i16][innerX:i16][innerZ:i16]
            // Relay mode: forward to host as 0x1B + String(requester) + u32(basket_id)
            // Host responds with 0x1B + String(requester) + u32(basket_id) + BasketContents
            // Server strips requester and forwards as 0x1A + u32(basket_id) + BasketContents
            // Managed mode: respond immediately with 0x1A + u32(basket_id) + BasketContents
            //
            // basket_id is a u32 (4 bytes), matching InventoryItem "long" property
            // encoding which is also 4 bytes despite the name.
            //
            // On success: update this player's OPD currently_using to the container
            // key (zone/chunkX/chunkZ/innerX/innerZ) and broadcast PlayerNearby to
            // others so their AnyoneUsing check returns true and shows the popup.
            // On lock rejection: send the holder's OPD to the requester so they
            // learn it's in use and close the connecting popup next tap.
            0x1A => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    if data.len() >= off + 4 {
                        let basket_id = u32::from_le_bytes([
                            data[off], data[off+1], data[off+2], data[off+3],
                        ]) as i64;
                        off += 4;
                        off += 1; // skip type byte
                        let (zone, mut off2) = unpack_string(data, off);
                        let (chunkx, chunkz, innerx, innerz) = if data.len() >= off2 + 8 {
                            let cx = i16::from_le_bytes([data[off2],   data[off2+1]]); off2 += 2;
                            let cz = i16::from_le_bytes([data[off2],   data[off2+1]]); off2 += 2;
                            let ix = i16::from_le_bytes([data[off2],   data[off2+1]]); off2 += 2;
                            let iz = i16::from_le_bytes([data[off2],   data[off2+1]]);
                            (cx, cz, ix, iz)
                        } else {
                            (0, 0, 0, 0)
                        };
                        let container_key = format!("{}/{}/{}/{}/{}", zone, chunkx, chunkz, innerx, innerz);

                        // Basket locking: reject if already held by another player.
                        let locked_by = {
                            let mut locks = session.open_baskets.lock().unwrap();
                            match locks.get(&basket_id).cloned() {
                                Some((ref holder, _)) if holder != uid => Some(holder.clone()),
                                None => { locks.insert(basket_id, (uid.clone(), container_key.clone())); None }
                                _ => None, // same player re-opening
                            }
                        };

                        if let Some(ref holder) = locked_by {
                            // Basket is in use: send 0x27 to the requester so their
                            // AnyoneUsing (field+48) check returns true and shows the popup.
                            session.send_to(uid, &SetInteractingObject {
                                player:     holder,
                                object_key: &container_key,
                            }.to_payload());
                            continue;
                        }

                        // Lock acquired — update OPD and tell all zone peers via 0x27.
                        let player_zone = session.players.lock().unwrap()
                            .get(uid.as_str())
                            .map(|p| p.zone.lock().unwrap().clone())
                            .unwrap_or_default();
                        if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                            let mut guard = p.initial_data.lock().unwrap();
                            if let Some(ref od) = *guard {
                                *guard = Some(opd_with_using(od, &container_key));
                            }
                        }
                        // 0x27 sets field+48 on the client's OnlinePlayer — the field
                        // that AnyoneUsing actually checks. Broadcast to all peers so
                        // they see the basket as locked immediately.
                        session.broadcast_zone(
                            &SetInteractingObject { player: uid, object_key: &container_key },
                            &player_zone, Some(uid.as_str()));

                        if matches!(session.mode, SessionMode::Relay) {
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                let bk = basket_id.to_string();
                                session.pending_containers.lock().unwrap()
                                    .entry(bk).or_default().push(uid.clone());

                                let mut relay = vec![0x1Cu8];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&(basket_id as u32).to_le_bytes());
                                session.send_to(hname, &relay);
                            }
                        } else if let SessionMode::Managed(ref world) = session.mode {
                            let contents = world.baskets.get_contents(basket_id);
                            let mut out = vec![0x1Bu8];
                            out.extend_from_slice(&(basket_id as u32).to_le_bytes());
                            out.extend_from_slice(&contents);
                            session.send_to(uid, &out);
                        }
                    }
                }
            }

            // ── HOST CONTAINER RESPONSE (0x1B) — relay mode only ─────────
            // Host sends: 0x1B + String(requester) + u32(basket_id) + BasketContents
            // Guest expects: 0x1B + u32(basket_id) + BasketContents  (same ID as the request)
            0x1B if matches!(session.mode, SessionMode::Relay) => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (_requester, off) = unpack_string(data, 10);
                        if data.len() >= off + 4 {
                            let basket_id = u32::from_le_bytes([
                                data[off], data[off+1], data[off+2], data[off+3],
                            ]) as i64;

                            let mut out = vec![0x1Bu8];
                            out.extend_from_slice(&data[off..]); // u32(basket_id) + contents

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

            // ── CLOSE_BASKET (0x1E) ───────────────────────────────────────
            // C→S: [validator:Str][basket_id:u32][BasketContents][item_name:Str][chunk:Str][shorts×4]
            // S→C: [basket_id:u32][BasketContents][item_name:Str] (no trailing chunk/shorts)
            0x1E => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip validator
                    let basket_start = off;
                    let after_basket_id = off + 4; // basket_id is u32 (4 bytes)
                    let after_contents = skip_basket_contents(data, after_basket_id);
                    let (_item_name, after_item_name) = unpack_string(data, after_contents);

                    // Release basket lock and clear currently_using in this player's OPD,
                    // then broadcast the updated OPD so nearby clients update AnyoneUsing.
                    if after_basket_id <= data.len() {
                        let lid = i64::from(u32::from_le_bytes([
                            data[basket_start], data[basket_start+1],
                            data[basket_start+2], data[basket_start+3],
                        ]));
                        session.open_baskets.lock().unwrap().remove(&lid);
                    }
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    {
                        if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                            let mut guard = p.initial_data.lock().unwrap();
                            if let Some(ref od) = *guard {
                                *guard = Some(opd_with_using(od, ""));
                            }
                        }
                    }
                    // 0x28 clears field+48 on the client's OnlinePlayer so
                    // AnyoneUsing stops blocking the basket for other players.
                    session.broadcast_zone(
                        &ReleaseInteractingObject { player: uid },
                        &player_zone, Some(uid.as_str()));

                    // Build S→C: basket_id + BasketContents + item_name (no trailing fields)
                    let mut pkt = vec![0x1Eu8];
                    pkt.extend_from_slice(&data[basket_start..after_item_name]);
                    session.broadcast(&pkt, Some(uid.as_str()));

                    // In relay mode, also forward save to host
                    if matches!(session.mode, SessionMode::Relay) {
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            if uid != hname {
                                let mut relay = vec![0x1Eu8];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&data[basket_start..after_item_name]);
                                session.send_to(hname, &relay);
                            }
                        }
                    } else if let SessionMode::Managed(ref world) = session.mode {
                        if after_basket_id <= data.len() && after_contents <= data.len() {
                            let basket_id = u32::from_le_bytes([
                                data[basket_start],     data[basket_start + 1],
                                data[basket_start + 2], data[basket_start + 3],
                            ]) as i64;
                            let contents = &data[after_basket_id..after_contents];
                            world.baskets.put(basket_id, contents);
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

                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![0x11u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
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
                    let (raw_msg, _) = unpack_string(data, 10);
                    let msg = crate::utils::text::strip_rich_text(&raw_msg);
                    let mut pkt = vec![0x06u8];
                    pkt.extend(pack_string(uid));  // player_id
                    pkt.extend(pack_string(uid));  // display_name (same for now)
                    pkt.extend(pack_string(&msg)); // message
                    pkt.push(0x00);                // type: 0 = public
                    session.broadcast(&pkt, None); // include sender so they see own msg
                }
            }

            // ── TELE_START (0x15) → broadcast to same zone ──────────────
            // C→S: [str tele_str]
            // S→C: [str destination]
            0x15 => {
                if let Some(ref uid) = player_id {
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![0x15u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
                }
            }

            // ── ZONE_CHANGE (0x14) → broadcast zone change ─────────────────
            // When a player moves between zones:
            //   1. Send "player gone" (0x13 type=0) to players in the OLD zone
            //   2. Update the player's zone
            //   3. Broadcast zone change to everyone
            //   4. Send "player nearby" (0x13 type=1) to players in the NEW zone
            0x14 => {
                if let Some(ref uid) = player_id {
                    let (zone_name, _) = unpack_string(data, 10);

                    // Get old zone and current initial_data.
                    let old_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();

                    // Send "gone" to old zone players.
                    if old_zone != zone_name {
                        session.broadcast_zone(
                            &PlayerGone { username: uid }, &old_zone, Some(uid.as_str()));
                    }

                    // Update player zone tracker. Clear currently_using because the
                    // player left any open basket behind when they changed zones.
                    if let Some(p) = session.players.lock().unwrap().get(uid.as_str()) {
                        *p.zone.lock().unwrap() = zone_name.clone();
                        let mut guard = p.initial_data.lock().unwrap();
                        if let Some(ref od) = *guard {
                            *guard = Some(opd_with_using(od, ""));
                        }
                    }
                    // Release any basket lock and notify old-zone peers.
                    session.open_baskets.lock().unwrap().retain(|_, (holder, _)| holder != uid);
                    session.broadcast_zone(
                        &ReleaseInteractingObject { player: uid },
                        &old_zone, Some(uid.as_str()));

                    // Broadcast zone change to everyone (the client uses this
                    // for its own tracking regardless of visibility).
                    let mut pkt = vec![0x14u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend(pack_string(&zone_name));
                    session.broadcast(&pkt, Some(uid.as_str()));

                    // Send "nearby" to new zone players so they see this player.
                    if old_zone != zone_name {
                        let init = session.players.lock().unwrap()
                            .get(uid.as_str())
                            .and_then(|p| p.initial_data.lock().unwrap().clone());
                        if let Some(init) = init {
                            session.broadcast_zone(
                                &PlayerNearby { username: uid, display: uid, opd: &init },
                                &zone_name, Some(uid.as_str()));
                        }
                    }
                }
            }

            // ── UNIQUE_IDS_RESPONSE (0x2A) ────────────────────────────────
            // C→S: [Str(uid)][Short(count)][count × Long(id)]
            //
            // IDA confirmed: client sends this in response to S→C 0x29 (which
            // asks the client to generate IDs locally).  We never send S→C 0x29
            // (we use S→C 0x2A instead), so this should be rare in managed mode.
            //
            // In relay mode: the host might have sent S→C 0x29 to a guest via
            // some other path; relay the response back to the host so it can
            // track which IDs belong to which player for cleanup on disconnect.
            // In managed mode: we allocated IDs ourselves, so just drop this.
            0x2A => {
                if matches!(session.mode, SessionMode::Relay) {
                    if let Some(ref uid) = player_id {
                        let is_host = session.host.lock().unwrap()
                            .as_ref().map(|h| h == uid).unwrap_or(false);
                        if !is_host {
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                let mut relay = vec![0x2Au8];
                                relay.extend_from_slice(&data[10..]);
                                session.send_to(hname, &relay);
                            }
                        }
                    }
                }
                // managed mode: drop — we allocated IDs ourselves
            }

            // ── MUSIC_BOX_NOTE (0x2D) — relay to named target ────────────
            // C→S: [str target_player][note data...] (point-to-point)
            0x2D => {
                if let Some(ref uid) = player_id {
                    let (target, off) = unpack_string(data, 10);
                    let mut pkt = vec![0x2Du8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[off..]);
                    session.send_to(&target, &pkt);
                }
            }

            // ── USED_UNIQUE_ID (0x2B) ────────────────────────────────────
            // C→S from GameServerSender$$SendUsedUniqueId:
            //   [0x2B][u32(used_id)]  (Packet.GetLong = 4 bytes)
            //
            // Broadcast to peers so they drop the ID from their per-player
            // unique_ids_given_away dict.  S→C wire (case 0x2B in receiver):
            //   [0x2B][Str(player)][u32(id)]
            //
            // NOTE: 0x2B is YOU_MAY_JOIN on the *friend* server — unrelated.
            0x2B => {
                if let Some(ref uid) = player_id {
                    if data.len() >= 10 + 4 {
                        let mut pkt = vec![0x2Bu8];
                        pkt.extend(pack_string(uid));
                        pkt.extend_from_slice(&data[10..10 + 4]);
                        session.broadcast(&pkt, Some(uid.as_str()));
                    }
                }
            }

            // ── BUILD (0x20) ───────────────────────────────────────────────
            // C→S: [validator:Str][Item][rot:u8][zone:Str][Short×4 cx,cz,tx,tz][extra:Str]
            // S→C: [Item][rot:u8][zone:Str][Short×4][owner:Str][extra]
            // DO NOT echo back to builder (client already placed it locally).
            0x20 => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    if let Some((item_bytes, next)) = read_inventory_item(data, off) {
                        off = next;
                        if off < data.len() {
                            let rotation = data[off]; off += 1;
                            let (zone_str, next) = unpack_string(data, off); off = next;
                            if off + 8 <= data.len() {
                                let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                                let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                                let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                                let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                                let shorts_bytes = &data[off..off+8]; off += 8;
                                let extra = &data[off..];

                                // Broadcast S→C: Item + rot + zone + shorts×4 + owner + extra
                                let mut pkt = vec![0x20u8];
                                pkt.extend_from_slice(&item_bytes);
                                pkt.push(rotation);
                                pkt.extend(pack_string(&zone_str));
                                pkt.extend_from_slice(shorts_bytes);
                                pkt.extend(pack_string(uid));
                                pkt.extend_from_slice(extra);
                                session.broadcast(&pkt, Some(uid.as_str()));

                                // Persist to WorldState in managed mode.
                                if let SessionMode::Managed(ref ws) = session.mode {
                                    use world_state::ChunkElement;
                                    let mut chunks = ws.chunks.write().unwrap();
                                    let chunk = chunks.entry((cx, cz)).or_insert_with(|| {
                                        let params = ws.generator.chunk_params(&ws.default_zone, cx as i32, cz as i32);
                                        world_state::Chunk {
                                            x: cx, z: cz, zone: ws.default_zone.clone(),
                                            biome: params.biome, floor_rot: params.floor_rot,
                                            floor_tex: params.floor_tex, floor_model: 0,
                                            mob_a: params.mob_a, mob_b: params.mob_b,
                                            elements: params.elements.into_iter()
                                                .map(|p| ChunkElement { cell_x: p.cell_x, cell_z: p.cell_z, rotation: p.rotation, item_data: p.item_data })
                                                .collect(),
                                        }
                                    });
                                    chunk.elements.push(ChunkElement {
                                        cell_x: tx as u8, cell_z: tz as u8,
                                        rotation, item_data: item_bytes.clone(),
                                    });
                                    drop(chunks);
                                    // Register zone for any item that carries a shack_id.
                                    if let Some((shack_id, _)) = parse_shack_info(&item_bytes) {
                                        let zone_name = format!("shack9072{}", shack_id);
                                        ws.zones.write().unwrap().insert(zone_name, world_state::ZoneEntry {
                                            interior: Some(world_state::InteriorData {
                                                item_bytes,
                                                rotation,
                                                cx, cz, tx, tz,
                                                outer_zone: zone_str.clone(),
                                            }),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── REMOVE_OBJECT (0x21) ──────────────────────────────────────
            // C→S: [validator:Str][zone:Str][Short×4 cx,cz,tx,tz][rot:u8][Item][extra:Str]
            // S→C: [zone:Str][Short×4][rot:u8][Item][owner:Str]
            0x21 => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    let (zone_str, next) = unpack_string(data, off); off = next;
                    if off + 8 <= data.len() {
                        let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                        let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                        let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                        let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                        let shorts_bytes = &data[off..off+8]; off += 8;
                        let rotation = if off < data.len() { data[off] } else { 0 }; off += 1;
                        if let Some((item_bytes, next)) = read_inventory_item(data, off) {
                            off = next;
                            let extra = &data[off..];

                            // Broadcast S→C: zone + shorts×4 + rot + item + owner
                            let mut pkt = vec![0x21u8];
                            pkt.extend(pack_string(&zone_str));
                            pkt.extend_from_slice(shorts_bytes);
                            pkt.push(rotation);
                            pkt.extend_from_slice(&item_bytes);
                            pkt.extend(pack_string(uid));
                            pkt.extend_from_slice(extra);
                            session.broadcast(&pkt, Some(uid.as_str()));

                            // Remove from WorldState in managed mode.
                            if let SessionMode::Managed(ref ws) = session.mode {
                                if let Some(chunk) = ws.chunks.write().unwrap().get_mut(&(cx, cz)) {
                                    // Remove by matching tile position + rotation + item.
                                    // Match on position first; item_data as tiebreaker.
                                    let target_x = tx as u8;
                                    let target_z = tz as u8;
                                    if let Some(pos) = chunk.elements.iter().position(|e| {
                                        e.cell_x == target_x && e.cell_z == target_z
                                            && e.rotation == rotation && e.item_data == item_bytes
                                    }) {
                                        chunk.elements.remove(pos);
                                    } else {
                                        // Fallback: match tile only (handles rotation mismatch)
                                        if let Some(pos) = chunk.elements.iter().position(|e| {
                                            e.cell_x == target_x && e.cell_z == target_z
                                        }) {
                                            chunk.elements.remove(pos);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Couldn't parse — fall back to raw relay
                        let mut pkt = vec![0x21u8];
                        pkt.extend_from_slice(&data[off..]);
                        session.broadcast(&pkt, Some(uid.as_str()));
                    }
                }
            }

            // ── REPLACE_BUILDABLE (0x22) ──────────────────────────────────
            // C→S: [validator:Str][old_Item][new_Item][rot:u8][zone:Str][shorts×4][extra:Str]
            // S→C: [old_Item][new_Item][rot:u8][zone:Str][shorts×4][owner:Str][extra]
            0x22 => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    if let Some((old_item, next)) = read_inventory_item(data, off) {
                        off = next;
                        if let Some((new_item, next)) = read_inventory_item(data, off) {
                            off = next;
                            if off < data.len() {
                                let rotation = data[off]; off += 1;
                                let (zone_str, next) = unpack_string(data, off); off = next;
                                if off + 8 <= data.len() {
                                    let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                                    let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                                    let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                                    let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                                    let shorts_bytes = &data[off..off+8]; off += 8;
                                    let extra = &data[off..];

                                    // S→C: old_Item + new_Item + rot + zone + shorts×4 + owner + extra
                                    let mut pkt = vec![0x22u8];
                                    pkt.extend_from_slice(&old_item);
                                    pkt.extend_from_slice(&new_item);
                                    pkt.push(rotation);
                                    pkt.extend(pack_string(&zone_str));
                                    pkt.extend_from_slice(shorts_bytes);
                                    pkt.extend(pack_string(uid));
                                    pkt.extend_from_slice(extra);
                                    session.broadcast(&pkt, Some(uid.as_str()));

                                    // Replace in WorldState for managed mode.
                                    if let SessionMode::Managed(ref ws) = session.mode {
                                        if let Some(chunk) = ws.chunks.write().unwrap().get_mut(&(cx, cz)) {
                                            let tx8 = tx as u8;
                                            let tz8 = tz as u8;
                                            // Remove old element (exact match; tile-only fallback)
                                            let pos = chunk.elements.iter().position(|e| {
                                                e.cell_x == tx8 && e.cell_z == tz8
                                                    && e.rotation == rotation && e.item_data == old_item
                                            }).or_else(|| chunk.elements.iter().position(|e| {
                                                e.cell_x == tx8 && e.cell_z == tz8
                                            }));
                                            if let Some(i) = pos { chunk.elements.remove(i); }
                                            // Add new element
                                            use world_state::ChunkElement;
                                            chunk.elements.push(ChunkElement {
                                                cell_x: tx8, cell_z: tz8,
                                                rotation, item_data: new_item,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Malformed — relay as-is without owner
                        let (_, off) = unpack_string(data, 10);
                        let mut pkt = vec![0x22u8];
                        pkt.extend_from_slice(&data[off..]);
                        session.broadcast(&pkt, Some(uid.as_str()));
                    }
                }
            }

            // ── ATTACK_ANIM (0x46) — no validator, zone-scoped relay ─────
            // C→S: [str combat_id]
            0x46 => {
                if let Some(ref uid) = player_id {
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![0x46u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
                }
            }

            // ── Combat: strip fn_validator (first string), relay ────────
            // 0x47 HIT_MOB, 0x48 MOB_DIE — validator is first field on wire
            0x47 | 0x48 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip fn_validator
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

            // ── MOB_DATA_REQUEST (0x42) — point-to-point relay ───────────
            // C→S: [target_owner: Str] [rest...]
            // Relay to target as: [0x42][requester: Str][rest...]
            // The guest asks a specific player (usually the host) for mob
            // data (companions, NPCs, etc). The owner responds with 0x43.
            0x42 => {
                if let Some(ref uid) = player_id {
                    let (target_owner, off) = unpack_string(data, 10);
                    if !target_owner.is_empty() {
                        let mut relay = vec![0x42u8];
                        relay.extend(pack_string(uid));        // requester
                        relay.extend_from_slice(&data[off..]);  // rest of body
                        session.send_to(&target_owner, &relay);
                    }
                }
            }

            // ── MOB_DATA_RESPONSE (0x43) — point-to-point relay ──────────
            // C→S: [target_requester: Str] [rest...]
            // Relay to target as: [0x43][responder: Str][rest...]
            0x43 => {
                if let Some(ref uid) = player_id {
                    let (target_requester, off) = unpack_string(data, 10);
                    if !target_requester.is_empty() {
                        let mut relay = vec![0x43u8];
                        relay.extend(pack_string(uid));        // responder
                        relay.extend_from_slice(&data[off..]);  // rest of body
                        session.send_to(&target_requester, &relay);
                    }
                }
            }

            // ── GUARD_DIE (0x09) — relay body as-is, no player prefix ────
            // C→S: [str mob_name][str owner_name]
            // S→C: [str guard_name][str additional_info] — no prefix
            0x09 => {
                if player_id.is_some() {
                    let mut pkt = vec![0x09u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, None);
                }
            }

            // ── Zone-scoped player-prefixed relay ───────────────────────
            // S→C format has [str player_name] first — visual/state updates
            // visible to players in the same zone.
            // 0x18 EQUIP_CHANGE, 0x19 CREATURE_CHANGE,
            // 0x4E COMPANION_EQUIP, 0x4F RENAME_COMPANION, 0x50 DESTROY_COMPANION
            0x18 | 0x19 | 0x4E | 0x4F | 0x50 => {
                if let Some(ref uid) = player_id {
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![pid];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
                }
            }

            // ── Strip fn_validator, broadcast ───────────────────────────
            // 0x4B INCREASE_HP, 0x53 QUICK_TAG — validator is first field
            0x4B | 0x53 => {
                if let Some(ref uid) = player_id {
                    let (_, off) = unpack_string(data, 10); // skip fn_validator
                    let mut pkt = vec![pid];
                    pkt.extend_from_slice(&data[off..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Global player-prefixed relay ────────────────────────────
            // Packets where S→C format has [str player_name] and all
            // players need to see it regardless of zone.
            // 0x23 LAND_CLAIM, 0x4A (unknown), 0x56 RESPAWN,
            // 0x57 RETURNING_TO_BREEDER, 0x58 UPDATE_SYNCED_TARGETS,
            // 0x59 CREATED_LOCAL_MOB, 0x5A BANDIT_FLAG_DESTROYED
            0x23 | 0x4A | 0x56 | 0x57 | 0x58 | 0x59 | 0x5A => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![pid];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Teleporter packets — relay to host ──────────────────────
            // These are processed by the host client which owns teleporter
            // data. The host responds with S→C packets directly.
            //
            // 0x2E REQ_TELE_PAGE: guest asks for a page of teleporters
            // 0x2F TELE_PAGE_DATA: host sends page back (needs routing)
            // 0x30 TELE_SCREENSHOT_UPLOAD: screenshot data for a teleporter
            // 0x31 REQ_TELE_SCREENSHOT: guest asks for a screenshot
            // 0x32 TELE_SCREENSHOT_RESPONSE: host sends screenshot back
            // 0x33 FINISHED_EDITING_TELE: notify others of edit
            // 0x34 NEW_TELE_SEARCH: search request → relay to host
            0x2E | 0x31 | 0x34 => {
                // Guest→host: relay with requester name so host can respond
                if let Some(ref uid) = player_id {
                    if matches!(session.mode, SessionMode::Relay) {
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            let is_host = hname == uid;
                            if !is_host {
                                let mut relay = vec![pid];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&data[10..]);
                                session.send_to(hname, &relay);
                            }
                        }
                    }
                }
            }

            // 0x2F TELE_PAGE_DATA — host→guest point-to-point
            // C→S: [str user_requesting][i16 page][u8 has_more][u8 count][tele data...]
            // Route the response to the named user.
            0x2F => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (target_user, off) = unpack_string(data, 10);
                        if !target_user.is_empty() {
                            // S→C 0x2F: [i16 page][u8 has_more][u8 count][tele data...]
                            let mut pkt = vec![0x2Fu8];
                            pkt.extend_from_slice(&data[off..]);
                            session.send_to(&target_user, &pkt);
                        }
                    } else {
                        // Non-host sending 0x2F — relay to host
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            let mut relay = vec![0x2Fu8];
                            relay.extend(pack_string(uid));
                            relay.extend_from_slice(&data[10..]);
                            session.send_to(hname, &relay);
                        }
                    }
                }
            }

            // 0x30 TELE_SCREENSHOT_UPLOAD — broadcast to all (screenshot cache)
            0x30 => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![0x30u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // 0x32 TELE_SCREENSHOT_RESPONSE — host→guest point-to-point
            // Host sends: [str requester][str tele_id][i32 size][bytes...]
            0x32 => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (target_user, off) = unpack_string(data, 10);
                        if !target_user.is_empty() {
                            let mut pkt = vec![0x32u8];
                            pkt.extend_from_slice(&data[off..]);
                            session.send_to(&target_user, &pkt);
                        }
                    }
                }
            }

            // 0x33 FINISHED_EDITING_TELE — broadcast to all
            0x33 => {
                if player_id.is_some() {
                    let mut pkt = vec![0x33u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, None);
                }
            }

            // ── Minigame packets — point-to-point relay ─────────────────
            // Minigames are between two named players. Most packets include
            // a target player name as the first field.
            //
            // 0x35 CHALLENGE_MINIGAME: [str target_username][u8 type]
            // 0x36 MINIGAME_RESPONSE: [u8 response][str challenger][u8 type]
            // 0x37 BEGIN_MINIGAME: [str owner][u8 response][u8 type][ball_layout...]
            // 0x38 EXIT_MINIGAME: empty payload — broadcast to all
            // 0x39 POOL_CUE_POS: [i32 angle] — broadcast to all
            // 0x3A POOL_SHOOT: [i32 deg][i16 power][recording...] — broadcast
            // 0x3B POOL_SYNC_READY: empty — broadcast
            // 0x3C POOL_PLACE_WHITE: [i16 x][i16 y] — broadcast
            // 0x3D POOL_PLAY_AGAIN: empty — broadcast

            // 0x35 CHALLENGE — route to target player
            0x35 => {
                if let Some(ref uid) = player_id {
                    let (target, off) = unpack_string(data, 10);
                    if !target.is_empty() {
                        let mut pkt = vec![0x35u8];
                        pkt.extend(pack_string(uid)); // challenger
                        pkt.extend_from_slice(&data[off..]); // minigame_type
                        session.send_to(&target, &pkt);
                    }
                }
            }

            // 0x36 MINIGAME_RESPONSE — route to challenger
            0x36 => {
                if let Some(ref uid) = player_id {
                    // C→S: [u8 response][str challenger][u8 type]
                    if data.len() > 10 {
                        let response = data[10];
                        let (challenger, off) = unpack_string(data, 11);
                        if !challenger.is_empty() {
                            let mut pkt = vec![0x36u8];
                            pkt.extend(pack_string(uid)); // responder name first
                            pkt.push(response);
                            if off < data.len() {
                                pkt.extend_from_slice(&data[off..]); // type
                            }
                            session.send_to(&challenger, &pkt);
                        }
                    }
                }
            }

            // 0x37 BEGIN_MINIGAME — route to named owner
            0x37 => {
                if let Some(ref uid) = player_id {
                    let (owner, off) = unpack_string(data, 10);
                    if !owner.is_empty() {
                        let mut pkt = vec![0x37u8];
                        pkt.extend(pack_string(uid)); // sender
                        pkt.extend_from_slice(&data[off..]); // response + type + ball_layout
                        session.send_to(&owner, &pkt);
                    }
                }
            }

            // 0x38-0x3D POOL/MINIGAME STATE — broadcast to all players
            // These are realtime game state updates visible to all.
            0x38 | 0x39 | 0x3A | 0x3B | 0x3C | 0x3D => {
                if let Some(ref uid) = player_id {
                    // S→C format: raw payload only, no player name prefix
                    let mut pkt = vec![pid];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── SIT_IN_CHAIR (0x3E) — zone-scoped broadcast ────────────
            // C→S: [str chair_id]  (empty = finished sitting)
            // S→C: [str player_name][str chair_id]
            0x3E => {
                if let Some(ref uid) = player_id {
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![0x3Eu8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
                }
            }

            // ── TRY_CLAIM_MOBS (0x3F) — relay to host ──────────────────
            // C→S: [i16 count][mob data...] — host processes mob ownership
            0x3F => {
                if let Some(ref uid) = player_id {
                    if matches!(session.mode, SessionMode::Relay) {
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            let mut relay = vec![0x3Fu8];
                            relay.extend(pack_string(uid));
                            relay.extend_from_slice(&data[10..]);
                            session.send_to(hname, &relay);
                        }
                    }
                }
            }

            // ── DELOAD_MOB (0x40) — broadcast ───────────────────────────
            // C→S: [str combat_id]
            0x40 => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![0x40u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── EXP_RECEIVE (0x4C) — zone-scoped, no prefix ────────────
            // C→S: [str text][Pos position] — visual popup
            0x4C => {
                if let Some(ref uid) = player_id {
                    let player_zone = session.players.lock().unwrap()
                        .get(uid.as_str())
                        .map(|p| p.zone.lock().unwrap().clone())
                        .unwrap_or_default();
                    let mut pkt = vec![0x4Cu8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast_zone(&pkt, &player_zone, Some(uid.as_str()));
                }
            }

            // ── CLAIM_OBJECT (0x27) / RELEASE_INTERACTING (0x28) ────────
            // 0x27: [str obj_str] — relay to host for object ownership
            // 0x28: empty — relay to host
            0x27 | 0x28 => {
                if let Some(ref uid) = player_id {
                    if matches!(session.mode, SessionMode::Relay) {
                        let host = session.host.lock().unwrap().clone();
                        if let Some(ref hname) = host {
                            let mut relay = vec![pid];
                            relay.extend(pack_string(uid));
                            relay.extend_from_slice(&data[10..]);
                            session.send_to(hname, &relay);
                        }
                    }
                }
            }

            // ── REQ_UNIQUE_IDS (0x29) — allocate another block of IDs ────
            // Client sends this (no payload) when its pool is running low.
            // Respond immediately with S→C 0x2A so the pool refills before it
            // drops below 2 (which triggers the "Unique IDs Exhausted" disconnect).
            // Sent unconditionally — some clients fire this before 0x26 completes.
            0x29 => {
                const REFILL_BLOCK: u16 = 64;
                let id_start = NEXT_UNIQUE_ID.fetch_add(REFILL_BLOCK as i64, Ordering::Relaxed);
                let _ = write_payload(&mut stream, 2, &UniqueIds { start: id_start, count: REFILL_BLOCK }.to_payload());
            }

            // ── Host-originated S→C packets — relay as-is ─────────────
            // These are already in S→C format when the host sends them.
            // The server must NOT prepend a player name.
            //
            // 0x08 COMPANION_DEATH_CHAT: [str player_name][str message]
            // 0x24 LAND_CLAIM_TIMER: [u8 type][str chunk_key][timer data...]
            // 0x25 ZONE_DATA_REFRESH: ZoneData::UnpackFromWeb body
            // 0x45 MOB_RESPAWN: [str combat_id] — destroy + respawn mob
            0x08 | 0x24 | 0x25 | 0x45 => {
                if let Some(ref uid) = player_id {
                    // Only the host should be sending these.
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host || !matches!(session.mode, SessionMode::Relay) {
                        let mut pkt = vec![pid];
                        pkt.extend_from_slice(&data[10..]);
                        session.broadcast(&pkt, Some(uid.as_str()));
                    }
                }
            }

            // ── Unknown — relay with player prefix ────────────────────────
            _ => {
                if let Some(ref uid) = player_id {
                    if session.log_packets {
                        use crate::defs::packet::to_hex_upper;
                        println!("[GAME:'{}'] [RELAY/UNKNOWN] 0x{:02X} from {} — relaying to zone | {}",
                            session.room_token, pid, uid, to_hex_upper(data));
                    }
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
        let player_zone = session.players.lock().unwrap()
            .get(uid.as_str())
            .map(|p| p.zone.lock().unwrap().clone())
            .unwrap_or_default();
        session.players.lock().unwrap().remove(uid.as_str());
        session.open_baskets.lock().unwrap().retain(|_, (holder, _)| holder != uid);
        session.broadcast_zone(
            &ReleaseInteractingObject { player: uid },
            &player_zone, Some(uid.as_str()));
        session.broadcast(&PlayerGone { username: uid }, None);
        session.broadcast(&JoinNotif { username: uid, joined: false }, None);

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
/// Loads existing world state from `<world_data_dir>/world.hws` if present,
/// otherwise generates a fresh world. Saves state every 5 minutes.
pub fn run(cfg: &Config) {
    let addr = format!("{}:{}", cfg.host, cfg.game_port);
    let listener = TcpListener::bind(&addr)
        .unwrap_or_else(|e| panic!("Failed to bind game server to {}: {}", addr, e));
    println!("[GAME] Standalone game server listening on {} ...", addr);

    let data_dir = std::path::PathBuf::from(&cfg.world_data_dir);
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("[GAME] Warning: could not create world_data_dir {}: {}", data_dir.display(), e);
    }
    let save_path    = data_dir.join(persist::FILE_NAME);
    let baskets_path = data_dir.join(baskets::FILE_NAME);

    let mut ws = match persist::load(&save_path) {
        Ok(ws) => {
            let chunk_count = ws.chunks.read().unwrap().len();
            println!("[GAME] Loaded world from {} ({} chunks)", save_path.display(), chunk_count);
            ws
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("[GAME] No save file found — generating fresh world");
            } else {
                eprintln!("[GAME] Failed to load world ({}), generating fresh", e);
            }
            let seed = match cfg.world_seed {
                Some(s) if s != 0 => s,
                _ => rand::random::<u64>(),
            };
            println!("[GAME] World seed: {}", seed);
            let mut tmpl = WorldTemplate::default();
            tmpl.seed = seed;
            WorldState::new("World", 5, tmpl)
        }
    };

    // Load baskets from their own file (independent of world.hws).
    match baskets::load(&baskets_path) {
        Ok(store) => {
            println!("[GAME] Loaded {} baskets from {}", store.len(), baskets_path.display());
            ws.baskets = store;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("[GAME] No basket file found — starting with empty store");
        }
        Err(e) => {
            eprintln!("[GAME] Failed to load baskets ({}), starting fresh", e);
        }
    }

    let world = Arc::new(ws);

    // Mutex shared between the SIGTERM handler and the background save thread
    // so they never write the world file concurrently (both write to a .tmp
    // then rename — two concurrent saves would corrupt the .tmp file).
    let save_lock = Arc::new(std::sync::Mutex::<()>::new(()));

    // SIGTERM / SIGINT handler: save world and exit cleanly.
    {
        let world_ref = Arc::clone(&world);
        let path      = save_path.clone();
        let bpath     = baskets_path.clone();
        let lock      = Arc::clone(&save_lock);
        ctrlc::set_handler(move || {
            println!("[GAME] Shutdown signal — saving world...");
            let _guard = lock.lock().unwrap();
            match persist::save(&world_ref, &path) {
                Ok(())  => println!("[GAME] World saved, exiting."),
                Err(e)  => eprintln!("[GAME] Save failed on shutdown: {}", e),
            }
            match baskets::save(&world_ref.baskets, &bpath) {
                Ok(())  => println!("[GAME] Baskets saved to {}", bpath.display()),
                Err(e)  => eprintln!("[GAME] Basket save failed on shutdown: {}", e),
            }
            std::process::exit(0);
        }).unwrap_or_else(|e| eprintln!("[GAME] Warning: could not install signal handler: {}", e));
    }

    // Background save thread: persists the full world every 5 minutes.
    {
        let world_ref = Arc::clone(&world);
        let path      = save_path.clone();
        let bpath     = baskets_path.clone();
        let lock      = Arc::clone(&save_lock);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(300));
                let _guard = lock.lock().unwrap();
                match persist::save(&world_ref, &path) {
                    Ok(()) => println!("[GAME] World saved to {}", path.display()),
                    Err(e) => eprintln!("[GAME] Save failed: {}", e),
                }
                match baskets::save(&world_ref.baskets, &bpath) {
                    Ok(()) => println!("[GAME] Baskets saved to {}", bpath.display()),
                    Err(e) => eprintln!("[GAME] Basket save failed: {}", e),
                }
            }
        });
    }

    LOG_PACKETS.store(cfg.log_packets, std::sync::atomic::Ordering::Relaxed);
    let session = Session::new(&cfg.server_name, SessionMode::Managed(Arc::clone(&world)), cfg.pvp_enabled, cfg.log_packets);

    // Optionally register with a friend server.
    if !cfg.friend_registry_host.is_empty()
        && cfg.friend_registry_port != 0
        && !cfg.friend_registry_secret.is_empty()
    {
        let public_ip = if cfg.public_ip.is_empty() { cfg.host.clone() } else { cfg.public_ip.clone() };
        let room_token = if cfg.server_room_token.is_empty() {
            session.room_token.clone()
        } else {
            cfg.server_room_token.clone()
        };
        registry_client::spawn(
            registry_client::RegistryParams {
                registry_addr: format!("{}:{}", cfg.friend_registry_host, cfg.friend_registry_port),
                secret:        cfg.friend_registry_secret.clone(),
                server_name:   cfg.server_name.clone(),
                server_desc:   cfg.server_desc.clone(),
                server_desc2:  cfg.server_desc2.clone(),
                server_desc3:  cfg.server_desc3.clone(),
                server_desc4:  cfg.server_desc4.clone(),
                max_players:   cfg.server_max_players,
                game_mode:     cfg.server_game_mode.clone(),
                public_ip,
                game_port:     cfg.game_port,
                room_token,
            },
            Arc::clone(&session),
        );
    }

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

    // Final save on clean shutdown (reached if `incoming` iterator ends).
    match persist::save(&world, &save_path) {
        Ok(()) => println!("[GAME] Final world save to {}", save_path.display()),
        Err(e) => eprintln!("[GAME] Final save failed: {}", e),
    }
    match baskets::save(&world.baskets, &baskets_path) {
        Ok(()) => println!("[GAME] Final basket save to {}", baskets_path.display()),
        Err(e) => eprintln!("[GAME] Final basket save failed: {}", e),
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
            let session = Session::new(room_token.clone(), SessionMode::Relay, false, true);
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

