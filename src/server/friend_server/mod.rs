// friend_server.rs — friend-list / social server (port configurable).

pub mod packets_client;
pub mod packets_server;
pub mod server_registry;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use rand::distr::{Alphanumeric, SampleString};

use crate::utils::config::Config;
use crate::server::game_server;
use crate::defs::packet::{pack_string, to_hex_upper, Str16};
use crate::defs::state::{SessionConn, SharedState};
use packets_client::ClientPacket;
use server_registry::RegisteredServer;
use packets_server::{
    AcceptFriendOk, AddFriendFail, AddFriendOk, AuthFail, FriendOnline, HeartbeatReply,
    JoinGrantHostClear, JumpToGame, PushAccepted, PushFriendReq, PushRemoved,
    RegisterFail, RegisterOk, RelayJoinReq, RelayPrivateMsg, RemoveFriendOk, ShowPopup,
};

// ── Helpers ────────────────────────────────────────────────────────────────

fn random_token() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 12)
}

/// Strip the second String from a WorldUpdate blob.
///
/// PackWorldString (C→S): `Byte + String₁ + String₂ + Short`
/// UnpackWorldString (read): `Byte + String₁ + Short`
///
/// Returns the 3-field version: `Byte + String₁ + Short`.
fn strip_world_update(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return crate::defs::packet::DEFAULT_WORLD.to_vec();
    }
    let mut off = 0;
    // Byte (world type)
    let wb = data[off..off + 1].to_vec();
    off += 1;
    // String₁ (length-prefixed UTF-16LE)
    if off + 2 > data.len() { return crate::defs::packet::DEFAULT_WORLD.to_vec(); }
    let s1_len = u16::from_le_bytes([data[off], data[off + 1]]) as usize;
    let s1 = data[off..off + 2 + s1_len].to_vec();
    off += 2 + s1_len;
    // String₂ — skip
    if off + 2 > data.len() { return crate::defs::packet::DEFAULT_WORLD.to_vec(); }
    let s2_len = u16::from_le_bytes([data[off], data[off + 1]]) as usize;
    off += 2 + s2_len;
    // Short (port)
    if off + 2 > data.len() { return crate::defs::packet::DEFAULT_WORLD.to_vec(); }
    let wp = data[off..off + 2].to_vec();
    // Repack as 3 fields
    let mut out = wb;
    out.extend_from_slice(&s1);
    out.extend_from_slice(&wp);
    return out;
}

// ── Public server list builders ────────────────────────────────────────────

/// S→C 0x1D — full public server list.
fn build_server_list(servers: &[RegisteredServer]) -> Vec<u8> {
    let count = servers.len().min(255) as u8;
    let mut p = vec![0x1Du8, count];
    for s in servers {
        p.extend(s.to_packet_entry());
    }
    return p;
}

/// Load a PNG from `{icons_dir}/{safe_name}.png`, returning `None` if missing
/// or too large for the wire format.
///
/// Hard limit: the client's `Connection` receive buffer is 8192 bytes (0x2000).
/// The batch envelope (9 bytes) + packet-ID (1) + Str16 name (2 + utf16_bytes)
/// + has_icon (1) + count (2) must all fit within that budget.
/// We conservatively cap at 8000 bytes to leave room for any server name.
fn load_icon(server_name: &str, cfg: &Config) -> Option<Vec<u8>> {
    const MAX_ICON_BYTES: usize = 8000;
    let safe = server_name.replace(['/', '\\', '.', ' '], "_");
    let path = std::path::Path::new(&cfg.icons_dir).join(format!("{}.png", safe));
    match std::fs::read(&path) {
        Ok(b) if b.len() <= MAX_ICON_BYTES => Some(b),
        Ok(b) => {
            eprintln!("[FRIEND] Icon '{}' is {} bytes — exceeds {} byte client buffer limit, ignoring",
                server_name, b.len(), MAX_ICON_BYTES);
            None
        }
        Err(_) => None,
    }
}

