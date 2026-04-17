#![allow(dead_code)]
// packet.rs — wire primitives shared by both server types.
//
// ┌─ ServerPacket      trait: any type sendable to a client (to_payload)
// ├─ RawPacket         relay bytes verbatim as a ServerPacket
// ├─ Wire helpers      pack_string / unpack_string / craft_batch / to_hex_upper
// ├─ Str16             UTF-16LE length-prefixed string (BinRead + BinWrite)
// ├─ PacketHeader      9-byte batch envelope + packet-ID byte
// └─ Constants         DEFAULT_WORLD / LOGIN_SUCCESS_TRAILER
//
// Friend-server packets live in server::friend_server::packets_{client,server}.
// Game-server packets live in server::game_server::packets_{client,server}.

use std::io::Cursor;

use binrw::binrw;

// ── ServerPacket trait ─────────────────────────────────────────────────────
//
// Implemented by every S→C packet type in both servers.
// Prevents accidentally passing a C→S (client-bound) type to send_to.
//
// For #[binwrite] structs, use the `impl_server_packet!` macro below.
// For relay/raw packets, use `RawPacket`.

pub trait ServerPacket {
    /// Serialises this packet into wire bytes (packet-ID byte + field bytes).
    fn to_payload(&self) -> Vec<u8>;
}

/// Forwards raw bytes verbatim as a `ServerPacket`.
/// Used for relay packets where the server constructs the payload manually.
pub struct RawPacket {
    pub id:   u8,
    pub body: Vec<u8>,
}

impl RawPacket {
    pub fn new(id: u8, body: Vec<u8>) -> Self {
        Self { id, body }
    }
}

impl ServerPacket for RawPacket {
    fn to_payload(&self) -> Vec<u8> {
        let mut out = vec![self.id];
        out.extend_from_slice(&self.body);
        out
    }
}

/// Allows an already-serialised `Vec<u8>` (including the ID byte) to be
/// passed directly to `broadcast` / `send_to` without wrapping.
impl ServerPacket for Vec<u8> {
    fn to_payload(&self) -> Vec<u8> { self.clone() }
}

/// Generates a `ServerPacket` impl for a `#[binwrite]` struct.
///
/// ```ignore
/// impl_server_packet!(MyPacket, 0x0A);
/// ```
#[macro_export]
macro_rules! impl_server_packet {
    ($ty:ty, $id:expr) => {
        impl $crate::defs::packet::ServerPacket for $ty {
            fn to_payload(&self) -> Vec<u8> {
                let mut buf = vec![$id];
                let mut cur = std::io::Cursor::new(Vec::<u8>::new());
                binrw::BinWrite::write_le(self, &mut cur)
                    .expect("ServerPacket serialisation failed");
                buf.extend(cur.into_inner());
                buf
            }
        }
    };
}

// ── Constants ──────────────────────────────────────────────────────────────

/// "In Personal World" world-state blob.
/// Default world state blob (3-field UnpackWorldString format).
///
/// World state has TWO wire formats:
///   PackWorldString   (C→S in 0x2C): Byte + String + String + Short  (4 fields)
///   UnpackWorldString (S→C in login/0x16): Byte + String + Short     (3 fields)
///
/// The second String in PackWorldString is NOT read by UnpackWorldString.
/// We always store and transmit the 3-field version. The WorldUpdate (0x2C)
/// handler strips the extra String before storing (see `strip_world_update`).
///
/// This constant = Byte(0x01) + String("") + Short(0) = 5 bytes.
pub const DEFAULT_WORLD: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];

/// Fixed footer appended to every LOGIN_SUCCESS response.
///
/// Layout (confirmed from Ghidra `FriendServerReceiver$$OnReceive` case 0x0B):
///   i16  N_ToPing            = 0   (no hosts to ping on login)
///   i16  give_gems_on_open   = 0   (gems awarded when friend screen opens)
///   u8   show_warning_on_open= 0
///   i16  unknown             = 0
///   i16  N_trophies          = 0   (no trophies)
pub const LOGIN_SUCCESS_TRAILER: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

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
    // Client receive_buffer is 0x2000 (8192) bytes; anything larger overflows
    // it and corrupts memory. Warn loudly so oversized packets are caught early
    // during development rather than silently crashing the client.
    const CLIENT_RECV_BUF: usize = 8192;
    let total = 9 + payload.len();
    if total > CLIENT_RECV_BUF {
        eprintln!(
            "[WARN] craft_batch: packet 0x{:02X} is {} bytes — exceeds client receive buffer ({} bytes), client will crash!",
            payload.first().copied().unwrap_or(0),
            total,
            CLIENT_RECV_BUF,
        );
    }
    let total_len = total as u16;
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&total_len.to_le_bytes());
    out.push(0x01);
    out.push(qid);
    out.push(0x03);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Maximum payload bytes that fit in a single wire frame.
