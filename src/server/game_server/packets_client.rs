#![allow(dead_code)]
// packets_client.rs — C→S game-server packet definitions.
//
// `GameClientPacket` covers every packet a client may send to the game server.
// `parse()` is the single entry point: reads the 9-byte batch envelope, then
// dispatches per-packet field extraction.
//
// Packets that require server-side field inspection have named variants.
// Everything else — packets the server routes or relays with minimal parsing —
// is captured as `Relay { id, payload }`.
//
// Wire helpers `skip_inventory_item` and `skip_basket_contents` (used for
// opaque blobs inside some packets) live here rather than in mod.rs so that
// `parse()` can use them without a cross-module dependency.
//
// Adding a new C→S packet
// ────────────────────────
// 1. Add a named variant with the fields the handler needs.
// 2. Add the parse arm in `parse()`.
// 3. Match the new variant in `handle_client` in mod.rs.

use crate::defs::packet::{unpack_string, PacketHeader};
use std::io::Cursor;
use binrw::BinRead;

// ── Wire blob helpers ──────────────────────────────────────────────────────

/// Skips past one `InventoryItem::UnpackFromWeb` blob.
///
/// Wire layout:
///   i16(short_prop_count) + [Str(key) + i16(val)] × count
///   i16(string_prop_count) + [Str(key) + Str(val)] × count
///   i16(long_prop_count)  + [Str(key) + i32(val)] × count  (despite the name, values are 32-bit)
pub fn skip_inventory_item(data: &[u8], mut off: usize) -> usize {
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o;
        off += 2;
    }
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o;
        let (_, o) = unpack_string(data, off); off = o;
    }
    if off + 2 > data.len() { return off; }
    let count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..count {
        let (_, o) = unpack_string(data, off); off = o;
        off += 4;
    }
    off
}

/// Skips past a `BasketContents` blob.
///
/// Wire layout: i16(slot_count) + [i16(index) + i16(quantity) + InventoryItem] × count
pub fn skip_basket_contents(data: &[u8], mut off: usize) -> usize {
    if off + 2 > data.len() { return off; }
    let slot_count = u16::from_le_bytes([data[off], data[off + 1]]) as usize; off += 2;
    for _ in 0..slot_count {
        off += 4; // index + quantity
        off = skip_inventory_item(data, off);
    }
    off
}

