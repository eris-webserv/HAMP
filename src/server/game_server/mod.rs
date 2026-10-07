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
mod mobs;
pub mod packets_client;
pub mod packets_server;
pub mod persist;
pub mod registry_client;
pub mod special_generators;
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
        0x2F => "TELE_PAGE_DATA",
        0x30 => "TELE_SCREENSHOT",
        0x31 => "REQ_TELE_SCREENSHOT",
        0x32 => "TELE_SCREENSHOT_RESP",
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
        0xE0 => "MOD_HELLO",
        0xE1 => "MOD_CHANNEL",
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
use generator::{
    BiomeWeights, WorldTemplate, ZoneConfig,
    BIOME_GRASS, BIOME_SNOW, BIOME_DESERT, BIOME_EVERGREEN,
    BIOME_OCEAN, BIOME_SWAMP, BIOME_WOODLANDS, BIOME_SAKURA,
};

/// Resolves a config `start_biome` string to a biome ID.
/// Returns `None` (no override) for an empty / unknown name.
fn parse_start_biome(name: &str) -> Option<i16> {
    match name.trim() {
        "" => None,
        s => match s.to_ascii_lowercase().as_str() {
            "grassland" | "grass"            => Some(BIOME_GRASS as i16),
            "snow"                           => Some(BIOME_SNOW as i16),
            "desert"                         => Some(BIOME_DESERT as i16),
            "evergreen"                      => Some(BIOME_EVERGREEN as i16),
            "ocean"                          => Some(BIOME_OCEAN as i16),
            "swamp"                          => Some(BIOME_SWAMP as i16),
            "woodlands"                      => Some(BIOME_WOODLANDS as i16),
            "sakura"                         => Some(BIOME_SAKURA as i16),
            _ => None,
        },
    }
}

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
    /// Usernames whose teleporters are listed first and marked with ★ (managed mode).
    admin_users: Vec<String>,
    /// Last teleporter search term per player (managed mode). Search-page
    /// requests (0x2E with in_search=1) page through results for this term.
    tele_search: Mutex<HashMap<String, String>>,
    wildlife: Mutex<mobs::Wildlife>,
    /// Players that completed the MOD_HELLO (0xE0) handshake, keyed by
    /// username. Custom packets are only ever sent to players in this map.
    mod_clients: Mutex<HashMap<String, ModClientInfo>>,
    /// When false (default), debug builds of HAModHelper are kicked at
    /// MOD_HELLO — i.e. this is a "Normal" server. Set true for an "Anarchy"
    /// server that accepts debug clients. See HAMP_MOD_PROTOCOL.md.
    allow_debug: bool,
}

impl Session {
    fn new(room_token: impl Into<String>, mode: SessionMode, pvp_enabled: bool, log_packets: bool, admin_users: Vec<String>, allow_debug: bool) -> Arc<Self> {
        Arc::new(Self {
            room_token: room_token.into(),
            mode,
            pvp_enabled,
            log_packets,
            allow_debug,
            players: Mutex::new(HashMap::new()),
            shutdown: AtomicBool::new(false),
            listen_addr: Mutex::new(None),
            host: Mutex::new(None),
            pending_chunks: Mutex::new(HashMap::new()),
            pending_containers: Mutex::new(HashMap::new()),
            open_baskets: Mutex::new(HashMap::new()),
            admin_users,
            tele_search: Mutex::new(HashMap::new()),
            wildlife: Mutex::new(mobs::Wildlife::default()),
            mod_clients: Mutex::new(HashMap::new()),
        })
    }

