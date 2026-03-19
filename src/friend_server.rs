// friend_server.rs — friend-list / social server (port configurable).

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use rand::distr::{Alphanumeric, SampleString};

use crate::config::Config;
use crate::game_server;
use crate::packet::{craft_batch, pack_string, to_hex_upper, DEFAULT_WORLD};
use crate::packet::{
    AcceptFriendOk, AddFriendFail, AddFriendOk, AuthFail, FriendOnline, HeartbeatReply,
    JoinGrantHostClear, JumpToGame, PushAccepted, PushFriendReq, PushRemoved,
    RegisterFail, RegisterOk, RelayJoinReq, RelayPrivateMsg, RemoveFriendOk, ShowPopup, Str16,
};
use crate::state::{SessionConn, SharedState};
use crate::packet::ClientPacket;

// ── Helpers ────────────────────────────────────────────────────────────────

fn random_token() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 12)
}

// ── Login response builder ─────────────────────────────────────────────────
//
// The LOGIN_SUCCESS payload is variable-length (friend/pending lists) and
// doesn't map cleanly to a single binrw struct, so we build it manually
// using `pack_string` for the UTF-16LE fields.

fn build_login_success(username: &str, state: &SharedState) -> Vec<u8> {
    let mut resp = vec![0x0B, 0x01]; // packet-ID, success byte

    let sessions = state.sessions.read().unwrap();
    let worlds   = state.world_states.read().unwrap();

    // ── Friend list ────────────────────────────────────────────────────────
    let friends = state.db.get_friends(username);
    resp.extend_from_slice(&(friends.len() as u16).to_le_bytes());
    for f_user in &friends {
        let is_on: u8 = if sessions.contains_key(f_user.as_str()) { 1 } else { 0 };
        resp.extend_from_slice(&pack_string(f_user));
        resp.extend_from_slice(&pack_string(f_user)); // username = display
        resp.push(is_on);
        if is_on == 1 {
            let w = worlds.get(f_user.as_str()).map(|w| w.as_slice()).unwrap_or(DEFAULT_WORLD);
            resp.extend_from_slice(w);
        } else {
            resp.push(0x00); // mandatory offline marker
        }
    }

    // ── Outbound pending requests (status=2: you sent them a request) ─────
    let outbound = state.db.get_pending_outbound(username);
    resp.extend_from_slice(&(outbound.len() as u16).to_le_bytes());
    for u in &outbound {
        resp.extend_from_slice(&pack_string(u));
        resp.extend_from_slice(&pack_string(u));
    }

    // ── Inbound pending requests (status=3: they sent you a request) ──────
    let inbound = state.db.get_pending_inbound(username);
    resp.extend_from_slice(&(inbound.len() as u16).to_le_bytes());
    for u in &inbound {
        resp.extend_from_slice(&pack_string(u));
        resp.extend_from_slice(&pack_string(u));
    }

    // ── Fixed trailer: N_ToPing(0) give_gems(0) warn(0) unk(0) N_trophy(0) ─
    resp.extend_from_slice(crate::packet::LOGIN_SUCCESS_TRAILER);
    resp
}

// ── Packet handler ─────────────────────────────────────────────────────────
//
// Separated from the read loop so the admin terminal can inject packets on
// behalf of a spoofed user via `SessionConn::Sink`.