// ── GameClientPacket ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum GameClientPacket {
    // ── No-payload ─────────────────────────────────────────────────────────
    /// 0x01 — latency ping.
    Ping,
    /// 0x0F — keepalive heartbeat.
    Heartbeat,
    /// 0x29 — request a block of unique object IDs.
    ReqUniqueIds,

    // ── Parsed: server acts on the extracted fields ─────────────────────────
    /// 0x26 — initial login.
    /// C→S: [Str(world_name)][Str(token)]
    Login {
        world: String,
        token: String,
    },

    /// 0x03 — initial player data (OnlinePlayerData source).
    /// C→S: [Pos(8 bytes)][Str(zone)][u8(body_slot)][level+items+hp+creatures+...]
    PlayerData {
        pos_bytes: [u8; 8],
        zone:      String,
        body_slot: u8,
        rest:      Vec<u8>,
    },

    /// 0x0A — request zone data.
    /// C→S: [Str(zone_name)][u8(type)][optional Pos if type==2|3]
    ReqZoneData {
        zone_name: String,
        zone_type: u8,
        extra:     Vec<u8>,
    },

    /// 0x0C — request a chunk.
    /// C→S: [Str(zone)][i16(x)][i16(z)][u8(dimension)][Str(sub_zone)]
    ReqChunk {
        zone_name: String,
        x:         i16,
        z:         i16,
        dimension: u8,
        sub_zone:  String,
    },

    /// 0x06 — chat message.
    /// C→S: [Str(message)]
    Chat {
        message: String,
    },

    /// 0x14 — zone change.
    /// C→S: [Str(zone_name)]
    ZoneChange {
        zone_name: String,
    },

    /// 0x16 — end teleport; carries new position.
    /// C→S: [Pos(8 bytes)]
    EndTeleport {
        pos_bytes: Vec<u8>,
    },

    /// 0x2A — sync complete; echo to sender, relay to others.
    /// Carries the raw body (from byte 9, including the 0x2A id byte).
    SyncComplete {
        body: Vec<u8>,
    },

    /// 0x1A — open/request a container.
    /// C→S: [Str(validator)][i64(basket_id)][u8(type)][Str(chunk)][i16×4]
    ContainerOpen {
        basket_id: i64,
    },

    /// 0x1E — close basket with updated contents.
    /// C→S: [Str(validator)][u32(basket_id)][BasketContents][Str(item_name)][trailing...]
    /// `basket_payload` = basket_id + BasketContents + item_name (trailing stripped).
    BasketUpdate {
        basket_payload: Vec<u8>,
    },

    /// 0x2D — ask to join a player.
    /// C→S: [Str(target)][rest...]
    AskJoin {
        target: String,
        rest:   Vec<u8>,
    },

    /// 0x2B — grant a join (game-server side: route to target).
    /// C→S: [Str(target)][rest...]
    YouMayJoin {
        target: String,
        rest:   Vec<u8>,
    },

    /// 0x35 — challenge another player to a minigame.
    /// C→S: [Str(target)][u8(type)]
    MinigameChallenge {
        target: String,
        rest:   Vec<u8>,
    },

    /// 0x36 — respond to a minigame challenge.
    /// C→S: [u8(response)][Str(challenger)][u8(type)]
    MinigameResponse {
        response:   u8,
        challenger: String,
        rest:       Vec<u8>,
    },

    /// 0x37 — begin a minigame.
    /// C→S: [Str(owner)][rest...]
    BeginMinigame {
        owner: String,
        rest:  Vec<u8>,
    },

    /// 0x42 — request mob data from an owner.
    /// C→S: [Str(target_owner)][rest...]
    MobDataReq {
        target_owner: String,
        rest:         Vec<u8>,
    },

    /// 0x43 — send mob data to a requester.
    /// C→S: [Str(target_requester)][rest...]
    MobDataResp {
        target_requester: String,
        rest:             Vec<u8>,
    },

    // ── Host-only responses (relay mode) ───────────────────────────────────
    /// 0x0B (relay host) — zone data response.
    /// Host C→S: [Str(requester)][Str(zone_name)][u8(type_flag)][zone_data...]
    HostZoneResponse {
        requester: String,
        zone_name: String,
        type_flag: u8,
        zone_data: Vec<u8>,
    },

    /// 0x0D (relay host) — chunk data response.
    /// Host C→S: [Str(requester)][PackForWeb body + bandit data]
    HostChunkResponse {
        requester:  String,
        chunk_data: Vec<u8>,
    },

    /// 0x1B (relay host) — container contents response.
    /// Host C→S: [Str(requester)][i64(basket_id)][BasketContents]
    HostContainerResponse {
        requester: String,
        basket_id: i64,
        body:      Vec<u8>,
    },

    // ── Relay passthrough ────────────────────────────────────────────────────
    /// All other packets: server routes/transforms the payload without
    /// inspecting individual fields. `id` drives routing decisions in the handler.
    Relay {
        id:      u8,
        payload: Vec<u8>,
    },
}

impl GameClientPacket {
    /// Parse raw socket bytes (a complete framed packet) into a typed packet.
    ///
    /// Returns `None` if the buffer is shorter than 10 bytes (no room for the
    /// header + ID byte). Never returns `None` for unknown IDs — they become
    /// `Relay`.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 10 { return None; }

        let mut cur = Cursor::new(data);
        let hdr = PacketHeader::read(&mut cur).ok()?;
        let id  = hdr.packet_id;