    fn is_admin(&self, user: &str) -> bool {
        self.admin_users.iter().any(|a| a.eq_ignore_ascii_case(user))
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

/// Skips over a ZoneData::PackForWeb body and returns the offset after it.
/// Format: [InventoryItem][rot:u8][coords:i16×4][outer_zone:str]
///          [land_claim_count:i16] [per claim: [key:str][time:i16×6][user:str×3]]
fn skip_zone_data_body(data: &[u8], start: usize) -> Option<usize> {
    // Skip InventoryItem
    let (_, mut off) = read_inventory_item(data, start)?;
    // rotation byte + 4 × i16 coords (1 + 8 = 9 bytes)
    if off + 9 > data.len() { return None; }
    off += 9;
    // outer_item_zone string
    if off + 2 > data.len() { return None; }
    off += 2 + u16::from_le_bytes([data[off], data[off+1]]) as usize;
    if off > data.len() { return None; }
    // land_claim_count
    if off + 2 > data.len() { return None; }
    let claim_count = u16::from_le_bytes([data[off], data[off+1]]) as usize;
    off += 2;
    for _ in 0..claim_count {
        // claim_key string
        if off + 2 > data.len() { return None; }
        off += 2 + u16::from_le_bytes([data[off], data[off+1]]) as usize;
        if off > data.len() { return None; }
        // 6 × i16 (second/minute/hour/day/month/year)
        if off + 12 > data.len() { return None; }
        off += 12;
        // user0, user1, user2 strings
        for _ in 0..3 {
            if off + 2 > data.len() { return None; }
            off += 2 + u16::from_le_bytes([data[off], data[off+1]]) as usize;
            if off > data.len() { return None; }
        }
    }
    Some(off)
}

/// Extracts `(shack_id, item_id)` from wire-format InventoryItem bytes.
///
/// InventoryItem strings are UTF-16LE length-prefixed (same as all game strings).
/// Returns None if either key is absent (not a shack item).
pub(super) fn parse_shack_info(data: &[u8]) -> Option<(i32, String)> {
    let mut off = 0usize;
    macro_rules! need { ($n:expr) => { if off + $n > data.len() { return None; } } }
    macro_rules! read_u16 { () => {{
        need!(2);
        let v = u16::from_le_bytes([data[off], data[off+1]]);
        off += 2;
        v as usize
    }} }
    macro_rules! read_str { () => {{
        let (s, next) = unpack_string(data, off);
        if next == off { return None; }
        off = next;
        s
    }} }

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
        if key == "shack_id" { shack_id = Some(val); }
    }
    Some((shack_id?, item_id?))
}

/// Returns the `item_id` string from a wire-format InventoryItem blob, or None.
fn parse_item_id(data: &[u8]) -> Option<String> {
    let mut off = 0usize;
    macro_rules! need { ($n:expr) => { if off + $n > data.len() { return None; } } }
    macro_rules! ru16 { () => {{ need!(2); let v = u16::from_le_bytes([data[off], data[off+1]]); off += 2; v as usize }} }
    macro_rules! read_str { () => {{ let (s, next) = unpack_string(data, off); if next == off { return None; } off = next; s }} }

    let n_shorts = ru16!();
    for _ in 0..n_shorts { let _ = read_str!(); need!(2); off += 2; }
    let n_strings = ru16!();
    for _ in 0..n_strings {
        let key = read_str!();
        let val = read_str!();
        if key == "item_id" { return Some(val); }
    }
    None
}

/// One row of the ordered teleporter listing (raw title, no display markup).
struct TeleListEntry {
    title: String,
    description: String,
    tele_str: String,
    zone: String,
    cx: i16,
    cz: i16,
    tx: i16,
    tz: i16,
    built_by: String,
    is_admin: bool,
}

impl TeleListEntry {
    /// Packs this entry in the S→C 0x2F / hamp:tele wire layout:
    /// [str title][str desc][str tele_str][str zone][i16×4][str built_by].
    /// `title` lets callers apply display markup (e.g. the ★ admin prefix).
    fn pack_into(&self, pkt: &mut Vec<u8>, title: &str) {
        pkt.extend(pack_string(title));
        pkt.extend(pack_string(&self.description));
        pkt.extend(pack_string(&self.tele_str));
        pkt.extend(pack_string(&self.zone));
        pkt.extend_from_slice(&self.cx.to_le_bytes());
        pkt.extend_from_slice(&self.cz.to_le_bytes());
        pkt.extend_from_slice(&self.tx.to_le_bytes());
        pkt.extend_from_slice(&self.tz.to_le_bytes());
        pkt.extend(pack_string(&self.built_by));
    }
}

/// Builds the ordered teleporter listing for a managed world: admin-built
/// teleporters first (stable creation order within each group), optionally
/// filtered by a case-insensitive title substring search.
fn tele_listing(session: &Session, ws: &WorldState, search_term: &str) -> Vec<TeleListEntry> {
    let term = search_term.to_lowercase();
    let teles = ws.teleporters.read().unwrap();
    let mut list: Vec<_> = teles.iter()
        .filter(|t| t.is_listed())
        .filter(|t| term.is_empty() || t.title.to_lowercase().contains(&term))
        .collect();
    list.sort_by_key(|t| !session.is_admin(&t.built_by)); // stable: admins first
    list.into_iter().map(|t| TeleListEntry {
        title: t.title.clone(),
        description: t.description.clone(),
        tele_str: t.tele_str(),
        zone: t.zone.clone(),
        cx: t.cx, cz: t.cz, tx: t.tx, tz: t.tz,
        built_by: t.built_by.clone(),
        is_admin: session.is_admin(&t.built_by),
    }).collect()
}

/// Sends one S→C 0x2F teleporter page to `requester` (managed mode).
///
/// Wire format (RE'd from GSR OnReceive case 0x2F):
///   [i16 page][u8 is_search][u8 has_more]
///   up to 3 × [u8 1][str title][str desc][str tele_str][str zone]
///              [i16 cx][i16 cz][i16 tx][i16 tz][str built_by]
///   [u8 0] terminator when fewer than 3 entries
fn send_tele_page(session: &Session, ws: &WorldState, requester: &str, page: i16, is_search: bool) {
    let term = if is_search {
        session.tele_search.lock().unwrap().get(requester).cloned().unwrap_or_default()
    } else {
        String::new()
    };
    let list = tele_listing(session, ws, &term);

    let start = (page.max(0) as usize) * 3;
    let has_more = start + 3 < list.len();

    let mut pkt = vec![0x2Fu8];
    pkt.extend_from_slice(&page.to_le_bytes());
    pkt.push(is_search as u8);
    pkt.push(has_more as u8);
    let mut count = 0;
    for e in list.iter().skip(start).take(3) {
        pkt.push(1);
        // ★ prefix marks admin teleporters in the stock single-list UI.
        let title = if e.is_admin { format!("★ {}", e.title) } else { e.title.clone() };
        e.pack_into(&mut pkt, &title);
        count += 1;
    }
    if count < 3 {
        pkt.push(0);
    }
    session.send_to(requester, &pkt);
}

// ── HAModHelper custom-packet layer ───────────────────────────────────────
//
// Opcodes 0xE0/0xE1 sit far outside the game's range (max 0x5A); the stock
// client's OnReceive hits `default: return` on them, so they are safe even
// if one slips through. The server still only *sends* custom packets to
// players that completed the MOD_HELLO handshake.
// Spec: docs/protocol/HAMP_MOD_PROTOCOL.md.

/// Version of the HAMP ↔ HAModHelper custom packet protocol.
const MOD_PROTOCOL_VERSION: i16 = 1;

/// Channels this server understands, advertised in MOD_WELCOME.
const MOD_CHANNELS: &[&str] = &["hamp:core", "hamp:tele"];

/// Grace period for a relay guest to send MOD_HELLO before the mod-match
/// check treats them as unmodded.
const MOD_CHECK_GRACE_SECS: u64 = 5;

/// Per-player handshake data from MOD_HELLO (0xE0).
#[allow(dead_code)]
struct ModClientInfo {
    protocol: i16,
    helper_version: String,
    mods: Vec<String>,
}

/// Sends a MOD_CHANNEL (0xE1) message to one player.
/// Silently does nothing if the player never sent MOD_HELLO.
fn send_mod_channel(session: &Session, target: &str, channel: &str, payload: &[u8]) {
    if !session.mod_clients.lock().unwrap().contains_key(target) {
        return;
    }
    let mut pkt = vec![0xE1u8];
    pkt.extend(pack_string(channel));
    pkt.extend_from_slice(payload);
    session.send_to(target, &pkt);
}

/// Returns a player's effective mod set: sorted mod ids if they completed
/// MOD_HELLO, empty otherwise. Vanilla and "HAModHelper with zero mods" are
/// deliberately equivalent — only actual mods gate compatibility.
fn effective_modset(session: &Session, user: &str) -> Vec<String> {
    session.mod_clients.lock().unwrap().get(user)
        .map(|m| {
            let mut v = m.mods.clone();
            v.sort();
            v
        })
        .unwrap_or_default()
}

/// Human-readable mod list for kick reasons: HAModHelper mod ids are
/// "guid@sha256" — show the guid plus a short hash prefix, not 64 hex chars.
/// The hash is the part after the LAST '@', and is only shortened when it
/// actually looks like a hash, so ids with '@' in the name stay intact.
fn display_modlist(mods: &[String]) -> String {
    mods.iter()
        .map(|m| match m.rsplit_once('@') {
            Some((name, hash))
                if hash.len() >= 16 && hash.chars().all(|c| c.is_ascii_hexdigit()) =>
                    format!("{} ({})", name, &hash[..8]),
            _ => m.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Relay-mode mod compatibility check for a guest against the session host.
/// Returns a kick reason when the effective mod sets differ.
fn relay_mod_mismatch(session: &Session, uid: &str) -> Option<String> {
    if !matches!(session.mode, SessionMode::Relay) {
        return None;
    }
    let host = session.host.lock().unwrap().clone()?;
    if host == uid {
        return None;
    }
    let host_mods = effective_modset(session, &host);
    let guest_mods = effective_modset(session, uid);
    if host_mods == guest_mods {
        return None;
    }
    Some(if guest_mods.is_empty() {
        format!("This world requires mods: {}", display_modlist(&host_mods))
    } else if host_mods.is_empty() {
        format!("This world is unmodded — your mods must be disabled to join: {}", display_modlist(&guest_mods))
    } else {
        format!("Mod mismatch — world runs: {} | you run: {}",
            display_modlist(&host_mods), display_modlist(&guest_mods))
    })
}

/// Kicks a player: best-effort reason via chat and hamp:core (modded clients
/// can show a proper dialog), then drops the connection. The player's read
/// thread runs the normal disconnect cleanup.
fn kick_player(session: &Session, uid: &str, reason: &str) {
    println!("[GAME:'{}'] Kicking '{}': {}", session.room_token, uid, reason);
    let mut body = vec![0x01u8]; // hamp:core 0x01 = kick notice
    body.extend(pack_string(reason));
    send_mod_channel(session, uid, "hamp:core", &body);

    // Give the client time to actually receive and dispatch the notices above
    // before we drop the socket. Closing immediately makes the client's
    // disconnect detection win the race against its packet pump, so the mod
    // never sees the hamp:core notice and the stock "Lost connection to Host"
    // screen is shown instead of our reason. Half a second is well inside the
    // client's frame loop and imperceptible for a kick.
    std::thread::sleep(std::time::Duration::from_millis(200));

    if let Some(p) = session.players.lock().unwrap().get(uid) {
        // Half-close (send FIN after the queued notices) rather than a full
        // shutdown: SHUT_RD can turn late inbound bytes into an RST that
        // discards everything still in flight, including the kick notice.
        let _ = p.sink.lock().unwrap().shutdown(std::net::Shutdown::Write);
    }
}

/// Dispatches a MOD_CHANNEL (0xE1) message from a modded client.
/// Unknown channels are dropped (logged when log_packets is on) so client
/// mods can probe for server support without consequences.
fn handle_mod_channel(session: &Session, uid: &str, channel: &str, payload: &[u8]) {
    match channel {
        // hamp:core — server→client notices only:
        //   S→C [u8 0x01][str reason]  kick notice, sent just before disconnect.
        // Client→server messages on this channel are ignored.
        "hamp:core" => {}

        // hamp:tele — teleporter extensions beyond the stock pager.
        // C→S [u8 0x01][i16 page]: request a page of ADMIN teleporters only.
        // S→C [u8 0x01][i16 page][u8 has_more][u8 count][count × entry]
        //     entry = [str title][str desc][str tele_str][str zone]
        //             [i16 cx][i16 cz][i16 tx][i16 tz][str built_by]
        //     (raw titles — no ★ prefix; the mod renders its own marker)
        "hamp:tele" => {
            if let SessionMode::Managed(ref ws) = session.mode {
                if payload.len() >= 3 && payload[0] == 0x01 {
                    let page = i16::from_le_bytes([payload[1], payload[2]]).max(0);
                    let list: Vec<TeleListEntry> = tele_listing(session, ws, "")
                        .into_iter()
                        .filter(|e| e.is_admin)
                        .collect();
                    let start = (page as usize * 3).min(list.len());
                    let end = (start + 3).min(list.len());
                    let mut body = vec![0x01u8];
                    body.extend_from_slice(&page.to_le_bytes());
                    body.push((end < list.len()) as u8);
                    body.push((end - start) as u8);
                    for e in &list[start..end] {
                        e.pack_into(&mut body, &e.title);
                    }
                    send_mod_channel(session, uid, "hamp:tele", &body);
                }
            }
        }
        _ => {
            if session.log_packets {
                println!("[GAME:'{}'] [MOD] '{}' → unknown channel '{}' ({} bytes) — dropped",
                    session.room_token, uid, channel, payload.len());
            }
        }
    }
}

/// Repacks a relay host's 0x2F teleporter page into the format the client
/// parser expects. The host's PackPageOfTeleporters (this build) emits
/// [i16 page][u8 has_more] + entries of [u8 1][str×4][i16×4], but the
/// client's 0x2F case reads [i16 page][u8 is_search][u8 has_more] and an
/// extra trailing built_by string per entry. `off` points just past the
/// requester string; `host_name` is used as built_by for every entry.
fn repack_host_tele_page(data: &[u8], mut off: usize, host_name: &str) -> Option<Vec<u8>> {
    if off + 3 > data.len() { return None; }
    let mut pkt = vec![0x2Fu8];
    pkt.extend_from_slice(&data[off..off + 2]); // page
    pkt.push(0);                                // is_search (host pages never are)
    pkt.push(data[off + 2]);                    // has_more
    off += 3;

    let mut count = 0;
    while count < 3 && off < data.len() && data[off] == 1 {
        off += 1;
        let entry_start = off;
        for _ in 0..4 { // name, desc, tele_str, zone
            let (_, next) = unpack_string(data, off);
            if next == off { return None; }
            off = next;
        }
        if off + 8 > data.len() { return None; } // cx, cz, tx, tz
        off += 8;
        pkt.push(1);
        pkt.extend_from_slice(&data[entry_start..off]);
        pkt.extend(pack_string(host_name)); // built_by
        count += 1;
    }
    if count < 3 {
        pkt.push(0);
    }
    Some(pkt)
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

fn handle_client(mut stream: TcpStream, addr: std::net::SocketAddr, session: Arc<Session>, registry: Option<registry_client::RegistryHandle>) {
    use crate::defs::packet::{split_batch_into, FragmentBuffers};

    let mut player_id: Option<String> = None;
    let mut read_buf = [0u8; 65536];
    let mut accum: Vec<u8> = Vec::new();
    // Packets reassembled from fragments and not yet dispatched, plus the
    // per-stream partial buffers (large packets — teleporter screenshots —
    // arrive split across several batches; see split_batch_into).
    let mut pending: std::collections::VecDeque<Vec<u8>> = std::collections::VecDeque::new();
    let mut partial: FragmentBuffers = FragmentBuffers::new();

    println!("[GAME:'{}'] {} connected", session.room_token, addr);

    'outer: loop {
        if session.shutdown.load(Ordering::Relaxed) { break; }
        let n = match stream.read(&mut read_buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        accum.extend_from_slice(&read_buf[..n]);

        // Dispatch reassembled packets; refill from complete wire batches.
        while !pending.is_empty() || !accum.is_empty() {
            let data: Vec<u8> = if let Some(frame) = pending.pop_front() {
                frame
            } else {
                // Handshake probe: single 0x66 byte (no length prefix).
                if accum[0] == 0x66 {
                    let _ = write_payload(&mut stream, 0, &[0x09, 0x01]);
                    accum.remove(0);
                    continue;
                }

                if accum.len() < 2 { break; }
                let total_len = u16::from_le_bytes([accum[0], accum[1]]) as usize;
                if total_len < 3 { accum.clear(); break; } // unframeable garbage
                if accum.len() < total_len { break; } // wait for more data

                let batch: Vec<u8> = accum.drain(..total_len).collect();
                let mut frames = Vec::new();
                split_batch_into(&batch, &mut partial, &mut frames);
                pending.extend(frames);
                continue;
            };

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
                    client_is_mod:  session.is_admin(&uid),
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

                // Relay: give the client MOD_CHECK_GRACE_SECS to declare its
                // mods (0xE0), then verify against the host's mod set. Vanilla
                // clients never send MOD_HELLO, so this timer is what catches
                // them joining a modded world.
                if matches!(session.mode, SessionMode::Relay) {
                    let sess = Arc::clone(&session);
                    let user = uid.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(MOD_CHECK_GRACE_SECS));
                        if !sess.players.lock().unwrap().contains_key(&user) {
                            return; // already gone
                        }
                        if let Some(reason) = relay_mod_mismatch(&sess, &user) {
                            kick_player(&sess, &user, &reason);
                        }
                    });
                }
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
                        let uid_owned  = uid.clone();
                        let opd_owned  = opd;
                        let zone_owned = zone_name.clone();
                        let reg_owned  = registry.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(1));

                            let get_display = |name: &str| -> String {
                                reg_owned.as_ref()
                                    .and_then(|r| r.get_display_name(name))
                                    .unwrap_or_else(|| name.to_string())
                            };

                            // 1. Broadcast this player to everyone in the same zone.
                            let my_display = get_display(&uid_owned);
                            let spawn_pkt = PlayerNearby { username: &uid_owned, display: &my_display, opd: &opd_owned }.to_payload();
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
                                let disp = get_display(name);
                                sess.send_to(&uid_owned, &PlayerNearby { username: name, display: &disp, opd: init }.to_payload());
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
                                // Host expects: [0x0A][requester:str][zone_name:str][type:u8][pos if type 2|3]
                                let mut relay = vec![0x0Au8];
                                relay.extend(pack_string(uid));        // requester first
                                relay.extend(pack_string(&zone_name)); // zone_name second
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
                                let wire = world.get_chunk_wire(&zone_name, x, z);
                                let _ = write_payload(&mut stream, 2, &wire);
                                // Replay teleporter titles for this chunk so the
                                // builder's local copy stays in sync with edits.
                                let teles = world.teleporters.read().unwrap();
                                for t in teles.iter().filter(|t| t.is_listed() && t.zone == zone_name && t.cx == x && t.cz == z) {
                                    let mut pkt = vec![0x33u8];
                                    pkt.extend(pack_string(&t.title));
                                    pkt.extend(pack_string(&t.description));
                                    pkt.extend(pack_string(&t.zone));
                                    pkt.extend_from_slice(&t.cx.to_le_bytes());
                                    pkt.extend_from_slice(&t.cz.to_le_bytes());
                                    pkt.extend_from_slice(&t.tx.to_le_bytes());
                                    pkt.extend_from_slice(&t.tz.to_le_bytes());
                                    let _ = write_payload(&mut stream, 2, &pkt);
                                }
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
            // Host format:  [requester:str][zone_name:str][type:u8][pos if 2|3][ZoneData body][trail]
            // Guest format: [0x0B][flag=1][0][zone_name:str][ZoneData body][type:u8][pos if 2|3]
            // Trail is excluded — ProcessIncomingZoneData doesn't read it.
            0x0B if matches!(session.mode, SessionMode::Relay) => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (requester, off) = unpack_string(data, 10);
                        let (zone_name, mut zoff) = unpack_string(data, off);
                        if zoff < data.len() {
                            let type_flag = data[zoff]; zoff += 1;
                            let pos_end = if (type_flag & 0xFE) == 2 {
                                (zoff + 12).min(data.len())
                            } else { zoff };
                            let pos_bytes = &data[zoff..pos_end];
                            // Find where ZoneData body ends (to drop the trailing zone-trail)
                            let body_end = skip_zone_data_body(data, pos_end)
                                .unwrap_or(data.len());
                            let zone_body = &data[pos_end..body_end.min(data.len())];

                            let mut out = vec![0x0Bu8, 0x01, 0x00]; // flag=1, second=0
                            out.extend(pack_string(&zone_name));
                            out.extend_from_slice(zone_body);
                            out.push(type_flag);
                            out.extend_from_slice(pos_bytes);

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
                    let zone = session.players.lock().unwrap().get(uid)
                        .map(|p| p.zone.lock().unwrap().clone()).unwrap_or_default();
                    session.broadcast_zone(&pkt, &zone, Some(uid.as_str()));
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
                        if matches!(session.mode, SessionMode::Managed(_)) {
                            let released = session.wildlife.lock().unwrap().release_player(uid);
                            for (id, zone) in released {
                                let mut pkt = vec![0x40];
                                pkt.extend(pack_string(&id));
                                session.broadcast_zone(&pkt, &zone, Some(uid));
                            }
                        }
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
                            let disp = registry.as_ref()
                                .and_then(|r| r.get_display_name(uid))
                                .unwrap_or_else(|| uid.to_string());
                            session.broadcast_zone(
                                &PlayerNearby { username: uid, display: &disp, opd: &init },
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
            // C→S: [validator:Str][Item][rot:u8][zone:Str][Short×4 cx,cz,tx,tz][cache_key:Str]
            // S→C: [Item][rot:u8][zone:Str][Short×4][owner:Str][cache_key]
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
                                    let zone_map = chunks.entry(zone_str.clone()).or_default();
                                    let chunk = zone_map.entry((cx, cz)).or_insert_with(|| {
                                        let params = ws.generator.chunk_params(&zone_str, cx as i32, cz as i32);
                                        world_state::Chunk {
                                            x: cx, z: cz, zone: zone_str.clone(),
                                            biome: params.biome, floor_rot: params.floor_rot,
                                            floor_tex: params.floor_tex, floor_model: 0,
                                            mob_a: params.mob_a, mob_b: params.mob_b,
                                            elements: params.elements.into_iter()
                                                .map(|p| ChunkElement { cell_x: p.cell_x, cell_z: p.cell_z, rotation: p.rotation, item_data: p.item_data })
                                                .collect(),
                                            land_claims: HashMap::new(),
                                        }
                                    });
                                    chunk.elements.push(ChunkElement {
                                        cell_x: tx as u8, cell_z: tz as u8,
                                        rotation, item_data: item_bytes.clone(),
                                    });
                                    drop(chunks);
                                    // Register zone for any item that carries a shack_id.
                                    if let Some((shack_id, item_id)) = parse_shack_info(&item_bytes) {
                                        let zone_name = format!("shack{}", shack_id);
                                        let kind = special_generators::zone_kind_from_item_id(&item_id);
                                        ws.zones.write().unwrap().insert(zone_name,
                                            world_state::ZoneEntry::interior(world_state::InteriorData {
                                                item_bytes: item_bytes.clone(), rotation, cx, cz, tx, tz,
                                                outer_zone: zone_str.clone(),
                                                kind,
                                            })
                                        );
                                    }
                                    // Add land claim to 3×3 neighbourhood.
                                    let days = parse_item_id(&item_bytes).and_then(|id| match id.as_str() {
                                        "10-day Land Claim" => Some(10u64),
                                        "30-day Land Claim" => Some(30u64),
                                        "Admin Land Claim"  => Some(99_999u64),
                                        _ => None,
                                    });
                                    if let Some(d) = days {
                                        ws.add_land_claims(&zone_str, cx, cz, tx as i16, tz as i16, uid, d);
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
                                if let Some(chunk) = ws.chunks.write().unwrap().get_mut(&zone_str).and_then(|m| m.get_mut(&(cx, cz))) {
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
                                // Removing a teleporter object drops its listing too.
                                if parse_item_id(&item_bytes).as_deref() == Some("Teleporter") {
                                    ws.remove_teleporter(&zone_str, cx, cz, tx, tz);
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
            // C→S: [validator:Str][new_Item][old_Item][rot:u8][zone:Str][shorts×4][cache_key:Str]
            // S→C: [new_Item][old_Item][rot:u8][zone:Str][shorts×4][owner:Str][cache_key]
            // (confirmed from SendReplaceBuildable: new_item packed first, old_element_item second)
            0x22 => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    if let Some((new_item, next)) = read_inventory_item(data, off) {
                        off = next;
                        if let Some((old_item, next)) = read_inventory_item(data, off) {
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
                                    let cache_key = &data[off..];

                                    // S→C: new_Item + old_Item + rot + zone + shorts×4 + owner + cache_key
                                    let mut pkt = vec![0x22u8];
                                    pkt.extend_from_slice(&new_item);
                                    pkt.extend_from_slice(&old_item);
                                    pkt.push(rotation);
                                    pkt.extend(pack_string(&zone_str));
                                    pkt.extend_from_slice(shorts_bytes);
                                    pkt.extend(pack_string(uid));
                                    pkt.extend_from_slice(cache_key);
                                    session.broadcast(&pkt, Some(uid.as_str()));

                                    // Replace in WorldState for managed mode.
                                    if let SessionMode::Managed(ref ws) = session.mode {
                                        use world_state::ChunkElement;
                                        let mut chunks = ws.chunks.write().unwrap();
                                        let zone_map = chunks.entry(zone_str.clone()).or_default();
                                        let chunk = zone_map.entry((cx, cz)).or_insert_with(|| {
                                            let params = ws.generator.chunk_params(&zone_str, cx as i32, cz as i32);
                                            world_state::Chunk {
                                                x: cx, z: cz, zone: zone_str.clone(),
                                                biome: params.biome, floor_rot: params.floor_rot,
                                                floor_tex: params.floor_tex, floor_model: 0,
                                                mob_a: params.mob_a, mob_b: params.mob_b,
                                                elements: params.elements.into_iter()
                                                    .map(|p| ChunkElement { cell_x: p.cell_x, cell_z: p.cell_z, rotation: p.rotation, item_data: p.item_data })
                                                    .collect(),
                                                land_claims: HashMap::new(),
                                            }
                                        });
                                        let tx8 = tx as u8;
                                        let tz8 = tz as u8;
                                        // Remove old element (exact match on stored item; position fallback)
                                        let pos = chunk.elements.iter().position(|e| {
                                            e.cell_x == tx8 && e.cell_z == tz8
                                                && e.rotation == rotation && e.item_data == old_item
                                        }).or_else(|| chunk.elements.iter().position(|e| {
                                            e.cell_x == tx8 && e.cell_z == tz8
                                        }));
                                        if let Some(i) = pos { chunk.elements.remove(i); }
                                        chunk.elements.push(ChunkElement {
                                            cell_x: tx8, cell_z: tz8,
                                            rotation, item_data: new_item,
                                        });
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
                    if pid == 0x48 {
                        let Some((id, after_id)) = mobs::read_id(data, off) else { continue; };
                        let (_, after_zone) = unpack_string(data, after_id + 4);
                        if after_zone <= after_id + 4 || after_zone + 10 > data.len() { continue; }
                        let respawn = i16::from_le_bytes([data[after_zone + 8], data[after_zone + 9]]);
                        let (_, after_killer) = unpack_string(data, after_zone + 10);
                        if after_killer <= after_zone + 10 || after_killer + 3 > data.len() { continue; }
                        if matches!(session.mode, SessionMode::Managed(_)) {
                            session.wildlife.lock().unwrap().died(&id, respawn.max(0) as u64);
                        }
                        // mob_type is a C→S field; the receiver starts with darksword here.
                        pkt.extend_from_slice(&data[off..after_killer]);
                        pkt.extend_from_slice(&data[after_killer + 1..]);
                    } else {
                        pkt.extend_from_slice(&data[off..]);
                    }
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

            // ── CHANGE_LAND_USER (0x23) ───────────────────────────────────
            // C→S: [validator:Str][zone:Str][i16 cx][i16 cz][i16 ix][i16 iz][u8 user_index][str new_username][str×9 cache_keys]
            // S→C: [0x23][str zone][i16 cx][i16 cz][i16 ix][i16 iz][u8 user_index][str new_username]
            0x23 => {
                if let Some(ref uid) = player_id {
                    let (_, mut off) = unpack_string(data, 10); // skip validator
                    let (zone, next) = unpack_string(data, off); off = next;
                    if off + 9 <= data.len() {
                        let cx = i16::from_le_bytes([data[off], data[off+1]]); off += 2;
                        let cz = i16::from_le_bytes([data[off], data[off+1]]); off += 2;
                        let ix = i16::from_le_bytes([data[off], data[off+1]]); off += 2;
                        let iz = i16::from_le_bytes([data[off], data[off+1]]); off += 2;
                        let user_index = data[off]; off += 1;
                        let (new_username, _) = unpack_string(data, off);

                        if let SessionMode::Managed(ref ws) = session.mode {
                            ws.update_land_claim_user(&zone, cx, cz, ix, iz, user_index, &new_username);
                        }

                        let mut pkt = vec![0x23u8];
                        pkt.extend(pack_string(&zone));
                        pkt.extend_from_slice(&cx.to_le_bytes());
                        pkt.extend_from_slice(&cz.to_le_bytes());
                        pkt.extend_from_slice(&ix.to_le_bytes());
                        pkt.extend_from_slice(&iz.to_le_bytes());
                        pkt.push(user_index);
                        pkt.extend(pack_string(&new_username));
                        session.broadcast(&pkt, None);
                    } else {
                        // Malformed — relay raw with player prefix
                        let mut pkt = vec![0x23u8];
                        pkt.extend(pack_string(uid));
                        pkt.extend_from_slice(&data[10..]);
                        session.broadcast(&pkt, Some(uid.as_str()));
                    }
                }
            }

            // ── Global player-prefixed relay ────────────────────────────
            // Packets where S→C format has [str player_name] and all
            // players need to see it regardless of zone.
            // 0x4A (unknown), 0x56 RESPAWN,
            // 0x57 RETURNING_TO_BREEDER, 0x58 UPDATE_SYNCED_TARGETS,
            // 0x59 CREATED_LOCAL_MOB, 0x5A BANDIT_FLAG_DESTROYED
            0x4A | 0x56 | 0x57 | 0x58 | 0x59 | 0x5A => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![pid];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // ── Teleporter packets ───────────────────────────────────────
            // Identity is the teleporter object's own location:
            // tele_str = "zone,cx,cz,tx,tz" (the client keys its screenshot
            // cache and slot matching on this exact string).
            //
            // Managed mode: the server is the teleporter authority. The
            // host-side serving code left in this client build is wire-broken
            // (PackPageOfTeleporters omits the is_search byte and built_by
            // string that the client's own 0x2F parser requires), so serving
            // from the server is the only path that works at all.
            //
            // Relay mode: requests still go to the host, but both directions
            // are repacked into the format the client actually parses.

            // 0x2E REQ_TELE_PAGE
            // C→S mode 0: [u8 0][u8 in_search][i16 page]
            // C→S mode 1: [u8 1][str zone][i16 cx][i16 cz][i16 tx][i16 tz]
            //             ("what page is the teleporter at this location on")
            0x2E => {
                if let Some(ref uid) = player_id {
                    match session.mode {
                        SessionMode::Managed(ref ws) => {
                            if data.len() >= 14 && data[10] == 0 {
                                let in_search = data[11] == 1;
                                let page = i16::from_le_bytes([data[12], data[13]]);
                                send_tele_page(&session, ws, uid, page, in_search);
                            } else if data.len() > 11 && data[10] == 1 {
                                let (zone, off) = unpack_string(data, 11);
                                if data.len() >= off + 8 && !zone.is_empty() {
                                    let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                                    let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                                    let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                                    let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                                    let key = format!("{},{},{},{},{}", zone, cx, cz, tx, tz);
                                    let page = tele_listing(&session, ws, "")
                                        .iter()
                                        .position(|e| e.tele_str == key)
                                        .map(|i| (i / 3) as i16)
                                        .unwrap_or(0);
                                    send_tele_page(&session, ws, uid, page, false);
                                }
                            }
                        }
                        SessionMode::Relay => {
                            // Host parses [requester][mode][...] with NO
                            // in_search byte — strip it from mode-0 requests
                            // or the host reads garbage for the page number.
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                if hname != uid && data.len() > 10 {
                                    let mut relay = vec![0x2Eu8];
                                    relay.extend(pack_string(uid));
                                    if data[10] == 0 && data.len() >= 14 {
                                        relay.push(0);
                                        relay.extend_from_slice(&data[12..14]);
                                    } else {
                                        relay.extend_from_slice(&data[10..]);
                                    }
                                    session.send_to(hname, &relay);
                                }
                            }
                        }
                    }
                }
            }

            // 0x2F TELE_PAGE_DATA — host's page reply (relay mode only).
            // Host packs: [str requester][i16 page][u8 has_more]
            //             entries [u8 1][str×4][i16×4], [u8 0] terminator.
            // Repack into the parser's format while routing to the requester
            // (see repack_host_tele_page).
            0x2F => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (target_user, off) = unpack_string(data, 10);
                        if !target_user.is_empty() {
                            if let Some(pkt) = repack_host_tele_page(data, off, uid) {
                                session.send_to(&target_user, &pkt);
                            }
                        }
                    }
                }
            }

            // 0x30 TELE_SCREENSHOT upload
            // C→S: [str zone][i16 cx][i16 cz][i16 tx][i16 tz][i32 len][bytes]
            // Managed: store so the server can answer 0x31 itself. Both
            // modes: broadcast unchanged (S→C 0x30 has the same layout) so
            // clients tracking the teleporter locally refresh their copy.
            0x30 => {
                if let Some(ref uid) = player_id {
                    if let SessionMode::Managed(ref ws) = session.mode {
                        let (zone, off) = unpack_string(data, 10);
                        if data.len() >= off + 12 && !zone.is_empty() {
                            let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                            let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                            let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                            let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                            let len = i32::from_le_bytes([data[off+8], data[off+9], data[off+10], data[off+11]]).max(0) as usize;
                            let start = off + 12;
                            if data.len() >= start + len {
                                ws.set_teleporter_screenshot(&zone, cx, cz, tx, tz,
                                    data[start..start + len].to_vec(), uid);
                            }
                        }
                    }
                    let mut pkt = vec![0x30u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // 0x31 REQ_TELE_SCREENSHOT
            // C→S: [str zone][i16 cx][i16 cz][i16 tx][i16 tz]
            // Managed: answer with S→C 0x32 [str tele_str][i32 len][bytes].
            // No reply if we have no screenshot — the client keeps its
            // placeholder sprite.
            0x31 => {
                if let Some(ref uid) = player_id {
                    match session.mode {
                        SessionMode::Managed(ref ws) => {
                            let (zone, off) = unpack_string(data, 10);
                            if data.len() >= off + 8 && !zone.is_empty() {
                                let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                                let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                                let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                                let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                                let teles = ws.teleporters.read().unwrap();
                                if let Some(t) = teles.iter().find(|t| t.is_at(&zone, cx, cz, tx, tz)) {
                                    if !t.screenshot.is_empty() {
                                        let mut pkt = vec![0x32u8];
                                        pkt.extend(pack_string(&t.tele_str()));
                                        pkt.extend_from_slice(&(t.screenshot.len() as i32).to_le_bytes());
                                        pkt.extend_from_slice(&t.screenshot);
                                        session.send_to(uid, &pkt);
                                    }
                                }
                            }
                        }
                        SessionMode::Relay => {
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                if hname != uid {
                                    let mut relay = vec![0x31u8];
                                    relay.extend(pack_string(uid));
                                    relay.extend_from_slice(&data[10..]);
                                    session.send_to(hname, &relay);
                                }
                            }
                        }
                    }
                }
            }

            // 0x32 TELE_SCREENSHOT_RESPONSE — host's screenshot reply (relay).
            // Host packs: [str requester][str zone][i16×4][i32 len][bytes];
            // the client parses [str tele_str][i32 len][bytes], so rebuild
            // the location fields into the tele_str key while routing.
            0x32 => {
                if let Some(ref uid) = player_id {
                    let is_host = session.host.lock().unwrap()
                        .as_ref().map(|h| h == uid).unwrap_or(false);
                    if is_host {
                        let (target_user, off) = unpack_string(data, 10);
                        let (zone, coords) = unpack_string(data, off);
                        if !target_user.is_empty() && data.len() >= coords + 8 {
                            let cx = i16::from_le_bytes([data[coords],   data[coords+1]]);
                            let cz = i16::from_le_bytes([data[coords+2], data[coords+3]]);
                            let tx = i16::from_le_bytes([data[coords+4], data[coords+5]]);
                            let tz = i16::from_le_bytes([data[coords+6], data[coords+7]]);
                            let mut pkt = vec![0x32u8];
                            pkt.extend(pack_string(&format!("{},{},{},{},{}", zone, cx, cz, tx, tz)));
                            pkt.extend_from_slice(&data[coords + 8..]); // [i32 len][bytes]
                            session.send_to(&target_user, &pkt);
                        }
                    }
                }
            }

            // 0x33 FINISHED_EDITING_TELE — broadcast to all + upsert registry
            // C→S: [title:str][description:str][zone:str][cx:i16][cz:i16][tx:i16][tz:i16]
            // (confirmed from SendFinishedEditingTeleporter decompilation)
            0x33 => {
                if let Some(ref uid) = player_id {
                    let mut pkt = vec![0x33u8];
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, None);

                    if let SessionMode::Managed(ref ws) = session.mode {
                        let mut off = 10;
                        let (title, next) = unpack_string(data, off); off = next;
                        let (desc,  next) = unpack_string(data, off); off = next;
                        let (zone,  next) = unpack_string(data, off); off = next;
                        if off + 8 <= data.len() && !zone.is_empty() {
                            let cx = i16::from_le_bytes([data[off],   data[off+1]]);
                            let cz = i16::from_le_bytes([data[off+2], data[off+3]]);
                            let tx = i16::from_le_bytes([data[off+4], data[off+5]]);
                            let tz = i16::from_le_bytes([data[off+6], data[off+7]]);
                            ws.upsert_teleporter(&zone, cx, cz, tx, tz, &title, &desc, uid);
                        }
                    }
                }
            }

            // 0x34 NEW_TELE_SEARCH — C→S: [str term]
            // Managed: remember the term and reply immediately with page 0 of
            // matching results (0x2F with is_search=1); the client pages
            // further via 0x2E with in_search=1. Mirrors the original
            // server's tele_search_NEW behaviour.
            0x34 => {
                if let Some(ref uid) = player_id {
                    match session.mode {
                        SessionMode::Managed(ref ws) => {
                            let (term, _) = unpack_string(data, 10);
                            session.tele_search.lock().unwrap().insert(uid.clone(), term);
                            send_tele_page(&session, ws, uid, 0, true);
                        }
                        SessionMode::Relay => {
                            // This client build has no host-side 0x34 handler;
                            // relay anyway in case an older host can serve it.
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                if hname != uid {
                                    let mut relay = vec![0x34u8];
                                    relay.extend(pack_string(uid));
                                    relay.extend_from_slice(&data[10..]);
                                    session.send_to(hname, &relay);
                                }
                            }
                        }
                    }
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
            // C→S: [u8 count][Str combat_id × count]. Managed sessions reply
            // with [u8 count][Str combat_id, u8 succeeded × count].
            0x3F => {
                if let Some(ref uid) = player_id {
                    if matches!(session.mode, SessionMode::Managed(_)) {
                        let zone = session.players.lock().unwrap().get(uid)
                            .map(|p| p.zone.lock().unwrap().clone()).unwrap_or_default();
                        let reply = session.wildlife.lock().unwrap().claim(uid, &zone, &data[10..]);
                        if let Some(reply) = reply { session.send_to(uid, &reply); }
                    }
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
                    if matches!(session.mode, SessionMode::Managed(_)) {
                        if let Some((id, _)) = mobs::read_id(data, 10) {
                            let zone = session.wildlife.lock().unwrap().release(uid, &id);
                            if let Some(zone) = zone {
                                let mut pkt = vec![0x40];
                                pkt.extend(pack_string(&id));
                                session.broadcast_zone(&pkt, &zone, Some(uid));
                            }
                        }
                        continue;
                    }
                    let mut pkt = vec![0x40u8];
                    pkt.extend(pack_string(uid));
                    pkt.extend_from_slice(&data[10..]);
                    session.broadcast(&pkt, Some(uid.as_str()));
                }
            }

            // The first client still loading an unowned mob takes over its AI.
            0x44 => {
                if let Some(ref uid) = player_id {
                    if matches!(session.mode, SessionMode::Managed(_)) {
                        let zone = session.players.lock().unwrap().get(uid)
                            .map(|p| p.zone.lock().unwrap().clone()).unwrap_or_default();
                        if let Some((id, _)) = mobs::read_id(data, 10) {
                            let inherited = session.wildlife.lock().unwrap().inherit(uid, &zone, &id);
                            if inherited {
                                let mut pkt = vec![0x45];
                                pkt.extend(pack_string(&id));
                                session.send_to(uid, &pkt);
                            }
                        }
                    } else if let Some(host) = session.host.lock().unwrap().clone() {
                        let mut pkt = vec![0x44];
                        pkt.extend(pack_string(uid));
                        pkt.extend_from_slice(&data[10..]);
                        session.send_to(&host, &pkt);
                    }
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
            // 0x27 C→S: [validator:Str][str claim_key]  ("zone,cx,cz,ix,iz")
            //   In managed mode: re-send the chunk so the client has fresh land-claim data.
            // 0x28: empty — relay to host
            0x27 | 0x28 => {
                if let Some(ref uid) = player_id {
                    if pid == 0x27 {
                        if let SessionMode::Managed(ref ws) = session.mode {
                            let (_, off) = unpack_string(data, 10); // skip validator
                            let (claim_key, _) = unpack_string(data, off);
                            // claim_key format: "zone,cx,cz,ix,iz"
                            let parts: Vec<&str> = claim_key.splitn(5, ',').collect();
                            if parts.len() == 5 {
                                if let (Ok(cx), Ok(cz)) = (parts[1].parse::<i16>(), parts[2].parse::<i16>()) {
                                    let wire = ws.get_chunk_wire(parts[0], cx, cz);
                                    let _ = write_payload(&mut stream, 2, &wire);
                                }
                            }
                        } else if matches!(session.mode, SessionMode::Relay) {
                            let host = session.host.lock().unwrap().clone();
                            if let Some(ref hname) = host {
                                let mut relay = vec![pid];
                                relay.extend(pack_string(uid));
                                relay.extend_from_slice(&data[10..]);
                                session.send_to(hname, &relay);
                            }
                        }
                    } else if matches!(session.mode, SessionMode::Relay) {
                        // 0x28 relay to host
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

            // ── MOD_HELLO (0xE0) — HAModHelper handshake ─────────────────
            // C→S: [i16 protocol][str helper_version][i16 n_mods][n × str mod_id]
            // S→C reply MOD_WELCOME (same opcode):
            //      [i16 protocol][str server_ident][i16 n_channels][n × str channel]
            // Must be sent after login; marks the player as modded so the
            // server will send/accept MOD_CHANNEL traffic for them.
            0xE0 => {
                if let Some(ref uid) = player_id {
                    if data.len() >= 12 {
                        let protocol = i16::from_le_bytes([data[10], data[11]]);
                        let (helper_version, mut off) = unpack_string(data, 12);
                        let mut mods = Vec::new();
                        if off + 2 <= data.len() {
                            let n = i16::from_le_bytes([data[off], data[off+1]]).max(0);
                            off += 2;
                            for _ in 0..n {
                                let (m, next) = unpack_string(data, off);
                                if next == off { break; }
                                off = next;
                                mods.push(m);
                            }
                        }
                        println!("[GAME:'{}'] '{}' is modded: HAModHelper {} (proto {}, {} mod(s))",
                            session.room_token, uid, helper_version, protocol, mods.len());

                        // Normal-server gate: reject debug builds of HAModHelper
                        // (helper_version is "debug:..."; legacy clients with no
                        // prefix count as release). Register first so the
                        // hamp:core kick notice reaches them, then kick — the
                        // disconnect cleanup removes the entry again.
                        if !session.allow_debug && helper_version.starts_with("debug:") {
                            session.mod_clients.lock().unwrap()
                                .insert(uid.clone(), ModClientInfo { protocol, helper_version, mods });
                            kick_player(&session, uid,
                                "This is a Normal server — debug builds of HAModHelper aren't allowed here. \
                                 Use a Release build, or join an Anarchy server.");
                            // Break to the disconnect cleanup (removes the player,
                            // decrements the count). `return` would skip it.
                            break 'outer;
                        }

                        session.mod_clients.lock().unwrap()
                            .insert(uid.clone(), ModClientInfo { protocol, helper_version, mods });

                        let mut pkt = vec![0xE0u8];
                        pkt.extend_from_slice(&MOD_PROTOCOL_VERSION.to_le_bytes());
                        pkt.extend(pack_string(concat!("HAMP ", env!("CARGO_PKG_VERSION"))));
                        pkt.extend_from_slice(&(MOD_CHANNELS.len() as i16).to_le_bytes());
                        for ch in MOD_CHANNELS {
                            pkt.extend(pack_string(ch));
                        }
                        session.send_to(uid, &pkt);

                        // Relay mode: enforce mod-set matching against the host.
                        if matches!(session.mode, SessionMode::Relay) {
                            let is_host = session.host.lock().unwrap().as_deref() == Some(uid.as_str());
                            if is_host {
                                // Host declared (possibly late) — re-validate
                                // every guest already in the session.
                                let guests: Vec<String> = session.players.lock().unwrap().keys()
                                    .filter(|n| n.as_str() != uid.as_str())
                                    .cloned()
                                    .collect();
                                for g in guests {
                                    if let Some(reason) = relay_mod_mismatch(&session, &g) {
                                        kick_player(&session, &g, &reason);
                                    }
                                }
                            } else if let Some(reason) = relay_mod_mismatch(&session, uid) {
                                kick_player(&session, uid, &reason);
                            }
                        }
                    }
                }
            }

            // ── MOD_CHANNEL (0xE1) — namespaced mod message ──────────────
            // C→S / S→C: [str channel][payload...]   (channel = "namespace:name")
            // Only accepted from players that completed MOD_HELLO; never
            // relayed or broadcast, so stock clients see none of this.
            0xE1 => {
                if let Some(ref uid) = player_id {
                    let known = session.mod_clients.lock().unwrap().contains_key(uid.as_str());
                    if known {
                        let (channel, off) = unpack_string(data, 10);
                        if !channel.is_empty() {
                            handle_mod_channel(&session, uid, &channel, &data[off.min(data.len())..]);
                        }
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
        if matches!(session.mode, SessionMode::Managed(_)) {
            let released = session.wildlife.lock().unwrap().release_player(uid);
            for (id, zone) in released {
                let mut pkt = vec![0x40];
                pkt.extend(pack_string(&id));
                session.broadcast_zone(&pkt, &zone, None);
            }
        }
        session.open_baskets.lock().unwrap().retain(|_, (holder, _)| holder != uid);
        session.mod_clients.lock().unwrap().remove(uid.as_str());
        session.tele_search.lock().unwrap().remove(uid.as_str());
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
            let chunk_count: usize = ws.chunks.read().unwrap().values().map(|m| m.len()).sum();
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
            let weights = BiomeWeights {
                grass:     cfg.biome_grass_commonness,
                snow:      cfg.biome_snow_commonness,
                desert:    cfg.biome_desert_commonness,
                evergreen: cfg.biome_evergreen_commonness,
                ocean:     cfg.biome_ocean_commonness,
                swamp:     cfg.biome_swamp_commonness,
                woodlands: cfg.biome_woodlands_commonness,
                sakura:    cfg.biome_sakura_commonness,
            };
            let mut tmpl = WorldTemplate::new(
                seed,
                vec![ZoneConfig::new("overworld", weights)],
            );
            tmpl.start_biome        = parse_start_biome(&cfg.start_biome).unwrap_or(-1);
            tmpl.start_biome_radius = cfg.start_biome_radius.max(0);
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
    let session = Session::new(&cfg.server_name, SessionMode::Managed(Arc::clone(&world)), cfg.pvp_enabled, cfg.log_packets, cfg.admin_users.clone(), cfg.allow_debug);

    let mut registry_handle: Option<registry_client::RegistryHandle> = None;

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
        registry_handle = registry_client::spawn(
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
                let reg  = registry_handle.clone();
                std::thread::spawn(move || handle_client(stream, peer, sess, reg));
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
            let session = Session::new(room_token.clone(), SessionMode::Relay, false, true, Vec::new(), cfg.allow_debug);
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
                            std::thread::spawn(move || handle_client(stream, peer, sess, None));
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


#[cfg(test)]
mod tele_tests {
    use super::*;

    /// Builds a fake C→S 0x2F frame as the relay host's PackPageOfTeleporters
    /// emits it: 10-byte frame header, requester string, [i16 page][u8 has_more],
    /// then entries of [u8 1][str×4][i16×4] and a [u8 0] terminator.
    fn host_page_frame(entries: usize) -> (Vec<u8>, usize) {
        let mut data = vec![0u8; 10];
        data.extend(pack_string("guest"));
        let off = data.len();
        data.extend_from_slice(&2i16.to_le_bytes()); // page
        data.push(1);                                // has_more
        for i in 0..entries {
            data.push(1);
            data.extend(pack_string(&format!("Tele {i}")));
            data.extend(pack_string("desc"));
            data.extend(pack_string("overworld,1,2,3,4"));
            data.extend(pack_string("overworld"));
            for v in [1i16, 2, 3, 4] {
                data.extend_from_slice(&v.to_le_bytes());
            }
        }
        if entries < 3 {
            data.push(0);
        }
        (data, off)
    }

    #[test]
    fn repack_inserts_is_search_and_built_by() {
        let (data, off) = host_page_frame(1);
        let pkt = repack_host_tele_page(&data, off, "hostname").unwrap();

        assert_eq!(pkt[0], 0x2F);
        assert_eq!(i16::from_le_bytes([pkt[1], pkt[2]]), 2); // page preserved
        assert_eq!(pkt[3], 0);                               // is_search inserted
        assert_eq!(pkt[4], 1);                               // has_more preserved
        assert_eq!(pkt[5], 1);                               // entry flag

        // built_by sits between the entry body and the terminator.
        assert_eq!(*pkt.last().unwrap(), 0);
        let bb = pack_string("hostname");
        let bb_start = pkt.len() - 1 - bb.len();
        assert_eq!(&pkt[bb_start..pkt.len() - 1], &bb[..]);
    }

    #[test]
    fn repack_full_page_has_no_terminator() {
        let (data, off) = host_page_frame(3);
        let pkt = repack_host_tele_page(&data, off, "h").unwrap();
        // Three entries, each ending in built_by; no trailing 0 after the third.
        let bb = pack_string("h");
        assert_eq!(&pkt[pkt.len() - bb.len()..], &bb[..]);
    }

    #[test]
    fn repack_rejects_truncated_entry() {
        let (mut data, off) = host_page_frame(1);
        data.truncate(data.len() - 6); // cut into the coord block
        assert!(repack_host_tele_page(&data, off, "h").is_none());
    }
}
