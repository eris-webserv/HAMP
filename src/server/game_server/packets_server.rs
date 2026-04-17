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

// ── 0x05 PVP_STATE ────────────────────────────────────────────────────────
//
// S→C: [0x05][u8(pvp_enabled)][u8(0)]
//
// IDA confirmed (case 4 in GameServerReceiver$OnReceive):
//   pvp_enabled = (byte == 1)
//   Written to GameServerConnector::Instance.pvp_enabled.
//   The second byte is read but ignored.
//
// Absence of this packet leaves pvp_enabled == false on the client, which
// causes CombatControl$HitAllowed to block all player-vs-player damage.

pub struct GamePvpState {
    pub pvp_enabled: bool,
}
impl ServerPacket for GamePvpState {
    fn to_payload(&self) -> Vec<u8> {
        vec![0x05u8, self.pvp_enabled as u8, 0x00]
    }
}

// ── 0x0B ZONE_DATA ────────────────────────────────────────────────────────
//
// Full zone-data packet including the ZoneData::UnpackFromWeb body.

pub struct ZoneData<'a> {
    pub zone_name: &'a str,
}
impl ServerPacket for ZoneData<'_> {
    fn to_payload(&self) -> Vec<u8> {
        let mut p = vec![0x0Bu8];
        p.push(0x01); // flag = 1 (zone data follows)
        p.push(0x00); // sub_flag = 0
        p.extend(pack_string(self.zone_name));
        // ZoneData::UnpackFromWeb: empty InventoryItem (3 zero counts) + zone_type
        p.extend_from_slice(&[0u8; 6]); // count1=0, count2=0, count3=0
        p.push(0x00);                   // zone_type = overworld
        p.extend_from_slice(&[0u8; 8]); // unknown1–4 (4 × i16)
        p.extend(pack_string(self.zone_name)); // zone_name (inner)
        p.extend_from_slice(&0i16.to_le_bytes()); // timer_dict_count = 0
        p.push(0x00); // trailing zone_type
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