        match id {
            // ── 0x01 PING ─────────────────────────────────────────────────
            0x01 => Some(Self::Ping),

            // ── 0x0F HEARTBEAT ───────────────────────────────────────────
            0x0F => Some(Self::Heartbeat),

            // ── 0x29 REQ_UNIQUE_IDS ──────────────────────────────────────
            0x29 => Some(Self::ReqUniqueIds),

            // ── 0x26 LOGIN ───────────────────────────────────────────────
            // C→S: [Str(world_name)][Str(token)]
            0x26 => {
                let (world, off) = unpack_string(data, 10);
                let (token, _)   = unpack_string(data, off);
                Some(Self::Login { world, token })
            }

            // ── 0x03 PLAYER_DATA ─────────────────────────────────────────
            // C→S: [Pos(8)][Str(zone)][u8(body_slot)][rest...]
            0x03 => {
                let raw = &data[10..];
                if raw.len() < 8 { return Some(Self::Relay { id, payload: raw.to_vec() }); }
                let mut pos_bytes = [0u8; 8];
                pos_bytes.copy_from_slice(&raw[..8]);
                let (zone, off) = unpack_string(raw, 8);
                let body_slot   = if off < raw.len() { raw[off] } else { 0 };
                let rest        = if off + 1 < raw.len() { raw[off + 1..].to_vec() } else { vec![] };
                Some(Self::PlayerData { pos_bytes, zone, body_slot, rest })
            }

            // ── 0x0A REQ_ZONE_DATA ───────────────────────────────────────
            // C→S: [Str(zone_name)][u8(type)][optional Pos]
            0x0A => {
                let (zone_name, off) = unpack_string(data, 10);
                let zone_type = if data.len() > off { data[off] } else { 0 };
                let extra = if data.len() > off + 1 { data[off + 1..].to_vec() } else { vec![] };
                Some(Self::ReqZoneData { zone_name, zone_type, extra })
            }

            // ── 0x0C REQ_CHUNK ───────────────────────────────────────────
            // C→S: [Str(zone)][i16(x)][i16(z)][u8(dimension)][Str(sub_zone)]
            0x0C => {
                let (zone_name, off) = unpack_string(data, 10);
                if data.len() < off + 4 {
                    return Some(Self::Relay { id, payload: data[10..].to_vec() });
                }
                let x = i16::from_le_bytes([data[off], data[off + 1]]);
                let z = i16::from_le_bytes([data[off + 2], data[off + 3]]);
                let dimension = if data.len() > off + 4 { data[off + 4] } else { 0 };
                let (sub_zone, _) = if data.len() > off + 5 {
                    unpack_string(data, off + 5)
                } else {
                    (String::new(), off + 5)
                };
                Some(Self::ReqChunk { zone_name, x, z, dimension, sub_zone })
            }

            // ── 0x06 CHAT ────────────────────────────────────────────────
            // C→S: [Str(message)]
            0x06 => {
                let (message, _) = unpack_string(data, 10);
                Some(Self::Chat { message })
            }

            // ── 0x14 ZONE_CHANGE ─────────────────────────────────────────
            // C→S: [Str(zone_name)]
            0x14 => {
                let (zone_name, _) = unpack_string(data, 10);
                Some(Self::ZoneChange { zone_name })
            }

            // ── 0x16 END_TELEPORT ────────────────────────────────────────
            // C→S: [Pos(8 bytes raw)]
            0x16 => {
                Some(Self::EndTeleport { pos_bytes: data[10..].to_vec() })
            }

            // ── 0x2A SYNC_COMPLETE ───────────────────────────────────────
            // Body includes the id byte (from byte 9).
            0x2A => {
                Some(Self::SyncComplete { body: data[9..].to_vec() })
            }

            // ── 0x1A CONTAINER_OPEN ──────────────────────────────────────
            // C→S: [Str(validator)][i64(basket_id)][...]
            0x1A => {
                let (_, off) = unpack_string(data, 10);
                if data.len() < off + 8 {
                    return Some(Self::Relay { id, payload: data[10..].to_vec() });
                }
                let basket_id = i64::from_le_bytes([
                    data[off], data[off+1], data[off+2], data[off+3],
                    data[off+4], data[off+5], data[off+6], data[off+7],
                ]);
                Some(Self::ContainerOpen { basket_id })
            }

            // ── 0x1E BASKET_UPDATE ───────────────────────────────────────
            // C→S: [Str(validator)][i64(basket_id)][BasketContents][Str(item)][trailing...]
            0x1E => {
                let (_, off) = unpack_string(data, 10);
                let basket_start     = off;
                let after_basket_id  = off + 8;
                let after_contents   = skip_basket_contents(data, after_basket_id);
                let (_, after_item)  = unpack_string(data, after_contents);
                let basket_payload   = data[basket_start..after_item.min(data.len())].to_vec();
                Some(Self::BasketUpdate { basket_payload })
            }

            // ── 0x2D ASK_JOIN ────────────────────────────────────────────
            0x2D => {
                let (target, off) = unpack_string(data, 10);
                Some(Self::AskJoin { target, rest: data[off..].to_vec() })
            }

            // ── 0x2B YOU_MAY_JOIN ────────────────────────────────────────
            0x2B => {
                let (target, off) = unpack_string(data, 10);
                Some(Self::YouMayJoin { target, rest: data[off..].to_vec() })
            }

            // ── 0x35 MINIGAME_CHALLENGE ──────────────────────────────────
            0x35 => {
                let (target, off) = unpack_string(data, 10);
                Some(Self::MinigameChallenge { target, rest: data[off..].to_vec() })
            }

            // ── 0x36 MINIGAME_RESPONSE ───────────────────────────────────
            // C→S: [u8(response)][Str(challenger)][u8(type)]
            0x36 => {
                if data.len() <= 10 {
                    return Some(Self::Relay { id, payload: vec![] });
                }
                let response = data[10];
                let (challenger, off) = unpack_string(data, 11);
                Some(Self::MinigameResponse {
                    response,
                    challenger,
                    rest: data[off..].to_vec(),
                })
            }

            // ── 0x37 BEGIN_MINIGAME ──────────────────────────────────────
            0x37 => {
                let (owner, off) = unpack_string(data, 10);
                Some(Self::BeginMinigame { owner, rest: data[off..].to_vec() })
            }

            // ── 0x42 MOB_DATA_REQ ────────────────────────────────────────
            0x42 => {
                let (target_owner, off) = unpack_string(data, 10);
                Some(Self::MobDataReq { target_owner, rest: data[off..].to_vec() })
            }

            // ── 0x43 MOB_DATA_RESP ───────────────────────────────────────
            0x43 => {
                let (target_requester, off) = unpack_string(data, 10);
                Some(Self::MobDataResp { target_requester, rest: data[off..].to_vec() })
            }

            // ── 0x0B HOST_ZONE_RESPONSE (relay mode) ─────────────────────
            // Host C→S: [Str(requester)][Str(zone_name)][u8(type_flag)][zone_data...]
            0x0B => {
                let (requester, off)  = unpack_string(data, 10);
                let (zone_name, zoff) = unpack_string(data, off);
                let type_flag = if data.len() > zoff { data[zoff] } else { 0 };
                let zone_data = if data.len() > zoff + 1 { data[zoff + 1..].to_vec() } else { vec![] };
                Some(Self::HostZoneResponse { requester, zone_name, type_flag, zone_data })
            }

            // ── 0x0D HOST_CHUNK_RESPONSE (relay mode) ────────────────────
            // Host C→S: [Str(requester)][PackForWeb body]
            0x0D => {
                let (requester, off) = unpack_string(data, 10);
                Some(Self::HostChunkResponse {
                    requester,
                    chunk_data: data[off..].to_vec(),
                })
            }

            // ── 0x1B HOST_CONTAINER_RESPONSE (relay mode) ────────────────
            // Host C→S: [Str(requester)][i64(basket_id)][BasketContents]
            0x1B => {
                let (requester, off) = unpack_string(data, 10);
                if data.len() < off + 8 {
                    return Some(Self::Relay { id, payload: data[10..].to_vec() });
                }
                let basket_id = i64::from_le_bytes([
                    data[off], data[off+1], data[off+2], data[off+3],
                    data[off+4], data[off+5], data[off+6], data[off+7],
                ]);
                Some(Self::HostContainerResponse {
                    requester,
                    basket_id,
                    body: data[off..].to_vec(), // includes basket_id bytes
                })
            }

            // ── Everything else: relay ────────────────────────────────────
            _ => Some(Self::Relay { id, payload: data[10..].to_vec() }),
        }
    }
}
