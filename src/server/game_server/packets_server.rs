#![allow(dead_code)]
// packets_server.rs — S→C game-server packet definitions.
//
// Every type here implements `ServerPacket` from `defs::packet`, which
// provides `to_payload() -> Vec<u8>`.
//
// Two categories:
//  - Structured packets: built from typed fields, using manual serialisation
//    (most game packets mix strings and raw blobs in ways that don't map
//    cleanly to #[binwrite] attribute macros).
//  - `RawPacket`: re-exported from `defs::packet` for relay / passthrough use.
//
// Adding a new S→C packet
// ────────────────────────
// 1. Add a struct with the fields that callers supply.
// 2. `impl ServerPacket for YourStruct { fn to_payload... }` — build the bytes.
//
// `RawPacket` is available for any packet where the server constructs the
// payload manually and just needs type-safe forwarding.

#[allow(unused_imports)]
pub use crate::defs::packet::{RawPacket, ServerPacket};
use crate::defs::packet::pack_string;

// ── 0x01 PING ─────────────────────────────────────────────────────────────

pub struct Pong;
impl ServerPacket for Pong {
    fn to_payload(&self) -> Vec<u8> { vec![0x01] }
}

// ── 0x0F HEARTBEAT ────────────────────────────────────────────────────────

pub struct HeartbeatReply;
impl ServerPacket for HeartbeatReply {
    fn to_payload(&self) -> Vec<u8> { vec![0x0F] }
}

// ── 0x26 LOGIN_RESPONSE ───────────────────────────────────────────────────
//
// S→C: [0x26][i16(zone_trail_count=0)][Str(world_name)][Str(player_uid)][u8(zone_type=0)]

pub struct LoginResponse<'a> {
    pub world_name: &'a str,
    pub player_uid: &'a str,
}
impl ServerPacket for LoginResponse<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x26u8];
        p.extend_from_slice(&0i16.to_le_bytes()); // zone_trail_count = 0
        p.extend(pack_string(self.world_name));
        p.extend(pack_string(self.player_uid));
        p.push(0x00); // zone_type = 0
        p
    }
}

// ── 0x2A UNIQUE_IDS ───────────────────────────────────────────────────────
//
// S→C: [0x2A][i16(count)][u32 × count]
//
// IDA confirmed: case 41 in GameServerReceiver reads GetShort(count) then
// count × GetLong (= System_BitConverter__ToInt32, i.e. 4 bytes each) adding
// each to ConstructionControl.online_unique_ids_.
// Sets requesting_unique_ids=0 when done.
//
// The packet ID is 0x2A, NOT 0x29.  0x29 (case 40) is a different flow:
// server sends [0x29][Str(uid)] to ask the client to generate its own IDs,
// and the client responds with C→S 0x2A + uid + Short(10) + 10 Longs.
// For managed sessions we generate IDs ourselves and send them via 0x2A.
//
// `start` is the first ID in the allocated block; IDs are start..start+count.

pub struct UniqueIds {
    pub start: i64,
    pub count: u16,
}
impl ServerPacket for UniqueIds {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x2Au8];
        p.extend_from_slice(&self.count.to_le_bytes());
        for i in 0..self.count {
            // GetLong = System_BitConverter__ToInt32: reads 4 bytes, not 8
            p.extend_from_slice(&((self.start + i as i64) as u32).to_le_bytes());
        }
        p
    }
}

// ── 0x02 JOIN_CONFIRMED ───────────────────────────────────────────────────
//
// S→C: [0x02]
//   [Str(server_name)]
//   [u8(is_host)]
//   [u8(pvp=0)]
//   [Str(validator_code)]   ← client stores this and uses it to sign packets
//   [i16(validator_variation=0)]
//   [i16(n_others)]
//   [n_others × Str(player_name)]
//
// IDA confirmed (case 1): n_others player names follow, but the loop only
// runs when is_host=true AND n_others>0.  We always send n_others=0 so the
// name list is empty.  Using username as validator_code is fine — we strip
// the validator field from all incoming packets without checking it.

