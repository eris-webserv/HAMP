// Wire primitive tests — verified against the IDA decompilation of Packet.cs.
//
// Wire format (from Packet__GetString / Packet__PutString):
//   GetByte:   1 byte
//   GetShort:  BitConverter.ToInt16 → 2-byte LE i16
//   GetLong:   BitConverter.ToInt32 → 4-byte LE i32
//   GetString: [i16 LE byte_count][UTF-16LE bytes]
//   PutString: same; null/empty string → [00 00]

use crate::defs::packet::{craft_batch, pack_string, unpack_string};

// ── pack_string ────────────────────────────────────────────────────────────

#[test]
fn pack_string_empty() {
    // Empty string → 2-byte zero length, no payload bytes.
    assert_eq!(pack_string(""), &[0x00, 0x00]);
}

#[test]
fn pack_string_ascii() {
    // "hi" → UTF-16LE: h=0068, i=0069 → 4 bytes → length prefix [04 00]
    assert_eq!(pack_string("hi"), &[0x04, 0x00, 0x68, 0x00, 0x69, 0x00]);
}

#[test]
fn pack_string_single_ascii() {
    assert_eq!(pack_string("A"), &[0x02, 0x00, 0x41, 0x00]);
}

#[test]
fn pack_string_non_ascii() {
    // '€' = U+20AC → UTF-16LE bytes [AC 20]
    assert_eq!(pack_string("€"), &[0x02, 0x00, 0xAC, 0x20]);
}

// ── unpack_string ──────────────────────────────────────────────────────────

#[test]
fn unpack_string_empty() {
    let (s, off) = unpack_string(&[0x00, 0x00], 0);
    assert_eq!(s, "");
    assert_eq!(off, 2);
}

#[test]
fn unpack_string_ascii() {
    let (s, off) = unpack_string(&[0x04, 0x00, 0x68, 0x00, 0x69, 0x00], 0);
    assert_eq!(s, "hi");
    assert_eq!(off, 6);
}

#[test]
fn unpack_string_with_offset() {
    // Byte at 0 is some header byte; string starts at 1.
    let data: &[u8] = &[0xFF, 0x04, 0x00, 0x68, 0x00, 0x69, 0x00];
    let (s, off) = unpack_string(data, 1);
    assert_eq!(s, "hi");
    assert_eq!(off, 7);
}

#[test]
fn unpack_string_truncated_length() {
    // Only 1 byte — can't read the 2-byte length prefix.
    let (s, off) = unpack_string(&[0x04], 0);
    assert_eq!(s, "");
    assert_eq!(off, 0); // unchanged on failure
}

#[test]
fn unpack_string_truncated_body() {
    // Length says 4 bytes but only 2 are present.
    let (s, off) = unpack_string(&[0x04, 0x00, 0x68, 0x00], 0);
    assert_eq!(s, "");
    assert_eq!(off, 0);
}

// ── roundtrip ──────────────────────────────────────────────────────────────

#[test]
fn pack_unpack_roundtrip() {
    for s in &["", "hello", "world name", "€uro", "player_1"] {
        let packed = pack_string(s);
        let (result, end) = unpack_string(&packed, 0);
        assert_eq!(&result, s, "roundtrip failed for {:?}", s);
        assert_eq!(end, packed.len());
    }
}

#[test]
fn unpack_two_consecutive_strings() {
    // Simulate reading two strings back-to-back (common in packet payloads).
    let mut data = Vec::new();
    data.extend(pack_string("zone1"));
    data.extend(pack_string("player"));
    let (s1, off1) = unpack_string(&data, 0);
    let (s2, _) = unpack_string(&data, off1);
    assert_eq!(s1, "zone1");
    assert_eq!(s2, "player");
}

// ── craft_batch ────────────────────────────────────────────────────────────

#[test]
fn craft_batch_structure() {
    // Minimal payload: single packet-id byte 0x0A.
    let out = craft_batch(2, &[0x0A]);
    // total_len = 9 + 1 = 10 → [0A 00]
    assert_eq!(&out[0..2], &[0x0A, 0x00]);
    assert_eq!(out[2], 0x01);
    assert_eq!(out[3], 0x02); // qid
    assert_eq!(out[4], 0x03);
    // payload_len = 1 → [01 00 00 00]
    assert_eq!(&out[5..9], &[0x01, 0x00, 0x00, 0x00]);
    assert_eq!(out[9], 0x0A);
    assert_eq!(out.len(), 10);
}

#[test]
fn craft_batch_total_len_matches() {
    let payload = b"\x26\x01\x02\x03\x04";
    let out = craft_batch(1, payload);
    let total_len = u16::from_le_bytes([out[0], out[1]]) as usize;
    assert_eq!(total_len, out.len());
}

// ── Packet-dump fixtures ────────────────────────────────────────────────────
//
// Exact wire bytes captured from a live server session (node-sulfur, 2026-04-17).
// Tests here assert our encoding is byte-identical to what the production
// server sends so client regressions fail at compile-time, not in-game.

// S→C heartbeat reply: craft_batch(qid=2, payload=[0x0F])
// Captured: [S->C [HB]] 0A00010203010000000F
#[test]
fn heartbeat_sc_frame_exact() {
    let frame = craft_batch(0x02, &[0x0F]);
    assert_eq!(
        frame,
        &[0x0A, 0x00, 0x01, 0x02, 0x03, 0x01, 0x00, 0x00, 0x00, 0x0F],
        "S→C heartbeat frame doesn't match live capture"
    );
}

// C→S heartbeat uses qid=3, not 2 — verify the frame is otherwise identical.
// Captured: [C->S] [HEARTBEAT] 0A00010303010000000F
#[test]
fn heartbeat_cs_frame_differs_by_qid() {
    let cs = &[0x0A, 0x00, 0x01, 0x03, 0x03, 0x01, 0x00, 0x00, 0x00, 0x0Fu8];
    // total_len, magic, payload_len, packet_id all match S→C; only qid differs
    assert_eq!(cs[3], 0x03, "C→S qid should be 3");
    assert_eq!(cs[9], 0x0F, "C→S heartbeat packet ID should be 0x0F");
    // Our outgoing heartbeat must use qid=2, not 3
    let sc = craft_batch(0x02, &[0x0F]);
    assert_eq!(sc[3], 0x02, "S→C qid must be 2");
    assert_ne!(&sc[..], cs, "S→C and C→S frames must differ");
}
