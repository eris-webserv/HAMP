// state.rs — shared runtime state and broadcast helpers.

use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};

use crate::utils::db::Db;
use crate::defs::packet::{Str16, craft_batch, to_hex_upper};
use crate::defs::packet::ServerPacket;
use crate::server::friend_server::packets_server::{FriendOffline, FriendOnline};
use crate::server::friend_server::server_registry::RegisteredServer;
use crate::server::game_server::dummy_world::DummyWorld;

// ── Session connection ─────────────────────────────────────────────────────

/// A tracked client connection.
///
/// `Real` wraps an actual TCP socket.
/// `Sink` captures outgoing bytes in memory (used by the admin spoof command).
pub enum SessionConn {
    Real {
        stream:  Mutex<TcpStream>,
        peer_ip: String,
    },
    Sink {
        buf:     Mutex<Vec<u8>>,
        peer_ip: String,
    },
}

impl SessionConn {
    pub fn new_real(stream: TcpStream, peer_ip: String) -> Arc<Self> {
        Arc::new(Self::Real {
            stream: Mutex::new(stream),
            peer_ip,
        })
    }

    /// Creates a sink connection that captures all sent bytes instead of
    /// writing to a socket.  Used by the admin `spoof` command.
    pub fn new_sink(peer_ip: String) -> Arc<Self> {
        Arc::new(Self::Sink {
            buf: Mutex::new(Vec::new()),
            peer_ip,
        })
    }

    /// Wraps `payload` in a batch frame, logs it, and writes it to the stream
    /// (or appends it to the sink buffer).
    pub fn send(&self, qid: u8, payload: &[u8], label: &str) {
        let batch = craft_batch(qid, payload);
        println!("[{}] -> {} | {}", label, self.peer_ip(), to_hex_upper(&batch));
        match self {
            Self::Real { stream, .. } => {
                if let Ok(mut s) = stream.lock() {
                    let _ = s.write_all(&batch);
                }
            }
            Self::Sink { buf, .. } => {
                buf.lock().unwrap().extend_from_slice(&batch);
            }
        }
    }

    /// Convenience: serialises `pkt` via `ServerPacket::to_payload` and sends.
    pub fn send_pkt<P: ServerPacket>(&self, pkt: &P, label: &str) {
        self.send(2, &pkt.to_payload(), label);
    }

    /// Returns the remote IP (or the spoof label for sink connections).
    pub fn peer_ip(&self) -> &str {
        match self {
            Self::Real { peer_ip, .. } | Self::Sink { peer_ip, .. } => peer_ip,
        }
    }

    /// Drains and returns all bytes captured by a `Sink` connection.
    /// Always returns an empty `Vec` for `Real` connections.
    pub fn drain_sink(&self) -> Vec<u8> {
        match self {
            Self::Sink { buf, .. } => std::mem::take(&mut *buf.lock().unwrap()),
            Self::Real { .. }      => Vec::new(),
        }
    }

    /// Shuts down the underlying TCP stream, causing the read loop in
    /// `handle_client` to see an error and break — triggering clean cleanup.
    /// No-op for sink connections.
    pub fn disconnect(&self) {
        match self {
            Self::Real { stream, .. } => {
                if let Ok(s) = stream.lock() {
                    let _ = s.shutdown(std::net::Shutdown::Both);
                }
            }
            Self::Sink { .. } => {}
        }
    }
}

// ── Shared state ───────────────────────────────────────────────────────────

/// All mutable state shared across connection-handler threads.
pub struct SharedState {
    /// Maps username → active connection.
    pub sessions: RwLock<HashMap<String, Arc<SessionConn>>>,
    /// Maps username → last-known world-state blob.
    pub world_states: RwLock<HashMap<String, Vec<u8>>>,
    /// Admin-spawned dummy worlds (managed game sessions with auto-accept).
    pub dummy_worlds: RwLock<HashMap<String, DummyWorld>>,
    /// Maps username → relay session port for every player currently in a relay
    /// session (both hosts and guests).  Used by JoinGrant to reuse an existing
    /// session instead of spawning a new one — e.g. when joining someone who is
    /// already a guest in another player's world.
    pub active_relay_sessions: RwLock<HashMap<String, u16>>,
    /// Live list of externally-registered public game servers.
    /// Updated by the registry thread; read by the friend server handler.
    /// Wrapped in its own Arc so the registry thread can hold a clone.
    pub public_servers: Arc<RwLock<Vec<RegisteredServer>>>,
    /// The database — shared with every handler thread.
    pub db: Arc<Db>,
}

impl SharedState {
    pub fn new(db: Arc<Db>) -> Arc<Self> {
        Arc::new(Self {
            sessions:              RwLock::new(HashMap::new()),
            world_states:          RwLock::new(HashMap::new()),
            dummy_worlds:          RwLock::new(HashMap::new()),
            active_relay_sessions: RwLock::new(HashMap::new()),
            public_servers:        Arc::new(RwLock::new(Vec::new())),
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
            let world = worlds
                .get(username)
                .map(|w| w.as_slice())
                .unwrap_or(crate::defs::packet::DEFAULT_WORLD);
            FriendOnline {
                username:   Str16::new(username),
                world_data: world.to_vec(),
            }
            .to_payload()
        } else {
            FriendOffline {
                username: Str16::new(username),
            }
            .to_payload()
        };

        let label = if online {
            "S->C [BROADCAST_ONLINE]"
        } else {
            "S->C [BROADCAST_OFFLINE]"
        };
        let sessions = self.sessions.read().unwrap();
        for friend in &friends {
            if let Some(conn) = sessions.get(friend.as_str()) {
                conn.send(2, &payload, label);
            }
        }
    }
}