pub fn handle_packet(
    packet:       ClientPacket,
    conn:         &Arc<SessionConn>,
    current_user: &mut Option<String>,
    state:        &SharedState,
    cfg:          &Config,
) {
    match packet {

        // ── REGISTER (0x0A) ────────────────────────────────────────────────
        // Creates a new account with a randomly generated token.
        ClientPacket::RegisterReq { username } => {
            if state.db.player_exists(&username) {
                conn.send_pkt(
                    &RegisterFail { reason: Str16::new("Taken") },
                    "S->C [REG_FAIL]",
                );
            } else {
                let token = random_token();
                state.db.create_player(&username, &token);
                conn.send_pkt(
                    &RegisterOk {
                        username: Str16::new(&username),
                        display:  Str16::new(&username),
                        token:    Str16::new(&token),
                    },
                    "S->C [REG_OK]",
                );
            }
        }

        // ── LOGIN (0x0B) ───────────────────────────────────────────────────
        // Validates credentials then sends the full friend/pending list.
        ClientPacket::Login { username, token } => {
            let player = state.db.get_player(&username);
            let authed = player.as_ref().map(|p| p.token == token).unwrap_or(false);

            if authed {
                // Use the stored casing as the canonical session key.
                let canonical = player.unwrap().username;
                *current_user = Some(canonical.clone());

                state.sessions.write().unwrap()
                    .insert(canonical.clone(), Arc::clone(conn));
                state.world_states.write().unwrap()
                    .entry(canonical.clone())
                    .or_insert_with(|| DEFAULT_WORLD.to_vec());

                state.broadcast_status(&canonical, true);

                let resp = build_login_success(&canonical, state);
                conn.send(2, &resp, "S->C [LOGIN_SUCCESS]");
            } else {
                conn.send_pkt(&AuthFail, "S->C [AUTH_FAIL]");
            }
        }

        // ── HEARTBEAT (0x0F) ───────────────────────────────────────────────
        ClientPacket::Heartbeat => {
            conn.send_pkt(&HeartbeatReply, "S->C [HB]");
        }

        // ── ADD FRIEND (0x10) ──────────────────────────────────────────────
        // Sends a friend request. On success, echoes confirmation to the
        // sender and pushes an incoming-request notification to the target
        // if they are online.
        //
        // Response quirk: AddFriendOk uses display FIRST, username SECOND.
        // PushFriendReq uses username FIRST, display SECOND.
        ClientPacket::AddFriend { target: target_raw } => {
            if let Some(ref user) = *current_user {
                // Resolve to canonical stored casing.
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());

                if t != *user && state.db.add_friend_request(user, &t) {
                    conn.send_pkt(
                        &AddFriendOk {
                            display:  Str16::new(&t),
                            username: Str16::new(&t),
                        },
                        "S->C [ADD_OK]",
                    );

                    // Push request notification to target if online.
                    let target_conn = state.sessions.read().unwrap()
                        .get(&t)
                        .map(Arc::clone);
                    if let Some(tc) = target_conn {
                        tc.send_pkt(
                            &PushFriendReq {
                                username: Str16::new(user),
                                display:  Str16::new(user),
                            },
                            "S->C [PUSH_REQ]",
                        );
                    }
                } else {
                    conn.send_pkt(
                        &AddFriendFail { target: Str16::new(&target_raw) },
                        "S->C [ADD_FAIL]",
                    );
                }
            }
        }

        // ── ACCEPT FRIEND (0x12) ───────────────────────────────────────────
        // Accepts a pending inbound request. Sends confirmation to the
        // acceptor, a push-accepted notification to the requester (if
        // online), and FR_ONLINE syncs to both parties.
        ClientPacket::AcceptFriend { target: target_raw } => {
            if let Some(ref user) = *current_user {
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());

                if state.db.accept_friend(user, &t) {
                    let sessions = state.sessions.read().unwrap();
                    let worlds   = state.world_states.read().unwrap();
                    let is_on_t  = sessions.contains_key(t.as_str());

                    // Confirm to acceptor.
                    let world_t = if is_on_t {
                        worlds.get(&t).cloned().unwrap_or_else(|| DEFAULT_WORLD.to_vec())
                    } else {
                        vec![0x00]
                    };
                    conn.send_pkt(
                        &AcceptFriendOk {
                            target:     Str16::new(&t),
                            is_online:  is_on_t as u8,
                            world_data: world_t.clone(),
                        },
                        "S->C [ACCEPT_OK]",
                    );

                    if let Some(tc) = sessions.get(&t).map(Arc::clone) {
                        let world_u = worlds.get(user.as_str())
                            .cloned()
                            .unwrap_or_else(|| DEFAULT_WORLD.to_vec());

                        // Notify requester they were accepted.
                        tc.send_pkt(
                            &PushAccepted {
                                username:   Str16::new(user),
                                display:    Str16::new(user),
                                world_data: world_u.clone(),
                            },
                            "S->C [PUSH_ACCEPTED]",
                        );

                        // Sync online presence both ways.
                        tc.send_pkt(
                            &FriendOnline {
                                username:   Str16::new(user),
                                world_data: world_u,
                            },
                            "S->C [SYNC_ONLINE]",
                        );
                    }

                    // Sync target's presence to acceptor (if target online).
                    if is_on_t {
                        conn.send_pkt(
                            &FriendOnline {
                                username:   Str16::new(&t),
                                world_data: world_t,
                            },
                            "S->C [SYNC_ONLINE]",
                        );
                    }
                } else {
                    // No pending request found (already accepted, or never existed).
                    // Send AddFriendFail so the client unblocks and cleans up its UI.
                    conn.send_pkt(
                        &AddFriendFail { target: Str16::new(&target_raw) },
                        "S->C [ACCEPT_FAIL]",
                    );
                }
            }
        }

        // ── REMOVE FRIEND (0x18) ───────────────────────────────────────────
        ClientPacket::RemoveFriend { target: target_raw } => {
            if let Some(ref user) = *current_user {
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());
                state.db.remove_friend(user, &t);

                conn.send_pkt(
                    &RemoveFriendOk { target: Str16::new(&t) },
                    "S->C [REMOVE_OK]",
                );

                // Push removal notification to the other party if online.
                let target_conn = state.sessions.read().unwrap()
                    .get(&t)
                    .map(Arc::clone);
                if let Some(tc) = target_conn {
                    tc.send_pkt(
                        &PushRemoved { username: Str16::new(user) },
                        "S->C [PUSH_REMOVED]",
                    );
                }
            }
        }

        // ── PRIVATE MESSAGE (0x1A) ─────────────────────────────────────────
        // Relays the message to the target and forwards a CHAT_RECV event
        // to the admin event listener.
        ClientPacket::PrivateMsg { target: target_raw, message } => {
            if let Some(ref user) = *current_user {
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());

                let target_conn = state.sessions.read().unwrap()
                    .get(&t)
                    .map(Arc::clone);
                if let Some(tc) = target_conn {
                    tc.send_pkt(
                        &RelayPrivateMsg {
                            from:    Str16::new(user),
                            message: Str16::new(&message),
                        },
                        "S->C [RELAY_PM]",
                    );
                }
                notify_chat_event(user, &t, &message);
            }
        }

        // ── JOIN REQUEST (0x2D) ────────────────────────────────────────────
        // Relays the request to the target host. Extra byte is always 0x00
        // per Ghidra analysis — the client value is ignored.
        ClientPacket::JoinReq { target: target_raw, .. } => {
            if let Some(ref user) = *current_user {
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());
                let target_conn = state.sessions.read().unwrap()
                    .get(&t)
                    .map(Arc::clone);
                if let Some(tc) = target_conn {
                    tc.send_pkt(
                        &RelayJoinReq {
                            from:       Str16::new(user),
                            extra_byte: 0x00,
                        },
                        "S->C [RELAY_JOIN_REQ]",
                    );
                }
            }
        }

        // ── JOIN GRANT (0x2B) ──────────────────────────────────────────────
        // Ghidra shows the client reads 0x2B as an empty payload — the
        // status/room fields are never parsed by the game on receipt.
        // Correct sequence (confirmed by intercept analysis):
        //   1. Send empty 0x2B to HOST  → clears the "Allowing…" popup.
        //   2. Send empty 0x2B to JOINER → unfreezes the joiner's UI.
        //   3. Send 0x25 JUMP to JOINER  → triggers the P2P handoff.
        ClientPacket::JoinGrant { target: target_raw, status, room_name } => {
            if let Some(ref user) = *current_user {
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());

                // 1. Unfreeze the host.
                conn.send_pkt(&JoinGrantHostClear, "S->C [ECHO_UNFREEZE_HOST]");

                let target_conn = state.sessions.read().unwrap()
                    .get(&t)
                    .map(Arc::clone);

                if let Some(tc) = target_conn {
                    // 2. Unfreeze the joiner.
                    tc.send_pkt(&JoinGrantHostClear, "S->C [JOINER_UI_UNFREEZE]");

                    // 3. Spawn a server-side game session and redirect both players.
                    if status == 1 {
                        let room = room_name.unwrap_or_else(|| user.clone());
                        let ip   = if cfg.public_ip.is_empty() { &cfg.host } else { &cfg.public_ip };

                        if let Some(port) = game_server::spawn_session(room.clone(), cfg) {
                            let jump = JumpToGame {
                                display:       Str16::new(user),
                                token:         Str16::new(&room),
                                host_ip:       Str16::new(ip),
                                mode:          Str16::new("P2P"),
                                port,
                                password_flag: 0x00,
                            };
                            tc.send_pkt(&jump, "S->C [JUMP_SIGNAL_JOINER]");
                            conn.send_pkt(&jump, "S->C [JUMP_SIGNAL_HOST]");
                        }
                    }
                }
            }
        }

        // ── SUBMIT REPORT (0x2E) ───────────────────────────────────────────
        // Stores a player report.  The reason is the category with any
        // additional notes appended.
        ClientPacket::SubmitReport { reported, category, notes } => {
            if let Some(ref user) = *current_user {
                let reason = if notes.is_empty() {
                    category
                } else {
                    format!("{}: {}", category, notes)
                };
                state.db.add_report(user, &reported, &reason);
                println!("[FRIEND] Report filed: {} reported {} ({})", user, reported, reason);
                conn.send_pkt(&ShowPopup, "S->C [REPORT_ACK]");
            }
        }

        // ── WORLD UPDATE (0x2C) ────────────────────────────────────────────
        // Stores the client's current world state and broadcasts FR_ONLINE
        // to all online friends (so their location text updates live).
        ClientPacket::WorldUpdate { world_data } => {
            if let Some(ref user) = *current_user {
                state.world_states.write().unwrap()
                    .insert(user.clone(), world_data);
                state.broadcast_status(user, true);
            }
        }
    }
}