pub struct JoinConfirmed<'a> {
    pub server_name: &'a str,
    pub username:    &'a str,
    pub is_host:     bool,
}
impl ServerPacket for JoinConfirmed<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x02u8];
        p.extend(pack_string(self.server_name));
        p.push(self.is_host as u8);
        p.push(0x00);
        p.extend(pack_string(self.username));
        p.extend_from_slice(&0i16.to_le_bytes());
        p.extend_from_slice(&0i16.to_le_bytes());
        p
    }
}

// ── 0x05 SESSION_INIT ─────────────────────────────────────────────────────
//
// IDA confirmed (case 4 in GameServerReceiver$$OnReceive).  This is a
// compound packet — the receiver reads all of the following in order:
//
//   [0x05]
//   i16  uid_count          — unique IDs to add to client's online_unique_ids_ pool
//   i32  × uid_count        — GetLong each
//   i16  daynight_ms        — consumed by ReceiveDaynight (time of day in ms)
//   i16  disabled_perk_count
//   Str  × disabled_perk_count
//   u8   client_is_mod      — sets GameServerConnector.is_moderator
//   u8   max_companions     — sets CompanionController.max_personal_companions_right_now
//   u8   skip_saved_pos     — 0 = client requests its saved zone/position; 1 = skip
//   u8   pvp_enabled        — sets GameServerConnector.pvp_enabled
//   u8   padding            — GetByte, discarded

pub struct SessionInit {
    pub daynight_ms:    i16,
    pub client_is_mod:  bool,
    pub max_companions: u8,
    pub pvp_enabled:    bool,
    pub uid_start:      i64,
    pub uid_count:      u16,
}
impl ServerPacket for SessionInit {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x05u8];
        p.extend_from_slice(&(self.uid_count as i16).to_le_bytes());
        for i in 0..self.uid_count {
            p.extend_from_slice(&((self.uid_start + i as i64) as u32).to_le_bytes());
        }
        p.extend_from_slice(&self.daynight_ms.to_le_bytes());
        p.extend_from_slice(&0i16.to_le_bytes()); // disabled_perk_count = 0
        p.push(self.client_is_mod as u8);
        p.push(self.max_companions);
        p.push(0x01); // skip_saved_pos = 1 → don't override our spawn via zone packets
        p.push(self.pvp_enabled as u8);
        p.push(0x00); // padding — GetByte, discarded
        p
    }
}

// ── 0x0B ZONE_DATA ────────────────────────────────────────────────────────
//
// switch dispatch is opcode-1, so case 10 = opcode 0x0B.
// case 10 reads: GetByte(flag) + GetByte(second_byte), then if flag==1 calls
// ProcessIncomingZoneData which reads: GetString(zone_name) + UnpackFromWeb + GetByte(type).
//
// S→C: [0x0B][u8 flag=1][u8 second_byte=0][Str zone_name]
//       ZoneData::UnpackFromWeb:
//         [InventoryItem][u8 rot][i16×4 coords][Str outer_item_zone][i16 timer_count]
//       [u8 type]

pub struct InteriorInfo<'a> {
    pub item_bytes: &'a [u8],
    pub rotation: u8,
    pub cx: i16,
    pub cz: i16,
    pub tx: i16,
    pub tz: i16,
    pub outer_zone: &'a str,
}

pub struct ZoneData<'a> {
    pub zone_name: &'a str,
    pub interior: Option<InteriorInfo<'a>>,
}
impl ServerPacket for ZoneData<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Bu8];
        p.push(0x01); // flag = 1 (zone data follows)
        p.extend(pack_string(self.zone_name));
        // ZoneData::UnpackFromWeb body
        match &self.interior {
            Some(s) => {
                p.extend_from_slice(s.item_bytes);
                p.push(s.rotation);
                p.extend_from_slice(&s.cx.to_le_bytes());
                p.extend_from_slice(&s.cz.to_le_bytes());
                p.extend_from_slice(&s.tx.to_le_bytes());
                p.extend_from_slice(&s.tz.to_le_bytes());
                p.extend(pack_string(s.outer_zone));
            }
            None => {
                // empty InventoryItem (3 × i16 zero counts)
                p.extend_from_slice(&[0u8; 6]);
                p.push(0x00); // rotation
                p.extend_from_slice(&[0u8; 8]); // 4 × i16 coords
                p.extend(pack_string("")); // outer_item_zone
            }
        }
        p.extend_from_slice(&0i16.to_le_bytes()); // timer_count = 0
        p.push(0x00); // type
        p
    }
}

