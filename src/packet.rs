// packets.rs — wire helpers + binary protocol types for the friend server.
//
// This file is the single home for everything protocol-related:
//
// ┌─ Wire helpers            pack_string / unpack_string / craft_batch / to_hex_upper
// ├─ Str16                   UTF-16LE length-prefixed string (BinRead + BinWrite)
// ├─ PacketHeader             The 9-byte batch envelope + packet-ID byte
// ├─ ClientPacket             Enum of every packet a CLIENT can send; parse() is
// │                           the single entry point for both servers
// └─ S→C packet structs       One struct per server-sent packet, each implementing
//                             ServerPacket for zero-copy serialisation
//
// Adding a new packet
// ───────────────────
// 1. Add the ID to `PacketId` in structs.rs.
// 2. For C→S: add a variant to `ClientPacket` and a match arm in `parse()`.
// 3. For S→C: add a struct with `#[binwrite]`, impl `ServerPacket`.

use std::io::{Cursor, Write};

use binrw::{binrw, binwrite, BinRead, BinReaderExt, BinWrite};

use crate::structs::PacketId;

// ── Default world-state blob ───────────────────────────────────────────────

/// Sent for any player whose world state has not yet been recorded.
/// Layout: u8(menu_id=1), then 6 zero bytes for room/count fields.
pub const DEFAULT_WORLD: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Fixed footer appended to every LOGIN_SUCCESS response.
///
/// Unpacked: u16(0), u16(10), u8(0), u16(0), u16(0)
/// The u16(10) appears to be a capacity or version field; purpose TBD.
pub const LOGIN_SUCCESS_TRAILER: &[u8] = &[0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

// ── Wire helpers ───────────────────────────────────────────────────────────

/// Encodes `s` as UTF-16LE with a 2-byte little-endian byte-length prefix.
///
/// Used for building login-success and other manually-assembled payloads that
/// mix fixed bytes with string fields and don't map cleanly to a single struct.
pub fn pack_string(s: &str) -> Vec<u8> {
    let encoded: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
    let mut out = Vec::with_capacity(2 + encoded.len());
    out.extend_from_slice(&(encoded.len() as u16).to_le_bytes());
    out.extend_from_slice(&encoded);
    out
}

/// Reads a UTF-16LE length-prefixed string from `data` at `offset`.
///
/// Returns `(decoded_string, new_offset)`. On any parse failure the original
/// offset is returned unchanged and the string is empty.
pub fn unpack_string(data: &[u8], offset: usize) -> (String, usize) {
    if offset + 2 > data.len() {
        return (String::new(), offset);
    }
    let byte_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
    let end = offset + 2 + byte_len;
    if end > data.len() {
        return (String::new(), offset);
    }
    let chars: Vec<u16> = data[offset + 2..end]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    (String::from_utf16_lossy(&chars).to_owned(), end)
}

/// Wraps `payload` in the batch packet envelope:
///
/// ```text
/// [total_len  : u16 LE]   (= 9 + payload.len())
/// [0x01]
/// [qid        : u8     ]
/// [0x03]
/// [payload_len: u32 LE]
/// [payload …]
/// ```
///
/// The packet-type byte is always the *first* byte of `payload`; callers are
/// responsible for prepending it before calling this function.
pub fn craft_batch(qid: u8, payload: &[u8]) -> Vec<u8> {
    let total_len = (9 + payload.len()) as u16;
    let mut out = Vec::with_capacity(9 + payload.len());
    out.extend_from_slice(&total_len.to_le_bytes());
    out.push(0x01);
    out.push(qid);
    out.push(0x03);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Returns an uppercase hex string with no separator — matches the Python
/// `binascii.hexlify(...).upper()` format used in the original log output.
pub fn to_hex_upper(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut s, b| { s.push_str(&format!("{:02X}", b)); s },
    )
}

// ── Str16: UTF-16LE length-prefixed string ─────────────────────────────────
//
// Wire format (matches Pattern Language):
//   u16 byte_len;          // byte count of the UTF-16LE data that follows
//   u16 chars[byte_len/2]; // code units, little-endian
//
// `byte_len` is a write-time temporary computed from `value`; it is never
// stored in the struct itself.

#[binrw]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[brw(little)]
pub struct Str16 {
    /// Byte length of the encoded UTF-16LE data.  Ephemeral: computed on
    /// write, consumed as a temp on read — never stored in the struct.
    #[br(temp)]
    #[bw(calc = (value.encode_utf16().count() * 2) as u16)]
    byte_len: u16,

    /// The decoded string value.
    #[br(count = byte_len / 2, map = |v: Vec<u16>| String::from_utf16_lossy(&v).to_owned())]
    #[bw(map = |s: &String| s.encode_utf16().collect::<Vec<u16>>())]
    pub value: String,
}

impl Str16 {
    pub fn new(s: impl Into<String>) -> Self {
        Self { value: s.into() }
    }
}

impl From<&str>  for Str16 { fn from(s: &str)   -> Self { Self::new(s) } }
impl From<String> for Str16 { fn from(s: String) -> Self { Self::new(s) } }

impl std::ops::Deref for Str16 {
    type Target = str;
    fn deref(&self) -> &str { &self.value }
}

impl std::fmt::Display for Str16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

// ── Batch packet header ────────────────────────────────────────────────────
//
// Every packet on the wire is wrapped in this 9-byte envelope, followed
// immediately by the packet-ID byte (which is the first byte of the payload).
//
// Wire layout:
//   u16 total_len;     // 9 + payload.len()
//   u8  0x01;
//   u8  qid;
//   u8  0x03;
//   u32 payload_len;   // payload.len()
//   u8  packet_id;     // first byte of payload

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct PacketHeader {
    pub total_len:   u16,
    _c1:             u8,   // always 0x01
    pub qid:         u8,
    _c3:             u8,   // always 0x03
    pub payload_len: u32,
    pub packet_id:   u8,
}

// ── ServerPacket trait ─────────────────────────────────────────────────────
//
// Implemented by every S→C struct. `to_payload()` prepends the packet-ID
// byte and serialises the struct fields, producing the raw bytes that are
// then handed to `craft_batch` / `conn.send`.

pub trait ServerPacket: for<'a> BinWrite<Args<'a> = ()> {
    const ID: u8;

    fn to_payload(&self) -> Vec<u8> {
        let mut buf = vec![Self::ID];
        let mut cur = Cursor::new(Vec::new());
        self.write_le(&mut cur).expect("ServerPacket serialisation failed");
        buf.extend(cur.into_inner());
        buf
    }
}

// ── Client-sendable packet enum ────────────────────────────────────────────

/// Every packet type a client can legitimately send to this server.
///
/// Server-only IDs (`PushReq`, `FrOnline`, `JumpToGame`, etc.) are absent —
/// `parse()` returns `None` for those bytes so they are dropped before any
/// handler logic runs.
#[derive(Debug)]
pub enum ClientPacket {
    /// `0x0A` — register a new account.
    RegisterReq {
        username: String,
    },

    /// `0x0B` — authenticate with username + token.
    Login {
        username: String,
        token:    String,
    },

    /// `0x0F` — keepalive ping; no payload fields.
    Heartbeat,

    /// `0x10` — send a friend request to another player.
    AddFriend {
        /// Raw (un-lowercased) target username as sent by the client.
        target: String,
    },

    /// `0x12` — accept a pending inbound friend request.
    AcceptFriend {
        target: String,
    },

    /// `0x18` — remove a player from the current user's friend list.
    RemoveFriend {
        target: String,
    },

    /// `0x1A` — private chat message destined for another online player.
    PrivateMsg {
        target:  String,
        message: String,
    },

    /// `0x2B` — host grants (`status = 1`) or denies (`status = 0`) a join
    /// request that was relayed through the server.
    JoinGrant {
        /// Lowercase target (the player who requested to join).
        target:    String,
        status:    u8,
        /// Room identifier included by the host when granting access.
        /// May be absent even on `status = 1` if the client omitted it.
        room_name: Option<String>,
    },

    /// `0x2C` — client broadcasts its current in-game / lobby world state.
    WorldUpdate {
        /// Raw blob bounded by the batch header's `total_len` field.
        world_data: Vec<u8>,
    },

    /// `0x2D` — request to join another player's active session.
    JoinReq {
        target:     String,
        /// Purpose not fully reversed; relayed verbatim to the host.
        extra_byte: u8,
    },
}

impl ClientPacket {
    /// Attempt to parse raw socket bytes into a typed packet.
    ///
    /// Returns `None` and the caller drops the packet silently if:
    /// - the buffer is shorter than 10 bytes (no room for header + ID),
    /// - the packet-ID byte is unknown to the protocol (`PacketId::from_u8`),
    /// - the packet-ID is a server-only ID, or
    /// - the payload is structurally malformed for its type.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 10 {
            return None;
        }

        let mut cur = Cursor::new(data);
        let hdr = PacketHeader::read(&mut cur).ok()?;

        // Gate on the full known-ID set before per-packet parsing.
        PacketId::from_u8(hdr.packet_id)?;

        match hdr.packet_id {
            // ── REGISTER_REQ (0x0A) ───────────────────────────────────────
            // C→S  [username: Str16]
            0x0A => {
                let username = Str16::read(&mut cur).ok()?;
                Some(Self::RegisterReq { username: username.value })
            }

            // ── LOGIN (0x0B) ──────────────────────────────────────────────
            // C→S  [username: Str16] [token: Str16]
            0x0B => {
                let username = Str16::read(&mut cur).ok()?;
                let token    = Str16::read(&mut cur).ok()?;
                Some(Self::Login { username: username.value, token: token.value })
            }

            // ── HEARTBEAT (0x0F) ──────────────────────────────────────────
            // C→S  (no payload fields)
            0x0F => Some(Self::Heartbeat),

            // ── ADD_FRIEND (0x10) ─────────────────────────────────────────
            // C→S  [target: Str16]
            0x10 => {
                let target = Str16::read(&mut cur).ok()?;
                Some(Self::AddFriend { target: target.value })
            }

            // ── ACCEPT_FRIEND (0x12) ──────────────────────────────────────
            // C→S  [target: Str16]
            0x12 => {
                let target = Str16::read(&mut cur).ok()?;
                Some(Self::AcceptFriend { target: target.value })
            }

            // ── REMOVE_FRIEND (0x18) ──────────────────────────────────────
            // C→S  [target: Str16]
            0x18 => {
                let target = Str16::read(&mut cur).ok()?;
                Some(Self::RemoveFriend { target: target.value })
            }

            // ── PRIVATE_MSG (0x1A) ────────────────────────────────────────
            // C→S  [target: Str16] [message: Str16]
            0x1A => {
                let target  = Str16::read(&mut cur).ok()?;
                let message = Str16::read(&mut cur).ok()?;
                Some(Self::PrivateMsg { target: target.value, message: message.value })
            }

            // ── JOIN_GRANT (0x2B) ─────────────────────────────────────────
            // C→S  [target: Str16] [status: u8] [room_name: Str16]?
            //
            // room_name is present only when status == 1, and is optional
            // even then — parse it when the bytes are there, otherwise None.
            0x2B => {
                let target = Str16::read(&mut cur).ok()?;
                let status: u8 = cur.read_le().ok()?;
                let room_name = if status == 1 {
                    Str16::read(&mut cur).ok().map(|s| s.value)
                } else {
                    None
                };
                Some(Self::JoinGrant { target: target.value, status, room_name })
            }

            // ── WORLD_UPDATE (0x2C) ───────────────────────────────────────
            // C→S  raw blob; length from batch header, not a payload field.
            0x2C => {
                let end = (hdr.total_len as usize).min(data.len());
                if end <= 10 { return None; }
                Some(Self::WorldUpdate { world_data: data[10..end].to_vec() })
            }

            // ── JOIN_REQ (0x2D) ───────────────────────────────────────────
            // C→S  [target: Str16] [extra_byte: u8]
            0x2D => {
                let target:     Str16 = Str16::read(&mut cur).ok()?;
                let extra_byte: u8    = cur.read_le().unwrap_or(0);
                Some(Self::JoinReq { target: target.value, extra_byte })
            }

            // All other IDs that survived `from_u8` are server-only — drop.
            _ => None,
        }
    }

    /// The `PacketId` variant that corresponds to this packet.
    pub fn id(&self) -> PacketId {
        match self {
            Self::RegisterReq { .. }  => PacketId::RegisterReq,
            Self::Login { .. }        => PacketId::Login,
            Self::Heartbeat           => PacketId::Heartbeat,
            Self::AddFriend { .. }    => PacketId::AddFriend,
            Self::AcceptFriend { .. } => PacketId::AcceptFriend,
            Self::RemoveFriend { .. } => PacketId::RemoveFriend,
            Self::PrivateMsg { .. }   => PacketId::PrivateMsg,
            Self::JoinGrant { .. }    => PacketId::JoinGrant,
            Self::WorldUpdate { .. }  => PacketId::WorldUpdate,
            Self::JoinReq { .. }      => PacketId::JoinReq,
        }
    }
}