// ── Per-client handler ─────────────────────────────────────────────────────

fn handle_client(stream: TcpStream, addr: std::net::SocketAddr, state: Arc<SharedState>, cfg: Config) {
    const HEARTBEAT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
    const READ_POLL:         std::time::Duration = std::time::Duration::from_secs(1);

    let peer_ip   = addr.ip().to_string();
    let read_copy = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => { eprintln!("[FRIEND] try_clone failed for {}: {}", addr, e); return; }
    };

    // Wake up every second so we can check the heartbeat deadline even when
    // the socket is quiet.  WouldBlock / TimedOut are handled explicitly below.
    if let Err(e) = read_copy.set_read_timeout(Some(READ_POLL)) {
        eprintln!("[FRIEND] set_read_timeout failed for {}: {}", addr, e);
    }

    let conn = SessionConn::new_real(stream, peer_ip);
    let mut rd = read_copy;
    let mut current_user: Option<String> = None;
    let mut last_heartbeat = std::time::Instant::now();

    println!("\n[FRIEND] New connection: {}", addr);

    let mut buf = [0u8; 8192];
    loop {
        let n = match rd.read(&mut buf) {
            Ok(0) => break, // clean EOF
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                   || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Read poll fired with no data — check the heartbeat deadline.
                if last_heartbeat.elapsed() >= HEARTBEAT_TIMEOUT {
                    println!("[FRIEND] {} timed out (no heartbeat for {}s)",
                        current_user.as_deref().unwrap_or(&addr.to_string()),
                        HEARTBEAT_TIMEOUT.as_secs());
                    break;
                }
                continue;
            }
            Err(_) => break, // real I/O error
        };
        let data = &buf[..n];

        // ── Handshake probe ────────────────────────────────────────────────
        if data.starts_with(&[0x66]) {
            conn.send(0, &[0x09, 0x01], "S->C [HANDSHAKE]");
            continue;
        }

        // ── Parse — drops unknown / malformed packets ──────────────────────
        let packet = match ClientPacket::parse(data) {
            Some(p) => p,
            None => {
                println!("[FRIEND] Dropped unrecognised packet from {} | {}", addr, to_hex_upper(data));
                continue;
            }
        };

        // Any valid packet resets the heartbeat deadline.
        last_heartbeat = std::time::Instant::now();

        println!("\n[CLIENT -> FRIEND] [{}] | {}", packet.id().name(), to_hex_upper(data));

        handle_packet(packet, &conn, &mut current_user, &state, &cfg);
    }

    // ── Cleanup on disconnect ──────────────────────────────────────────────
    // Only remove the session entry if it still points to *this* connection.
    // A kicked client may reconnect before this cleanup runs; in that case a
    // new session has already been inserted and we must not evict it.
    if let Some(ref user) = current_user {
        let mut sessions = state.sessions.write().unwrap();
        let is_ours = sessions.get(user)
            .map(|s| Arc::ptr_eq(s, &conn))
            .unwrap_or(false);
        if is_ours {
            sessions.remove(user);
            drop(sessions);
            state.broadcast_status(user, false);
        } else {
            drop(sessions);
        }
        println!("\n[FRIEND] {} disconnected", user);
    }
}