// ── 0x17 DAY_NIGHT ────────────────────────────────────────────────────────

pub struct DayNight {
    pub ms: i16,
}
impl ServerPacket for DayNight {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x17u8];
        p.extend_from_slice(&self.ms.to_le_bytes());
        p
    }
}

// ── 0x07 JOIN_NOTIF ───────────────────────────────────────────────────────
//
// S→C: [0x07][Str(uid)][Str(display)][u8(joined)]
//
// IDA confirmed: client reads uid FIRST (used as dictionary key when
// joined=0 to release unique IDs back to the pool), display SECOND.
// Using username for both uid and display is correct for managed sessions.

pub struct JoinNotif<'a> {
    pub username: &'a str,
    pub joined:   bool,
}
impl ServerPacket for JoinNotif<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x07u8];
        p.extend(pack_string(self.username)); // uid  — used for ID cleanup on leave
        p.extend(pack_string(self.username)); // display
        p.push(self.joined as u8);
        p
    }
}

// ── 0x13 PLAYER_GONE ──────────────────────────────────────────────────────
//
// S→C: [0x13][u8(0=gone)][Str(username)][u8(mob_count=0)]

pub struct PlayerGone<'a> {
    pub username: &'a str,
}
impl ServerPacket for PlayerGone<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x13u8, 0x00];
        p.extend(pack_string(self.username));
        p.push(0x00); // mob_count = 0
        p
    }
}

// ── 0x13 PLAYER_NEARBY ────────────────────────────────────────────────────
//
// S→C: [0x13][u8(1=nearby)][Str(username)][Str(display)][OPD blob]

pub struct PlayerNearby<'a> {
    pub username:    &'a str,
    pub display:     &'a str,
    pub opd:         &'a [u8],
}
impl ServerPacket for PlayerNearby<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x13u8, 0x01];
        p.extend(pack_string(self.username));
        p.extend(pack_string(self.display));
        p.extend_from_slice(self.opd);
        p
    }
}

// ── 0x11 POSITION_UPDATE ──────────────────────────────────────────────────
//
// S→C: [0x11][Str(player)][raw position bytes]

pub struct PositionUpdate<'a> {
    pub player: &'a str,
    pub body:   &'a [u8],
}
impl ServerPacket for PositionUpdate<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x11u8];
        p.extend(pack_string(self.player));
        p.extend_from_slice(self.body);
        p
    }
}

// ── 0x06 CHAT ─────────────────────────────────────────────────────────────
//
// S→C: [0x06][Str(player_id)][Str(display_name)][Str(message)][u8(type)]

pub struct ChatBroadcast<'a> {
    pub player_id:    &'a str,
    pub display_name: &'a str,
    pub message:      &'a str,
    pub chat_type:    u8,
}
impl ServerPacket for ChatBroadcast<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x06u8];
        p.extend(pack_string(self.player_id));
        p.extend(pack_string(self.display_name));
        p.extend(pack_string(self.message));
        p.push(self.chat_type);
        p
    }
}

// ── 0x14 ZONE_CHANGE ──────────────────────────────────────────────────────
//
// S→C: [0x14][Str(player)][Str(zone)]

pub struct ZoneChangeBroadcast<'a> {
    pub player:    &'a str,
    pub zone_name: &'a str,
}
impl ServerPacket for ZoneChangeBroadcast<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x14u8];
        p.extend(pack_string(self.player));
        p.extend(pack_string(self.zone_name));
        p
    }
}

// ── 0x0A ZONE_RELAY_TO_HOST ───────────────────────────────────────────────
//
// Sent to host when a guest requests zone data.
// S→C (host receives): [0x0A][Str(zone_name)][Str(requester)][u8(type)][optional Pos]

pub struct ZoneRelayToHost<'a> {
    pub zone_name: &'a str,
    pub requester: &'a str,
    pub zone_type: u8,
    pub extra:     &'a [u8],
}
impl ServerPacket for ZoneRelayToHost<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Au8];
        p.extend(pack_string(self.zone_name));
        p.extend(pack_string(self.requester));
        p.push(self.zone_type);
        p.extend_from_slice(self.extra);
        p
    }
}

