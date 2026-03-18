// state.rs — shared runtime state and broadcast helpers.

use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};

use crate::db::Db;
use crate::packet::{craft_batch, to_hex_upper, DEFAULT_WORLD};
use crate::packet::{FriendOffline, FriendOnline, ServerPacket, Str16};

// ── Session connection ─────────────────────────────────────────────────────

/// A tracked client connection.
///
/// `Dummy` sessions are created by `sim_login`; they appear online to friends
/// but silently discard every outbound packet.
pub enum SessionConn {
    Real {
        stream:  Mutex<TcpStream>,
        /// IP address string of the remote client — used in JumpToGame packets.
        peer_ip: String,
    },
    Dummy,
}

impl SessionConn {
    pub fn new_real(stream: TcpStream, peer_ip: String) -> Arc<Self> {
        Arc::new(Self::Real {
            stream: Mutex::new(stream),
            peer_ip,
        })
    }

    /// Wraps `payload` in a batch frame, logs it, and writes it to the stream.
    pub fn send(&self, qid: u8, payload: &[u8], label: &str) {
        let batch = craft_batch(qid, payload);
        println!("[{}] | {}", label, to_hex_upper(&batch));
        if let Self::Real { stream, .. } = self {
            if let Ok(mut s) = stream.lock() {
                let _ = s.write_all(&batch);
            }
        }
    }

    /// Convenience: serialises `pkt` via `ServerPacket::to_payload` and sends.
    pub fn send_pkt<P: ServerPacket>(&self, pkt: &P, label: &str) {
        self.send(2, &pkt.to_payload(), label);
    }

    /// Returns the remote IP, or `"127.0.0.1"` for dummy sessions.
    pub fn peer_ip(&self) -> &str {
        match self {
            Self::Real { peer_ip, .. } => peer_ip,
            Self::Dummy               => "127.0.0.1",
        }
    }

    pub fn is_dummy(&self) -> bool {
        matches!(self, Self::Dummy)
    }
}

// ── Shared state ───────────────────────────────────────────────────────────

/// All mutable state shared across connection-handler threads.
pub struct SharedState {
    /// Maps lowercase username → active connection.
    pub sessions:    RwLock<HashMap<String, Arc<SessionConn>>>,
    /// Maps lowercase username → last-known world-state blob.
    pub world_states: RwLock<HashMap<String, Vec<u8>>>,
    /// The database — shared with every handler thread.
    pub db: Arc<Db>,
}

impl SharedState {
    pub fn new(db: Arc<Db>) -> Arc<Self> {
        Arc::new(Self {
            sessions:     RwLock::new(HashMap::new()),
            world_states: RwLock::new(HashMap::new()),
            db,
        })
    }

    // ── Broadcast helpers ──────────────────────────────────────────────────

    /// Sends `FR_ONLINE` (with current world state) or `FR_OFFLINE` to every
    /// online friend of `username`.
    pub fn broadcast_status(&self, username: &str, online: bool) {
        let friends = self.db.get_friends(username);

        let payload: Vec<u8> = if online {
            let worlds = self.world_states.read().unwrap();
            let world  = worlds.get(username).map(|w| w.as_slice()).unwrap_or(DEFAULT_WORLD);
            FriendOnline {
                username:   Str16::new(username),
                world_data: world.to_vec(),
            }.to_payload()
        } else {
            FriendOffline { username: Str16::new(username) }.to_payload()
        };

        let label    = if online { "S->C [BROADCAST_ONLINE]" } else { "S->C [BROADCAST_OFFLINE]" };
        let sessions = self.sessions.read().unwrap();
        for (friend, _) in &friends {
            if let Some(conn) = sessions.get(friend.as_str()) {
                conn.send(2, &payload, label);
            }
        }
    }

    // ── Social graph — convenience wrappers around Db ──────────────────────

    /// Delegates to the appropriate `Db` method and returns `true` on success.
    ///
    /// | `action`   | effect                                              |
    /// |------------|-----------------------------------------------------|
    /// | `"add"`    | Friend request from `user_a` to `user_b`.          |
    /// | `"accept"` | Accept `user_b`'s pending request on `user_a`.     |
    /// | `"remove"` | Remove all ties between the two users.              |
    pub fn perform_social_action(&self, action: &str, user_a: &str, user_b: &str) -> bool {
        match action {
            "add"    => self.db.add_friend_request(user_a, user_b),
            "accept" => self.db.accept_friend(user_a, user_b),
            "remove" => { self.db.remove_friend(user_a, user_b); true }
            _        => false,
        }
    }
}