///
/// Client receive buffer is 8 192 bytes (0x2000); the 9-byte frame header
/// (total_len u16 + 0x01 + qid + status + payload_len u32) leaves 8 183 bytes
/// for payload.  `write_payload` uses this to decide whether to fragment.
pub const MAX_FRAME_PAYLOAD: usize = 8183;

/// Low-level helper: builds a single wire frame with an explicit status byte.
///
/// Fragment status semantics (confirmed via RE of `SendQueue$$Write` /
/// `ReceiveQueue$$Read`):
///   0 — continuation fragment, **last** in sequence → client finalises packet
///   1 — continuation fragment, more follow
///   2 — **first** fragment of a new multi-frame packet
///   3 — complete packet in one frame (normal `craft_batch` behaviour)
fn craft_frame(qid: u8, status: u8, payload: &[u8]) -> Vec<u8> {
    let total_len = (9 + payload.len()) as u16;
    let mut out = Vec::with_capacity(9 + payload.len());
    out.extend_from_slice(&total_len.to_le_bytes());
    out.push(0x01);   // one queue record
    out.push(qid);    // stream / queue id
    out.push(status); // fragment status
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Writes `payload` to `w`, transparently splitting into multiple frames when
/// the payload exceeds `MAX_FRAME_PAYLOAD`.
///
/// For small payloads this is identical to `write_all(&craft_batch(qid, payload))`.
/// For large payloads the game's built-in fragment protocol is used so the
/// client reassembles them before dispatching to its packet handler — no
/// client-side changes required.
pub fn write_payload<W: std::io::Write>(w: &mut W, qid: u8, payload: &[u8]) -> std::io::Result<()> {
    if payload.len() <= MAX_FRAME_PAYLOAD {
        return w.write_all(&craft_batch(qid, payload));
    }
    let chunks: Vec<&[u8]> = payload.chunks(MAX_FRAME_PAYLOAD).collect();
    let last = chunks.len() - 1;
    for (i, chunk) in chunks.iter().enumerate() {
        let status: u8 = if i == 0 { 2 } else if i == last { 0 } else { 1 };
        w.write_all(&craft_frame(qid, status, chunk))?;
    }
    Ok(())
}

/// Returns an uppercase hex string with no separator.
pub fn to_hex_upper(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut s, b| { s.push_str(&format!("{:02X}", b)); s },
    )
}

// ── Str16: UTF-16LE length-prefixed string ─────────────────────────────────
//
// Wire format:
//   u16 byte_len;          // byte count of the UTF-16LE data that follows
//   u16 chars[byte_len/2]; // code units, little-endian

#[binrw]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[brw(little)]
pub struct Str16 {
    #[br(temp)]
    #[bw(calc = (value.encode_utf16().count() * 2) as u16)]
    byte_len: u16,

    #[br(count = byte_len / 2, map = |v: Vec<u16>| String::from_utf16_lossy(&v).to_owned())]
    #[bw(map = |s: &String| s.encode_utf16().collect::<Vec<u16>>())]
    pub value: String,
}

impl Str16 {
    pub fn new(s: impl Into<String>) -> Self {
        Self { value: s.into() }
    }
}

impl From<&str>   for Str16 { fn from(s: &str)   -> Self { Self::new(s) } }
impl From<String> for Str16 { fn from(s: String)  -> Self { Self::new(s) } }

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

// Suppress the dead-code lint: Cursor is used transitively via the re-exported
// ServerPacket::to_payload, but rustc doesn't see through re-exports.
const _: () = { let _ = Cursor::<Vec<u8>>::new; };