// ── 0x0C CHUNK_RELAY_TO_HOST ─────────────────────────────────────────────
//
// Sent to host when a guest requests a chunk.
// S→C (host receives): [0x0C][Str(requester)][Str(zone)][Str(sub_zone)][i16(x)][i16(z)]

pub struct ChunkRelayToHost<'a> {
    pub requester: &'a str,
    pub zone_name: &'a str,
    pub sub_zone:  &'a str,
    pub x:         i16,
    pub z:         i16,
}
impl ServerPacket for ChunkRelayToHost<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Cu8];
        p.extend(pack_string(self.requester));
        p.extend(pack_string(self.zone_name));
        p.extend(pack_string(self.sub_zone));
        p.extend_from_slice(&self.x.to_le_bytes());
        p.extend_from_slice(&self.z.to_le_bytes());
        p
    }
}

// ── 0x0D CHUNK_FOR_GUEST ─────────────────────────────────────────────────
//
// S→C: [0x0D][Str(zone)][i16(cx)][i16(cz)][u8(flag)][Str(checkpoint)][body]

pub struct ChunkForGuest<'a> {
    pub zone:       &'a str,
    pub cx:         i16,
    pub cz:         i16,
    pub flag:       u8,
    pub checkpoint: &'a str,
    pub body:       &'a [u8],
}
impl ServerPacket for ChunkForGuest<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Du8];
        p.extend(pack_string(self.zone));
        p.extend_from_slice(&self.cx.to_le_bytes());
        p.extend_from_slice(&self.cz.to_le_bytes());
        p.push(self.flag);
        p.extend(pack_string(self.checkpoint));
        p.extend_from_slice(self.body);
        p
    }
}

// ── 0x0B ZONE_FOR_GUEST ───────────────────────────────────────────────────
//
// Forwarded from host to guest after stripping the requester field.
// S→C: [0x0B][u8(1)][u8(0)][Str(zone_name)][zone_data...]

pub struct ZoneForGuest<'a> {
    pub zone_name: &'a str,
    pub data:      &'a [u8],
}
impl ServerPacket for ZoneForGuest<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Bu8, 0x01, 0x00];
        p.extend(pack_string(self.zone_name));
        p.extend_from_slice(self.data);
        p
    }
}

// ── 0x1C CONTAINER_RELAY_TO_HOST ─────────────────────────────────────────
//
// S→C (host receives): [0x1C][Str(requester)][i64(basket_id)]

pub struct ContainerRelayToHost<'a> {
    pub requester: &'a str,
    pub basket_id: i64,
}
impl ServerPacket for ContainerRelayToHost<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x1Cu8];
        p.extend(pack_string(self.requester));
        p.extend_from_slice(&self.basket_id.to_le_bytes());
        p
    }
}

// ── 0x1B CONTAINER_CONTENTS ───────────────────────────────────────────────
//
// S→C: [0x1B][i64(basket_id)][BasketContents]
// `body` starts with basket_id bytes (host sends basket_id + contents together).

pub struct ContainerContents<'a> {
    pub body: &'a [u8],
}
impl ServerPacket for ContainerContents<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x1Bu8];
        p.extend_from_slice(self.body);
        p
    }
}

// ── 0x1E BASKET_UPDATE ────────────────────────────────────────────────────

/// Broadcast to all players: [0x1E][basket_id + BasketContents + item_name]
pub struct BasketUpdateBroadcast<'a> {
    pub basket_payload: &'a [u8],
}
impl ServerPacket for BasketUpdateBroadcast<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x1Eu8];
        p.extend_from_slice(self.basket_payload);
        p
    }
}

/// Relay to host for persistence: [0x1E][Str(requester)][basket_payload]
pub struct BasketUpdateToHost<'a> {
    pub requester:      &'a str,
    pub basket_payload: &'a [u8],
}
impl ServerPacket for BasketUpdateToHost<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x1Eu8];
        p.extend(pack_string(self.requester));
        p.extend_from_slice(self.basket_payload);
        p
    }
}

// ── 0x35/0x36/0x37 MINIGAME ───────────────────────────────────────────────

/// 0x35: [Str(challenger)][minigame_type...]
pub struct MinigameChallengeRelay<'a> {
    pub challenger: &'a str,
    pub rest:       &'a [u8],
}
impl ServerPacket for MinigameChallengeRelay<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x35u8];
        p.extend(pack_string(self.challenger));
        p.extend_from_slice(self.rest);
        p
    }
}

