#![allow(dead_code)]
// packets_client.rs — C→S packet definitions.
//
// `ClientPacket` covers every packet type a client can legitimately send.
// `parse()` is the single entry point: it reads the 9-byte batch envelope,
// gates on the known-ID set, then dispatches to per-packet field parsing.
//
// Adding a new C→S packet
// ────────────────────────
// 1. Add the ID to `PacketId` in structs.rs.
// 2. Add a variant to `ClientPacket`.
// 3. Add a match arm in `parse()`.
// 4. Add the corresponding arm in `id()`.

use std::io::Cursor;

use binrw::{BinRead, BinReaderExt};

use crate::defs::packet::{PacketHeader, Str16};
use crate::defs::structs::PacketId;

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

    /// `0x2B` — host grants a join request relayed through the server.
    /// C→S wire format is just `[target: Str16]` — no status or room fields.
    JoinGrant {
        target: String,
    },

    /// `0x2C` — client broadcasts its current in-game / lobby world state.
    ///
    /// The raw blob uses PackWorldString format (4 fields):
    ///   Byte + String + String + Short
    /// Before storing, the handler must strip the second String to produce
    /// the 3-field UnpackWorldString format that readers expect.
    /// See `strip_world_update`.
    WorldUpdate {
        /// Raw 4-field blob bounded by the batch header's `total_len` field.
        world_data: Vec<u8>,
    },

    /// `0x2D` — request to join another player's active session.
    JoinReq {
        target:     String,
        /// Purpose not fully reversed; relayed verbatim to the host.
        #[allow(dead_code)]
        extra_byte: u8,
    },

    /// `0x1D` — request the current public server list.
    RequestServerList,

    /// `0x1F` — request the icon for a specific public server.
    RequestServerIcon {
        server_name: String,
    },

    /// `0x1E` — request to join a public server by name.
    TryJoinServer {
        server_name: String,
    },

    /// `0x20` — ping results for server selection.
    ///
    /// Sent after the server issues a 0x20 ping-dispatch packet. Each entry
    /// is the key string from the dispatch paired with the measured ping (ms).
    /// The server uses these to pick the best target and respond with 0x25.
    PingResults {
        /// `(key, ping_ms)` pairs — key matches the IP string sent in 0x20.
        entries: Vec<(String, i16)>,
    },

    /// `0x2E` — submit a player report.
    ///
    /// The client sends an i32 count followed by that many UTF-16LE key/value
    /// string pairs.  The server extracts the three meaningful keys and stores
    /// the report in the database.
    SubmitReport {
        /// Lowercase username of the reported player (`"report_username_lower"`).
        reported: String,
        /// Report category string (`"category"`).
        category: String,
        /// Free-text notes (`"additional_notes"`).
        notes:    String,
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

            // ── REQUEST_SERVER_LIST (0x1D) ───────────────────────────────
            // C→S  (no payload fields)
            0x1D => Some(Self::RequestServerList),

            // ── REQUEST_SERVER_ICON (0x1F) ───────────────────────────────
            // C→S  [server_name: Str16]
            0x1F => {
                let server_name = Str16::read(&mut cur).ok()?;
                Some(Self::RequestServerIcon { server_name: server_name.value })
            }

            // ── TRY_JOIN_SERVER (0x1E) ───────────────────────────────────
            // C→S  [server_name: Str16]
            0x1E => {
                let server_name = Str16::read(&mut cur).ok()?;
                Some(Self::TryJoinServer { server_name: server_name.value })
            }

            // ── PING_RESULTS (0x20) ───────────────────────────────────────
            // C→S  [count: i16] [count × (key: Str16, ping_ms: i16)]
            0x20 => {
                let count: i16 = cur.read_le().ok()?;
                let mut entries = Vec::with_capacity(count.max(0) as usize);
                for _ in 0..count.max(0) {
                    let key:     Str16 = Str16::read(&mut cur).ok()?;
                    let ping_ms: i16   = cur.read_le().ok()?;
                    entries.push((key.value, ping_ms));
                }
                Some(Self::PingResults { entries })
            }

            // ── JOIN_GRANT (0x2B) ─────────────────────────────────────────
            // C→S  [target: Str16]
            // The client sends only the target username — no status byte.
            0x2B => {
                let target = Str16::read(&mut cur).ok()?;
                Some(Self::JoinGrant { target: target.value })
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

            // ── SUBMIT_REPORT (0x2E) ──────────────────────────────────────
            // C→S  [count: i32] [count × (key: Str16, value: Str16)] [trailing client context]
            //
            // The client sends a dictionary of report fields.  We extract the
            // three meaningful keys and ignore the trailing friend/seen bytes.
            0x2E => {
                let count: i32 = cur.read_le().ok()?;
                let mut reported = String::new();
                let mut category = String::new();
                let mut notes    = String::new();
                for _ in 0..count.max(0) {
                    let key = Str16::read(&mut cur).ok()?.value;
                    let val = Str16::read(&mut cur).ok()?.value;
                    match key.as_str() {
                        "report_username_lower" => reported = val,
                        "category"              => category = val,
                        "additional_notes"      => notes    = val,
                        _                       => {}
                    }
                }
                if reported.is_empty() { return None; }
                Some(Self::SubmitReport { reported, category, notes })
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
            Self::RequestServerList        => PacketId::ServerList,
            Self::RequestServerIcon { .. } => PacketId::ServerIcon,
            Self::TryJoinServer { .. } => PacketId::TryJoinServer,
            Self::PingResults { .. }   => PacketId::PingResults,
            Self::JoinGrant { .. }    => PacketId::JoinGrant,
            Self::WorldUpdate { .. }  => PacketId::WorldUpdate,
            Self::JoinReq { .. }      => PacketId::JoinReq,
            Self::SubmitReport { .. } => PacketId::SubmitReport,
        }
    }
}