// ── Server-sent packet structs ─────────────────────────────────────────────
//
// Each struct covers exactly the payload fields that follow the packet-ID
// byte. The ID byte itself is prepended by `ServerPacket::to_payload()` so
// it never needs to appear in the struct.
//
// `world_data` fields are `Vec<u8>` — binrw writes Vec<u8> as raw bytes
// with no length prefix, which is correct here because the receiver derives
// the length from the outer batch header's `total_len`.

// ── Registration ──────────────────────────────────────────────────────────

/// `0x0A 0x01` — account created successfully.
///
/// Wire (after packet-ID): [0x01] [username: Str16] [display: Str16] [token: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct RegisterOk {
    #[bw(calc = 0x01u8)] _status: u8,
    pub username: Str16,
    pub display:  Str16,
    pub token:    Str16,
}
impl ServerPacket for RegisterOk { const ID: u8 = 0x0A; }

/// `0x0A 0x02` — registration rejected (username already taken).
///
/// Wire (after packet-ID): [0x02] [reason: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct RegisterFail {
    #[bw(calc = 0x02u8)] _status: u8,
    pub reason: Str16,
}
impl ServerPacket for RegisterFail { const ID: u8 = 0x0A; }

// ── Authentication ────────────────────────────────────────────────────────

/// `0x0C` — authentication failure (bad username or token).
///
/// Wire (after packet-ID): (empty)
#[binwrite]
#[derive(Debug, Default)]
#[bw(little)]
pub struct AuthFail;
impl ServerPacket for AuthFail { const ID: u8 = 0x0C; }