/// 0x36: [Str(responder)][u8(response)][rest...]
pub struct MinigameResponseRelay<'a> {
    pub responder: &'a str,
    pub response:  u8,
    pub rest:      &'a [u8],
}
impl ServerPacket for MinigameResponseRelay<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x36u8];
        p.extend(pack_string(self.responder));
        p.push(self.response);
        p.extend_from_slice(self.rest);
        p
    }
}

/// 0x37: [Str(sender)][rest...]
pub struct BeginMinigameRelay<'a> {
    pub sender: &'a str,
    pub rest:   &'a [u8],
}
impl ServerPacket for BeginMinigameRelay<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x37u8];
        p.extend(pack_string(self.sender));
        p.extend_from_slice(self.rest);
        p
    }
}

// ── 0x27 SET_INTERACTING ─────────────────────────────────────────────────
//
// S→C: [0x27][Str(player)][Str(object_key)]
//
// IDA confirmed (case 38 in GameServerReceiver):
//   v701 = GetString()  // player username
//   v702 = GetString()  // container key ("zone/cx/cz/ix/iz")
//   *(_QWORD *)(nearby_player + 48) = v702   // sets field+48
//
// This is what AnyoneUsing checks — NOT currently_using (field+32 in OPD).
// Broadcast to all zone peers when a player opens a basket so their
// AnyoneUsing returns true and shows the "in use" popup.

pub struct SetInteractingObject<'a> {
    pub player:     &'a str,
    pub object_key: &'a str,
}
impl ServerPacket for SetInteractingObject<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x27u8];
        p.extend(pack_string(self.player));
        p.extend(pack_string(self.object_key));
        p
    }
}

// ── 0x28 RELEASE_INTERACTING ─────────────────────────────────────────────
//
// S→C: [0x28][Str(player)]
//
// IDA confirmed (case 39 in GameServerReceiver):
//   v1111 = GetString()  // player username
//   *(_QWORD *)(nearby_player + 48) = ""    // clears field+48
//
// Broadcast to all zone peers when a player closes a basket.

pub struct ReleaseInteractingObject<'a> {
    pub player: &'a str,
}
impl ServerPacket for ReleaseInteractingObject<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x28u8];
        p.extend(pack_string(self.player));
        p
    }
}

// ── Generic patterns ───────────────────────────────────────────────────────
//
// These cover the large group of packets that follow one of three shapes:
//
//   PlayerPrefixPacket:  [id][Str(player)][body]
//   NoPrefixPacket:      [id][body]
//   PointToPointPacket:  [id][Str(from_or_to)][body]

/// [id][Str(player)][body] — many position/state broadcasts.
pub struct PlayerPrefixPacket<'a> {
    pub id:     u8,
    pub player: &'a str,
    pub body:   &'a [u8],
}
impl ServerPacket for PlayerPrefixPacket<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![self.id];
        p.extend(pack_string(self.player));
        p.extend_from_slice(self.body);
        p
    }
}

/// [id][body] — broadcast with no player name prefix.
pub struct NoPrefixPacket<'a> {
    pub id:   u8,
    pub body: &'a [u8],
}
impl ServerPacket for NoPrefixPacket<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![self.id];
        p.extend_from_slice(self.body);
        p
    }
}

/// [id][Str(target_or_from)][body] — point-to-point with a name field.
pub struct NamedRelayPacket<'a> {
    pub id:   u8,
    pub name: &'a str,
    pub body: &'a [u8],
}
impl ServerPacket for NamedRelayPacket<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![self.id];
        p.extend(pack_string(self.name));
        p.extend_from_slice(self.body);
        p
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────
//
// Each test simulates the client-side parser so that a wrong encoding fails
// here rather than silently misbehaving in game.  The helpers mirror the
// actual IL2CPP methods:
//
//   GetByte  → read 1 byte
//   GetShort → System_BitConverter__ToInt16 → 2 bytes LE
//   GetLong  → System_BitConverter__ToInt32 → 4 bytes LE  (NOT 8!)
//   GetString→ GetShort(len) + len bytes

#[cfg(test)]
mod tests {
    use super::*;

