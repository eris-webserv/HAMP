// structs.rs — reusable types shared across the server.
// This file is yours to own; add fields as the protocol evolves.

use serde::{Deserialize, Serialize};

// ── Protocol packet IDs ────────────────────────────────────────────────────

/// Every known packet type identifier in the protocol.
///
/// Variants marked **C→S** are sent by clients.
/// Variants marked **S→C** are only ever sent by the server.
/// Variants marked **both** travel in both directions (e.g. heartbeat).
///
/// `from_u8` returns `None` for bytes outside this set; those packets are
/// dropped at the framing layer before any handler logic runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketId {
    RegisterReq   = 0x0A, // C→S  register a new account
    Login         = 0x0B, // C→S  authenticate
    AuthFail      = 0x0C, // S→C  bad credentials
    Heartbeat     = 0x0F, // both keepalive ping/pong
    AddFriend     = 0x10, // C→S  send a friend request
    PushReq       = 0x11, // S→C  incoming friend request notification
    AcceptFriend  = 0x12, // C→S  accept a pending inbound request
    PushAccepted  = 0x13, // S→C  your outbound request was accepted
    PushRemoved   = 0x15, // S→C  you were removed from someone's list
    FrOnline      = 0x16, // S→C  a friend came online / world state changed
    FrOffline     = 0x17, // S→C  a friend went offline
    RemoveFriend  = 0x18, // C→S  remove a player from your list
    PrivateMsg    = 0x1A, // both private chat message
    JumpToGame    = 0x25, // S→C  P2P handoff after join grant
    JoinGrant     = 0x2B, // C→S  host grants or denies a join request
    WorldUpdate   = 0x2C, // C→S  client broadcasts its world/lobby state
    TryJoinServer = 0x1E, // C→S  request to join a public server by name
    PingResults   = 0x20, // C→S  ping results for server selection (after 0x20 S→C)
    JoinReq       = 0x2D, // C→S  request to join another player's session
    SubmitReport  = 0x2E, // C→S  submit a player report (also S→C: show popup)
    ShowWarning   = 0x2F, // S→C  stop it
    GiveGems      = 0x34, // S→C  the server gave you gems
}

impl PacketId {
    pub fn from_u8(b: u8) -> Option<Self> {
        use PacketId::*;
        Some(match b {
            0x0A => RegisterReq,
            0x0B => Login,
            0x0C => AuthFail,
            0x0F => Heartbeat,
            0x10 => AddFriend,
            0x11 => PushReq,
            0x12 => AcceptFriend,
            0x13 => PushAccepted,
            0x15 => PushRemoved,
            0x16 => FrOnline,
            0x17 => FrOffline,
            0x18 => RemoveFriend,
            0x1A => PrivateMsg,
            0x1E => TryJoinServer,
            0x20 => PingResults,
            0x25 => JumpToGame,
            0x2B => JoinGrant,
            0x2C => WorldUpdate,
            0x2D => JoinReq,
            0x2E => SubmitReport,
            0x2F => ShowWarning,
            0x34 => GiveGems,
            _    => return None,
        })
    }

    pub fn name(self) -> &'static str {
        use PacketId::*;
        match self {
            RegisterReq  => "REGISTER_REQ",
            Login        => "LOGIN",
            AuthFail     => "AUTH_FAIL",
            Heartbeat    => "HEARTBEAT",
            AddFriend    => "ADD_FRIEND",
            PushReq      => "PUSH_REQ",
            AcceptFriend => "ACCEPT_FRIEND",
            PushAccepted => "PUSH_ACCEPTED",
            PushRemoved  => "PUSH_REMOVED",
            FrOnline     => "FR_ONLINE",
            FrOffline    => "FR_OFFLINE",
            RemoveFriend => "REMOVE_FRIEND",
            PrivateMsg   => "PRIVATE_MSG",
            TryJoinServer => "TRY_JOIN_SERVER",
            PingResults   => "PING_RESULTS",
            JumpToGame   => "JUMP_TO_GAME",
            JoinGrant    => "JOIN_GRANT",
            WorldUpdate  => "WORLD_UPDATE",
            JoinReq      => "JOIN_REQ",
            SubmitReport => "SUBMIT_REPORT",
            ShowWarning  => "SHOW_WARNING",
            GiveGems     => "GIVE_GEMS"
        }
    }
}

// ── Persistent data types ──────────────────────────────────────────────────

/// A single player-report entry, appended to `reports.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Report {
    pub timestamp: String,
    pub reporter:  String,
    pub reported:  String,
    pub reason:    String,
}