// ── Heartbeat ─────────────────────────────────────────────────────────────

/// `0x0F` — heartbeat reply.
///
/// Wire (after packet-ID): (empty)
#[binwrite]
#[derive(Debug, Default)]
#[bw(little)]
pub struct HeartbeatReply;
impl ServerPacket for HeartbeatReply { const ID: u8 = 0x0F; }

// ── Friend requests ───────────────────────────────────────────────────────

/// `0x10 0x00` — friend request sent successfully.
///
/// IMPORTANT: the game client expects **display FIRST, username SECOND** here,
/// which is the opposite of most other packets. This is a confirmed quirk of
/// the protocol.
///
/// Wire (after packet-ID): [0x00] [display: Str16] [username: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct AddFriendOk {
    #[bw(calc = 0x00u8)] _status: u8,
    /// Display name of the target player.
    pub display:  Str16,
    /// Raw (un-lowercased) username of the target player.
    pub username: Str16,
}
impl ServerPacket for AddFriendOk { const ID: u8 = 0x10; }

/// `0x10 0x01` — friend request failed (target not found or already friends).
///
/// Wire (after packet-ID): [0x01] [target: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct AddFriendFail {
    #[bw(calc = 0x01u8)] _status: u8,
    pub target: Str16,
}
impl ServerPacket for AddFriendFail { const ID: u8 = 0x10; }