    // ── Client-side parser primitives ─────────────────────────────────────

    fn get_byte(buf: &[u8], off: usize) -> (u8, usize) {
        (buf[off], off + 1)
    }

    fn get_short(buf: &[u8], off: usize) -> (i16, usize) {
        let v = i16::from_le_bytes([buf[off], buf[off + 1]]);
        (v, off + 2)
    }

    // GetLong = System_BitConverter__ToInt32: reads only 4 bytes.
    fn get_long(buf: &[u8], off: usize) -> (i32, usize) {
        let v = i32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        (v, off + 4)
    }

    // GetString: GetShort(byte_len) + byte_len bytes of UTF-16LE.
    fn get_string(buf: &[u8], off: usize) -> (String, usize) {
        let (byte_len, off) = get_short(buf, off);
        let byte_len = byte_len as usize;
        let u16s: Vec<u16> = buf[off..off + byte_len]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        (String::from_utf16(&u16s).unwrap(), off + byte_len)
    }

    // ── 0x01 Pong ─────────────────────────────────────────────────────────

    #[test]
    fn pong_is_single_byte() {
        let p = Pong.to_payload();
        assert_eq!(p, vec![0x01]);
    }

    // ── 0x2A UniqueIds ────────────────────────────────────────────────────
    //
    // Case 41: GetShort(count) then count × GetLong (= 4 bytes each).
    // Regression test for the original 8-byte-per-ID bug.

    #[test]
    fn unique_ids_encodes_as_u32_not_u64() {
        let pkt = UniqueIds { start: 1000, count: 3 }.to_payload();
        assert_eq!(pkt[0], 0x2A);
        let (count, mut off) = get_short(&pkt, 1);
        assert_eq!(count, 3);
        for i in 0..3i32 {
            let (id, next) = get_long(&pkt, off);
            assert_eq!(id, 1000 + i, "id[{i}] wrong");
            off = next;
        }
        assert_eq!(off, pkt.len(), "trailing bytes after IDs");
    }

    #[test]
    fn unique_ids_wraps_at_u32_boundary() {
        // IDs near u32::MAX should wrap, not overflow into 8 bytes.
        let start = u32::MAX as i64 - 1;
        let pkt = UniqueIds { start, count: 2 }.to_payload();
        let (_, mut off) = get_short(&pkt, 1);
        let (id0, next) = get_long(&pkt, off); off = next;
        let (id1, _)    = get_long(&pkt, off);
        assert_eq!(id0 as u32, (start) as u32);
        assert_eq!(id1 as u32, (start + 1) as u32);
    }

    // ── 0x26 LoginResponse ────────────────────────────────────────────────
    //
    // Case 37: GetShort(zone_trail_count=0), GetString(world), GetString(uid), GetByte(zone_type).

    #[test]
    fn login_response_layout() {
        let pkt = LoginResponse { world_name: "TestWorld", player_uid: "alice" }.to_payload();
        assert_eq!(pkt[0], 0x26);
        let (trail, off) = get_short(&pkt, 1);
        assert_eq!(trail, 0);
        let (world, off) = get_string(&pkt, off);
        assert_eq!(world, "TestWorld");
        let (uid, off) = get_string(&pkt, off);
        assert_eq!(uid, "alice");
        let (zone_type, off) = get_byte(&pkt, off);
        assert_eq!(zone_type, 0);
        assert_eq!(off, pkt.len());
    }

    // ── 0x02 JoinConfirmed ────────────────────────────────────────────────
    //
    // Case 1: GetString(server), GetByte(is_host), GetByte(pvp),
    //         GetString(validator), GetShort(validator_var), GetShort(n_others).

    #[test]
    fn join_confirmed_layout() {
        let pkt = JoinConfirmed { server_name: "MySrv", username: "bob", is_host: true }.to_payload();
        assert_eq!(pkt[0], 0x02);
        let (srv, off)  = get_string(&pkt, 1);
        assert_eq!(srv, "MySrv");
        let (host, off) = get_byte(&pkt, off);
        assert_eq!(host, 1);
        let (pvp, off)  = get_byte(&pkt, off);
        assert_eq!(pvp, 0);
        let (val, off)  = get_string(&pkt, off);
        assert_eq!(val, "bob");
        let (var, off)  = get_short(&pkt, off);
        assert_eq!(var, 0);
        let (n, off)    = get_short(&pkt, off);
        assert_eq!(n, 0);
        assert_eq!(off, pkt.len());
    }