// ── Chat event notification ────────────────────────────────────────────────

/// Fires a `CHAT_RECV|sender|target|msg` event at the admin event listener.
/// Best-effort; failures are silently ignored so PM delivery is never blocked.
fn notify_chat_event(sender: &str, target: &str, msg: &str) {
    use std::io::Write;
    use std::net::TcpStream;
    const EVENT_PORT: u16 = 7005;
    if let Ok(mut s) = TcpStream::connect(("127.0.0.1", EVENT_PORT)) {
        let _ = s.write_all(format!("CHAT_RECV|{}|{}|{}", sender, target, msg).as_bytes());
    }
}

// ── Server entry point ─────────────────────────────────────────────────────

pub fn run(cfg: &Config, state: Arc<SharedState>) {
    let addr = format!("{}:{}", cfg.host, cfg.friend_port);
    let listener = TcpListener::bind(&addr)
        .unwrap_or_else(|e| panic!("Failed to bind friend server to {}: {}", addr, e));
    println!("[FRIEND] FRIEND server listening on {} ...", addr);

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let peer = stream.peer_addr()
                    .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                let state = Arc::clone(&state);
                let cfg   = cfg.clone();
                std::thread::spawn(move || handle_client(stream, peer, state, cfg));
            }
            Err(e) => eprintln!("[FRIEND] accept error: {}", e),
        }
    }
}