/// `0x11` — an incoming friend request pushed to the target player.
///
/// IMPORTANT: **username FIRST, display SECOND** — opposite of AddFriendOk.
///
/// Wire (after packet-ID): [from_username: Str16] [from_display: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct PushFriendReq {
    pub username: Str16,
    pub display:  Str16,
}
impl ServerPacket for PushFriendReq { const ID: u8 = 0x11; }

// ── Accept / push_accepted ────────────────────────────────────────────────

/// `0x12` — confirmation to the accepting client that the request was handled.
///
/// Wire (after packet-ID):
///   [target: Str16] [is_online: u8]
///   if is_online == 1: [world_data: Vec<u8>]  (7+ bytes, no length prefix)
///   if is_online == 0: [0x00]                 (mandatory presence marker)
///
/// Caller is responsible for setting `world_data` to `vec![0x00]` when
/// offline, or the actual world-state blob when online.
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct AcceptFriendOk {
    pub target:     Str16,
    pub is_online:  u8,
    /// Either the 7-byte world-state blob (online) or `[0x00]` (offline).
    pub world_data: Vec<u8>,
}
impl ServerPacket for AcceptFriendOk { const ID: u8 = 0x12; }

/// `0x13` — pushed to the player whose outbound request was just accepted.
///
/// Wire (after packet-ID):
///   [username: Str16] [display: Str16] [0x01] [world_data: Vec<u8>]
///
/// `0x01` is the is_online flag; the accepting player is always online.
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct PushAccepted {
    pub username:   Str16,
    pub display:    Str16,
    #[bw(calc = 0x01u8)] _online: u8,
    pub world_data: Vec<u8>,
}
impl ServerPacket for PushAccepted { const ID: u8 = 0x13; }