    // ── 0x07 JoinNotif ────────────────────────────────────────────────────
    //
    // Case 6: GetString(uid), GetString(display), GetByte(joined).

    #[test]
    fn join_notif_layout() {
        let pkt = JoinNotif { username: "carol", joined: false }.to_payload();
        assert_eq!(pkt[0], 0x07);
        let (uid, off)  = get_string(&pkt, 1);
        let (disp, off) = get_string(&pkt, off);
        let (join, off) = get_byte(&pkt, off);
        assert_eq!(uid, "carol");
        assert_eq!(disp, "carol");
        assert_eq!(join, 0);
        assert_eq!(off, pkt.len());
    }

    // ── 0x13 PlayerGone ───────────────────────────────────────────────────
    //
    // Case 18: GetByte(type=0), GetString(uid), GetByte(mob_count).

    #[test]
    fn player_gone_layout() {
        let pkt = PlayerGone { username: "dave" }.to_payload();
        assert_eq!(pkt[0], 0x13);
        let (kind, off) = get_byte(&pkt, 1);
        assert_eq!(kind, 0);
        let (uid, off)  = get_string(&pkt, off);
        assert_eq!(uid, "dave");
        let (mobs, off) = get_byte(&pkt, off);
        assert_eq!(mobs, 0);
        assert_eq!(off, pkt.len());
    }

    // ── 0x27 SetInteractingObject ─────────────────────────────────────────
    //
    // Case 38: GetString(player), GetString(object_key) → *(player+48) = key.

    #[test]
    fn set_interacting_layout() {
        let pkt = SetInteractingObject {
            player:     "eve",
            object_key: "overworld/1/2/3/4",
        }.to_payload();
        assert_eq!(pkt[0], 0x27);
        let (player, off) = get_string(&pkt, 1);
        let (key, off)    = get_string(&pkt, off);
        assert_eq!(player, "eve");
        assert_eq!(key, "overworld/1/2/3/4");
        assert_eq!(off, pkt.len());
    }

    // ── 0x28 ReleaseInteractingObject ─────────────────────────────────────
    //
    // Case 39: GetString(player) → *(player+48) = "".

    #[test]
    fn release_interacting_layout() {
        let pkt = ReleaseInteractingObject { player: "eve" }.to_payload();
        assert_eq!(pkt[0], 0x28);
        let (player, off) = get_string(&pkt, 1);
        assert_eq!(player, "eve");
        assert_eq!(off, pkt.len());
    }

    // ── 0x05 SessionInit ──────────────────────────────────────────────────
    //
    // Verify the compound structure so a future refactor can't accidentally
    // break the receiver's sequential reads.

    #[test]
    fn session_init_layout() {
        let pkt = SessionInit {
            daynight_ms:    12000,
            client_is_mod:  false,
            max_companions: 3,
            pvp_enabled:    true,
            uid_start:      0,
            uid_count:      0,
        }.to_payload();
        let mut off = 0;
        assert_eq!(pkt[off], 0x05); off += 1;
        // uid_count = 0
        let uid_count = get_short(&pkt, off); off += 2;
        assert_eq!(uid_count.0, 0);
        // daynight_ms
        let daynight = get_short(&pkt, off); off += 2;
        assert_eq!(daynight.0, 12000);
        // disabled_perk_count = 0
        let perk_count = get_short(&pkt, off); off += 2;
        assert_eq!(perk_count.0, 0);
        // client_is_mod, max_companions, skip_saved_pos, pvp, padding
        let (is_mod,  off) = get_byte(&pkt, off);
        let (max_cmp, off) = get_byte(&pkt, off);
        let (skip,    off) = get_byte(&pkt, off);
        let (pvp,     off) = get_byte(&pkt, off);
        let (_pad,    off) = get_byte(&pkt, off);
        assert_eq!(is_mod,  0);
        assert_eq!(max_cmp, 3);
        assert_eq!(skip,    1); // must be non-zero so client skips saved-position request
        assert_eq!(pvp,     1);
        assert_eq!(off, pkt.len());
    }
}