// ── Login response builder ─────────────────────────────────────────────────
//
// S→C 0x0B LOGIN_SUCCESS wire layout:
//
//   u8   packet_id       = 0x0B
//   u8   success         = 0x01
//   u16  friend_count
//   for each friend:
//     Str16  username
//     Str16  display
//     u8     is_online
//     if is_online == 1:
//       [world_data]     — 3-field UnpackWorldString blob (variable length)
//     else:
//       u8  0x00         — menu_id=0, no further world fields
//   u16  outbound_pending_count     (requests YOU sent)
//   for each outbound:
//     Str16  username
//     Str16  display
//   u16  inbound_pending_count      (requests sent TO you)
//   for each inbound:
//     Str16  username
//     Str16  display
//   --- trailer (9 bytes) ---
//   i16  N_ToPing          = 0
//   i16  give_gems_on_open = 0   (set to 10 to award gems)
//   u8   show_warning      = 0
//   i16  unknown           = 0
//   i16  N_trophies        = 0

fn build_login_success(username: &str, state: &SharedState) -> Vec<u8> {
    let mut resp = vec![0x0B, 0x01];

    let sessions = state.sessions.read().unwrap();
    let worlds   = state.world_states.read().unwrap();

    // ── Friend list ────────────────────────────────────────────────────────
    let friends = state.db.get_friends(username);
    resp.extend_from_slice(&(friends.len() as u16).to_le_bytes());
    for f_user in &friends {
        let is_on = sessions.contains_key(f_user.as_str());
        let world = worlds.get(f_user.as_str());
        let f_display = state.db.get_display_name(f_user);
        resp.extend_from_slice(&pack_string(f_user));
        resp.extend_from_slice(&pack_string(&f_display));
        resp.push(is_on as u8);
        if is_on {
            let w = world.map(|w| w.as_slice())
                     .unwrap_or(crate::defs::packet::DEFAULT_WORLD);
            resp.extend_from_slice(w);
        } else {
            resp.push(0x00);
        }
    }

    // ── Outbound pending (you sent them a request) ─────────────────────────
    let outbound = state.db.get_pending_outbound(username);
    resp.extend_from_slice(&(outbound.len() as u16).to_le_bytes());
    for u in &outbound {
        let disp = state.db.get_display_name(u);
        resp.extend_from_slice(&pack_string(u));
        resp.extend_from_slice(&pack_string(&disp));
    }

    // ── Inbound pending (they sent you a request) ──────────────────────────
    let inbound = state.db.get_pending_inbound(username);
    resp.extend_from_slice(&(inbound.len() as u16).to_le_bytes());
    for u in &inbound {
        let disp = state.db.get_display_name(u);
        resp.extend_from_slice(&pack_string(u));
        resp.extend_from_slice(&pack_string(&disp));
    }

    // ── Trailer ────────────────────────────────────────────────────────────
    resp.extend_from_slice(crate::defs::packet::LOGIN_SUCCESS_TRAILER);
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
    state:        &Arc<SharedState>,
    cfg:          &Config,
) {
    match packet {

        // ── REGISTER (0x0A) ────────────────────────────────────────────────
        // Creates a new account with a randomly generated token.
        // Rich text tags are stripped from the username before any check so
        // players cannot embed markup in names shown to others.
        ClientPacket::RegisterReq { username } => {
            let username = crate::utils::text::strip_rich_text(&username);
            let username = username.trim().to_string();
            if username.is_empty() {
                conn.send_pkt(
                    &RegisterFail { name: Str16::new("") },
                    "S->C [REG_FAIL]",
                );
                return;
            }
            if state.db.player_exists(&username) {
                conn.send_pkt(
                    &RegisterFail { name: Str16::new(&username) },
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
                // Clear any stale world state from a previous session.
                // The client will send a fresh WorldUpdate (0x2C) shortly.
                state.world_states.write().unwrap().remove(&canonical);

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
        // PushFriendReq uses username FIRST, display SECOND (same order).
        ClientPacket::AddFriend { target: target_raw } => {
            if let Some(ref user) = *current_user {
                // Resolve to canonical stored casing.
                let t = state.db.get_player(&target_raw)
                    .map(|p| p.username)
                    .unwrap_or_else(|| target_raw.clone());

                if t != *user && state.db.add_friend_request(user, &t) {
                    let t_display = state.db.get_display_name(&t);
                    conn.send_pkt(
                        &AddFriendOk {
                            username: Str16::new(&t),
                            display:  Str16::new(&t_display),
                        },
                        "S->C [ADD_OK]",
                    );

                    // Push request notification to target if online.
                    let user_display = state.db.get_display_name(user);
                    let target_conn = state.sessions.read().unwrap()
                        .get(&t)
                        .map(Arc::clone);
                    if let Some(tc) = target_conn {
                        tc.send_pkt(
                            &PushFriendReq {
                                username: Str16::new(user),
                                display:  Str16::new(&user_display),
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
                        worlds.get(&t).cloned()
                            .unwrap_or_else(|| crate::defs::packet::DEFAULT_WORLD.to_vec())
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
                            .unwrap_or_else(|| crate::defs::packet::DEFAULT_WORLD.to_vec());
                        let user_display = state.db.get_display_name(user);

                        // Notify requester they were accepted.
                        tc.send_pkt(
                            &PushAccepted {
                                username:   Str16::new(user),
                                display:    Str16::new(&user_display),
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

                let message = crate::utils::text::strip_rich_text(&message);
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
            }
        }

        // ── JOIN REQUEST (0x2D) ────────────────────────────────────────────
        // Relays the request to the target host. Extra byte is always 0x00
        // per Ghidra analysis — the client value is ignored.
        // If the target is a dummy world bot, auto-accept after 3 seconds.
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
        ClientPacket::JoinGrant { target: target_raw } => {
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

                    // 3. Reuse existing relay session or spawn a new one.
                    //    The map tracks ALL players in relay sessions (hosts + guests).
                    //    If the granting user is already in someone else's world,
                    //    the joiner gets sent to that same session.
                    let room         = user.clone();
                    let user_display = state.db.get_display_name(user);
                    let ip   = if cfg.public_ip.is_empty() { &cfg.host } else { &cfg.public_ip };

                    let existing_port = {
                        let relays = state.active_relay_sessions.read().unwrap();
                        relays.get(user).copied()
                    };

                    if let Some(p) = existing_port {
                        // Granter is already in a relay session — send ONLY the
                        // joiner there.  Do NOT re-jump the granter.
                        use std::net::TcpStream;
                        let alive = TcpStream::connect_timeout(
                            &format!("127.0.0.1:{}", p).parse().unwrap(),
                            std::time::Duration::from_secs(2),
                        ).is_ok();

                        if alive {
                            println!("[FRIEND] {} already in relay on port {}, sending joiner {} there",
                                     user, p, t);
                            state.active_relay_sessions.write().unwrap().insert(t.clone(), p);
                            // Joiner gets their OWN username as token so the
                            // game server can distinguish them from the host.
                            let jump_joiner = JumpToGame {
                                display:       Str16::new(&user_display),
                                token:         Str16::new(&t),
                                host_ip:       Str16::new(ip),
                                mode:          Str16::new(ip),
                                port:          p,
                                password_flag: 0x00,
                            };
                            tc.send_pkt(&jump_joiner, "S->C [JUMP_SIGNAL_JOINER]");
                        } else {
                            // Stale session — clean up and spawn fresh.
                            println!("[FRIEND] Stale relay on port {}, spawning new", p);
                            state.active_relay_sessions.write().unwrap().remove(user);
                            if let Some(np) = game_server::spawn_relay_session(room.clone(), cfg) {
                                state.active_relay_sessions.write().unwrap().insert(user.clone(), np);
                                state.active_relay_sessions.write().unwrap().insert(t.clone(), np);
                                // Host token = host username (matches room_token → host).
                                // Joiner token = joiner username (won't match → guest).
                                let jump_host = JumpToGame {
                                    display:       Str16::new(&user_display),
                                    token:         Str16::new(&room),
                                    host_ip:       Str16::new(ip),
                                    mode:          Str16::new(ip),
                                    port:          np,
                                    password_flag: 0x00,
                                };
                                let jump_joiner = JumpToGame {
                                    display:       Str16::new(&user_display),
                                    token:         Str16::new(&t),
                                    host_ip:       Str16::new(ip),
                                    mode:          Str16::new(ip),
                                    port:          np,
                                    password_flag: 0x00,
                                };
                                tc.send_pkt(&jump_joiner, "S->C [JUMP_SIGNAL_JOINER]");
                                conn.send_pkt(&jump_host, "S->C [JUMP_SIGNAL_HOST]");
                            }
                        }
                    } else {
                        // No existing session — spawn a new relay.
                        if let Some(np) = game_server::spawn_relay_session(room.clone(), cfg) {
                            state.active_relay_sessions.write().unwrap().insert(user.clone(), np);
                            state.active_relay_sessions.write().unwrap().insert(t.clone(), np);
                            // Host token = host username (matches room_token → host).
                            // Joiner token = joiner username (won't match → guest).
                            let jump_host = JumpToGame {
                                display:       Str16::new(&user_display),
                                token:         Str16::new(&room),
                                host_ip:       Str16::new(ip),
                                mode:          Str16::new(ip),
                                port:          np,
                                password_flag: 0x00,
                            };
                            let jump_joiner = JumpToGame {
                                display:       Str16::new(&user_display),
                                token:         Str16::new(&t),
                                host_ip:       Str16::new(ip),
                                mode:          Str16::new(ip),
                                port:          np,
                                password_flag: 0x00,
                            };
                            tc.send_pkt(&jump_joiner, "S->C [JUMP_SIGNAL_JOINER]");
                            conn.send_pkt(&jump_host, "S->C [JUMP_SIGNAL_HOST]");
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
        // The client's PackWorldString sends 4 fields:
        //   Byte + String + String + Short
        // But UnpackWorldString (used when reading world state in login
        // responses and 0x16 notifications) only reads 3 fields:
        //   Byte + String + Short
        // We must strip the second String before storing so the blob can
        // be injected directly into outgoing packets.
        ClientPacket::WorldUpdate { world_data } => {
            if let Some(ref user) = *current_user {
                let stripped = strip_world_update(&world_data);
                state.world_states.write().unwrap()
                    .insert(user.clone(), stripped);
                state.broadcast_status(user, true);
            }
        }

        // ── TRY JOIN SERVER (0x1E) ─────────────────────────────────────────
        // Client wants to join a public server by name.  Look up the server
        // and send JumpToGame immediately — no ping round-trip needed since
        // we have exactly one candidate.
        ClientPacket::TryJoinServer { server_name } => {
            if current_user.is_some() {
                let server = state.public_servers.read().unwrap()
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(&server_name))
                    .cloned();
                if let Some(s) = server {
                    // 0x1E Byte(1) — "connecting" acknowledgment; puts the
                    // client into the connecting state before 0x25 arrives.
                    conn.send(2, &[0x1Eu8, 0x01], "S->C [JOIN_ACK]");
                    let ip = if cfg.public_ip.is_empty() { &cfg.host } else { &cfg.public_ip };
                    conn.send_pkt(
                        &JumpToGame {
                            display:       Str16::new(&s.name),
                            token:         Str16::new(&s.room_token),
                            host_ip:       Str16::new(&s.public_ip),
                            mode:          Str16::new(ip),
                            port:          s.port,
                            password_flag: 0x00,
                        },
                        "S->C [JUMP_TO_GAME_PUBLIC]",
                    );
                }
            }
        }

        // ── REQUEST SERVER ICON (0x1F) ────────────────────────────────────
        // Client requests the PNG icon for a named server.
        // S→C: 0x1F + Str16(server_name) + Byte(has_icon=1)
        //        + Short(byte_count) + byte_count×Byte(raw PNG bytes)
        // If no icon file is found, send has_icon=0 so the client uses its
        // default icon instead of waiting forever.
        ClientPacket::RequestServerIcon { server_name } => {
            if current_user.is_some() {
                // Check registered servers first, then fall back to disk.
                let stored = state.public_servers.read().unwrap()
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(&server_name))
                    .and_then(|s| s.icon_bytes.clone());

                let bytes = stored.or_else(|| load_icon(&server_name, cfg));

                let mut payload = vec![0x1Fu8];
                payload.extend(pack_string(&server_name));
                match bytes {
                    Some(b) => {
                        payload.push(0x01);
                        payload.extend_from_slice(&(b.len() as u16).to_le_bytes());
                        payload.extend_from_slice(&b);
                        conn.send(2, &payload, "S->C [SERVER_ICON]");
                    }
                    None => {
                        payload.push(0x00);
                        conn.send(2, &payload, "S->C [SERVER_ICON_NONE]");
                    }
                }
            }
        }

        // ── REQUEST SERVER LIST (0x1D) ────────────────────────────────────
        // Client explicitly requests the current public server list.
        // Always respond — even with an empty list — or the game hardlocks.
        ClientPacket::RequestServerList => {
            if current_user.is_some() {
                let servers = state.public_servers.read().unwrap();
                conn.send(2, &build_server_list(&servers), "S->C [SERVER_LIST]");
            }
        }

        // ── PING RESULTS (0x20) ────────────────────────────────────────────
        // We don't send a ping dispatch, so this should never arrive — drop.
        ClientPacket::PingResults { .. } => {}
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
    let mut current_user:  Option<String> = None;
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

        // ── Parse all complete packets in this read buffer ─────────────────
        // Multiple packets can arrive in a single TCP segment; process them
        // all.  Each has a u16 total_len at bytes [0..2], which tells us
        // exactly where the next packet starts.
        let mut pos = 0;
        while pos + 10 <= n {
            let pkt_total = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            if pkt_total < 10 || pos + pkt_total > n {
                // Incomplete packet split across reads — stop for now.
                break;
            }
            let pkt = &data[pos..pos + pkt_total];

            let packet = match ClientPacket::parse(pkt) {
                Some(p) => p,
                None => {
                    println!("[FRIEND] Dropped unrecognised packet from {} | {}", addr, to_hex_upper(pkt));
                    pos += pkt_total;
                    continue;
                }
            };

            // Any valid packet resets the heartbeat deadline.
            last_heartbeat = std::time::Instant::now();

            println!("\n[C->S] [{}] {}({}) | {}", packet.id().name(),
                current_user.as_deref().unwrap_or("?"), conn.peer_ip(), to_hex_upper(pkt));

            handle_packet(packet, &conn, &mut current_user, &state, &cfg);
            pos += pkt_total;
        }
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
            state.world_states.write().unwrap().remove(user);
            state.active_relay_sessions.write().unwrap().remove(user);
            state.broadcast_status(user, false);
        } else {
            drop(sessions);
        }
        println!("\n[FRIEND] {} disconnected", user);
    }
}


// ── Server entry point ─────────────────────────────────────────────────────

pub fn run(cfg: &Config, state: Arc<SharedState>) {
    // Start the external game-server registry listener (no-op if unconfigured).
    server_registry::run(cfg, Arc::clone(&state.public_servers));

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