// ── Removal ───────────────────────────────────────────────────────────────

/// `0x15` — pushed to a player who was removed from someone else's list.
///
/// Wire (after packet-ID): [removed_by: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct PushRemoved {
    pub username: Str16,
}
impl ServerPacket for PushRemoved { const ID: u8 = 0x15; }

/// `0x18` — confirmation to the initiating client that removal succeeded.
///
/// Wire (after packet-ID): [target: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct RemoveFriendOk {
    pub target: Str16,
}
impl ServerPacket for RemoveFriendOk { const ID: u8 = 0x18; }

// ── Presence broadcasts ───────────────────────────────────────────────────

/// `0x16` — a friend came online, or their world state changed.
///
/// Wire (after packet-ID): [username: Str16] [world_data: Vec<u8>]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct FriendOnline {
    pub username:   Str16,
    pub world_data: Vec<u8>,
}
impl ServerPacket for FriendOnline { const ID: u8 = 0x16; }

/// `0x17` — a friend went offline.
///
/// Wire (after packet-ID): [username: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct FriendOffline {
    pub username: Str16,
}
impl ServerPacket for FriendOffline { const ID: u8 = 0x17; }

// ── Private message relay ─────────────────────────────────────────────────

/// `0x1A` — private message forwarded to its recipient.
///
/// Wire (after packet-ID): [from: Str16] [message: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct RelayPrivateMsg {
    pub from:    Str16,
    pub message: Str16,
}
impl ServerPacket for RelayPrivateMsg { const ID: u8 = 0x1A; }

// ── Session join signalling ───────────────────────────────────────────────

/// `0x2D` — a join request relayed to the target host.
///
/// Wire (after packet-ID): [from: Str16] [extra_byte: u8]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct RelayJoinReq {
    pub from:       Str16,
    pub extra_byte: u8,
}
impl ServerPacket for RelayJoinReq { const ID: u8 = 0x2D; }

/// `0x2B` (no payload) — sent to the **host** to clear its "Allowing…" UI popup.
///
/// Wire (after packet-ID): (empty)
#[binwrite]
#[derive(Debug, Default)]
#[bw(little)]
pub struct JoinGrantHostClear;
impl ServerPacket for JoinGrantHostClear { const ID: u8 = 0x2B; }

/// `0x2B` (with payload) — join grant or denial relayed to the **joiner**.
///
/// Wire (after packet-ID):
///   [from: Str16] [status: u8]
///   if status == 1: [room_name: Str16]
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct JoinGrantRelay {
    /// Display name of the host player.
    pub from:      Str16,
    pub status:    u8,
    /// Present only when `status == 1`. binrw skips writing `None`.
    #[bw(if(*status == 1))]
    pub room_name: Option<Str16>,
}
impl ServerPacket for JoinGrantRelay { const ID: u8 = 0x2B; }

/// `0x25` — P2P handoff packet sent to the **joiner** after a successful grant.
///
/// Wire (after packet-ID):
///   [display: Str16]   host player's display name
///   [token: Str16]     room identifier / join token  
///   [host_ip: Str16]   host's socket IP (from active_sessions addr)
///   [mode: Str16]      connection mode, e.g. "P2P"
///   [port: u16]        target port (typically 7003)
///   [password_flag: u8] 0x00 = no password
#[binwrite]
#[derive(Debug)]
#[bw(little)]
pub struct JumpToGame {
    pub display:       Str16,
    pub token:         Str16,
    pub host_ip:       Str16,
    pub mode:          Str16,
    pub port:          u16,
    pub password_flag: u8,
}
impl ServerPacket for JumpToGame { const ID: u8 = 0x25; }

