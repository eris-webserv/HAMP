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
use std::sync::atomic::{AtomicBool, Ordering};

pub static LOG_PACKETS: AtomicBool = AtomicBool::new(false);

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
    if LOG_PACKETS.load(Ordering::Relaxed) && !payload.is_empty() {
        println!("[S→C] 0x{:02X} | {}", payload[0], to_hex_upper(payload));
    }
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

/// Per-stream reassembly state for fragmented inbound packets.
///
/// The client splits packets larger than its 8 192-byte send buffer across
/// multiple batches (`SendQueue$$Write`), keyed by stream/queue id. One
/// `FragmentBuffers` per connection.
pub type FragmentBuffers = std::collections::HashMap<u8, Vec<u8>>;

/// Hard cap on a single reassembled packet. The biggest legitimate payload
/// is a 150×150 PNG teleporter screenshot (tens of KB); anything past this
/// is a hostile or corrupt stream and the partial buffer is discarded.
const MAX_REASSEMBLED: usize = 4 * 1024 * 1024;

/// Splits one inbound wire batch into complete packet payloads, reassembling
/// fragments across batches.
///
/// Batch layout (verified via RE of `Connection$$ProcessSendQueues` /
/// `SendQueue$$Write` / `ReceiveQueue$$Read`):
///   u16 total_len; u8 n_records;
///   n_records × ( u8 qid; u8 status; u32 chunk_len; u8 chunk[chunk_len] )
///
/// A batch carries up to one record per send queue (SUPER_HIGH/HIGH/DEFAULT/
/// LOW), so small packets from one queue can share a batch with a fragment
/// of a large packet from another. Status semantics match `craft_frame`:
/// 3 = complete packet, 2 = first fragment, 1 = middle, 0 = last.
///
/// Completed packets are pushed to `out` re-framed with a synthetic 9-byte
/// header (same shape as `craft_batch`), so downstream parsers keep using
/// their `data[9]` opcode / `data[10..]` field offsets unchanged.
pub fn split_batch_into(batch: &[u8], partial: &mut FragmentBuffers, out: &mut Vec<Vec<u8>>) {
    if batch.len() < 3 {
        return;
    }
    let n_records = batch[2] as usize;
    let mut off = 3usize;

    for _ in 0..n_records {
        if off + 6 > batch.len() {
            return; // truncated record header — drop the rest of the batch
        }
        let qid = batch[off];
        let status = batch[off + 1];
        let len = u32::from_le_bytes([
            batch[off + 2], batch[off + 3], batch[off + 4], batch[off + 5],
        ]) as usize;
        off += 6;
        if len > batch.len() - off {
            return; // chunk runs past the batch — drop
        }
        let chunk = &batch[off..off + len];
        off += len;

        let payload: Vec<u8> = match status {
            3 => chunk.to_vec(),
            2 => {
                partial.insert(qid, chunk.to_vec());
                continue;
            }
            1 => {
                if let Some(buf) = partial.get_mut(&qid) {
                    buf.extend_from_slice(chunk);
                    if buf.len() > MAX_REASSEMBLED {
                        partial.remove(&qid);
                    }
                }
                continue;
            }
            0 => {
                let mut buf = match partial.remove(&qid) {
                    Some(b) => b,
                    None => continue, // last fragment without a start — drop
                };
                buf.extend_from_slice(chunk);
                if buf.len() > MAX_REASSEMBLED {
                    continue;
                }
                buf
            }
            _ => continue, // unknown status — skip record
        };
        if payload.is_empty() {
            continue;
        }

        let mut frame = Vec::with_capacity(9 + payload.len());
        // total_len wraps for reassembled packets >64 KB; no parser reads it
        // (they use the buffer length), it only keeps the frame shape.
        frame.extend_from_slice(&((9 + payload.len()) as u16).to_le_bytes());
        frame.push(0x01);
        frame.push(qid);
        frame.push(0x03);
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        out.push(frame);
    }
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

#[cfg(test)]
mod batch_tests {
    use super::*;

    /// Builds one wire batch from (qid, status, chunk) records, the way the
    /// client's ProcessSendQueues does.
    fn batch(records: &[(u8, u8, &[u8])]) -> Vec<u8> {
        let mut body = vec![records.len() as u8];
        for (qid, status, chunk) in records {
            body.push(*qid);
            body.push(*status);
            body.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
            body.extend_from_slice(chunk);
        }
        let mut out = ((body.len() + 2) as u16).to_le_bytes().to_vec();
        out.extend_from_slice(&body);
        out
    }

    fn split(batches: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let mut partial = FragmentBuffers::new();
        let mut out = Vec::new();
        for b in batches {
            split_batch_into(b, &mut partial, &mut out);
        }
        out
    }

    #[test]
    fn complete_packet_passes_through() {
        let frames = split(&[batch(&[(2, 3, &[0x0F, 0xAA])])]);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0][9], 0x0F);            // opcode at data[9]
        assert_eq!(&frames[0][9..], &[0x0F, 0xAA]); // payload intact
    }

    /// A screenshot-sized packet split across batches, with a SUPER_HIGH
    /// record (qid 1) sharing the middle batch — the shape the client
    /// actually produces during a 0x30 upload.
    #[test]
    fn reassembles_fragmented_screenshot() {
        let mut big = vec![0x30u8];
        big.extend((0..20_000u32).map(|i| (i % 251) as u8));

        let frames = split(&[
            batch(&[(2, 2, &big[..8000])]),
            batch(&[(1, 3, &[0x01]), (2, 1, &big[8000..16000])]),
            batch(&[(2, 0, &big[16000..])]),
        ]);

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0][9], 0x01);      // interleaved ping comes out first
        assert_eq!(frames[1][9], 0x30);
        assert_eq!(&frames[1][9..], &big[..]); // screenshot reassembled byte-exact
    }

    #[test]
    fn orphan_final_fragment_is_dropped() {
        // Last-fragment with no first-fragment (e.g. we connected mid-stream).
        let frames = split(&[batch(&[(2, 0, &[1, 2, 3])])]);
        assert!(frames.is_empty());
    }

    #[test]
    fn truncated_record_is_dropped_without_panic() {
        // Record header claims more bytes than the batch holds.
        let mut b = batch(&[(2, 3, &[0x0F])]);
        let fixed = b.len() - 1;
        b[5] = 200; // chunk_len low byte now lies
        b.truncate(fixed);
        assert!(split(&[b]).is_empty());
    }

    /// The second latent bug: two complete packets coalesced into one batch.
    /// The old loop processed only the first record and dropped the rest.
    #[test]
    fn multiple_complete_records_in_one_batch() {
        let frames = split(&[batch(&[
            (0, 3, &[0x01]),             // SUPER_HIGH ping
            (2, 3, &[0x0F, 0xAA, 0xBB]), // DEFAULT packet
        ])]);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0][9], 0x01);
        assert_eq!(&frames[1][9..], &[0x0F, 0xAA, 0xBB]);
    }

    /// Two large packets uploading at once on different streams must
    /// reassemble independently — keying the partial buffers by qid.
    #[test]
    fn concurrent_streams_reassemble_independently() {
        let mut a = vec![0x30u8];
        a.extend((0..5000u32).map(|i| (i % 97) as u8));
        let mut b = vec![0x31u8];
        b.extend((0..5000u32).map(|i| (i % 131) as u8));

        // Interleave the two streams across batches (qid 2 vs qid 3).
        let frames = split(&[
            batch(&[(2, 2, &a[..2500])]),
            batch(&[(3, 2, &b[..2500])]),
            batch(&[(2, 0, &a[2500..]), (3, 0, &b[2500..])]),
        ]);

        assert_eq!(frames.len(), 2);
        assert_eq!(&frames[0][9..], &a[..]); // qid 2 finalised first in the last batch
        assert_eq!(&frames[1][9..], &b[..]);
    }

    #[test]
    fn middle_fragment_without_start_is_dropped() {
        // status 1 with no prior status-2 has nowhere to append → no output.
        let frames = split(&[batch(&[(2, 1, &[1, 2, 3])])]);
        assert!(frames.is_empty());
    }

    #[test]
    fn unknown_status_is_skipped() {
        // A bogus status byte (e.g. 7) must not be treated as a packet.
        let frames = split(&[batch(&[(2, 7, &[0x0F])])]);
        assert!(frames.is_empty());
    }

    #[test]
    fn empty_and_tiny_batches_are_ignored() {
        assert!(split(&[batch(&[])]).is_empty());      // n_records = 0
        assert!(split(&[vec![0x02, 0x00]]).is_empty()); // < 3 bytes, no record count
    }

    #[test]
    fn empty_payload_record_produces_nothing() {
        // A complete record with a zero-length chunk has no opcode to dispatch.
        let frames = split(&[batch(&[(2, 3, &[])])]);
        assert!(frames.is_empty());
    }

    #[test]
    fn oversized_reassembly_is_discarded() {
        // A fragment stream exceeding MAX_REASSEMBLED is dropped, not buffered
        // unboundedly. ~1 MB chunks: first(2) + middles(1) push past the 4 MB
        // cap, so the final(0) finds no buffer and yields nothing.
        let chunk = vec![0xABu8; 1_000_000];
        let frames = split(&[
            batch(&[(2, 2, &chunk)]),
            batch(&[(2, 1, &chunk)]),
            batch(&[(2, 1, &chunk)]),
            batch(&[(2, 1, &chunk)]),
            batch(&[(2, 1, &chunk)]),
            batch(&[(2, 0, &chunk)]),
        ]);
        assert!(frames.is_empty());
    }

    // ── Real-capture fixtures ──────────────────────────────────────────────
    //
    // Drop captured client→server bytes here as a hex string (the
    // raw TCP payload, one or more batches concatenated, exactly as logged).
    // `decode_hex` + `split` will exercise the real wire against the splitter.

    #[allow(dead_code)]
    fn decode_hex(s: &str) -> Vec<u8> {
        let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        clean.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let hi = (pair[0] as char).to_digit(16).unwrap() as u8;
                let lo = (pair[1] as char).to_digit(16).unwrap() as u8;
                (hi << 4) | lo
            })
            .collect()
    }

    /// Feeds a hex blob through the splitter the same way handle_client does:
    /// one batch at a time, draining by each batch's u16 total_len, with the
    /// partial buffers persisting across batches.
    #[allow(dead_code)]
    fn split_wire(bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut partial = FragmentBuffers::new();
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos + 3 <= bytes.len() {
            let total = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
            if total < 3 || pos + total > bytes.len() { break; }
            split_batch_into(&bytes[pos..pos + total], &mut partial, &mut out);
            pos += total;
        }
        out
    }

    #[test]
    #[ignore = "fill in with a real client capture"]
    fn real_screenshot_capture_reassembles() {
        // Example: paste the logged hex of a full 0x30 upload here.
        let capture = "";
        let frames = split_wire(&decode_hex(capture));
        assert!(frames.iter().any(|f| f.get(9) == Some(&0x30)),
            "expected a reassembled 0x30 teleporter screenshot in the capture");
    }
}
