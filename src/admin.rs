// admin.rs — password-protected remote admin terminal.
//
// Connect with: nc <host> <terminal_port>
//
// After authenticating, send one command per line:
//   help                  — list all commands
//   list                  — show online users and their IPs
//   send <user|*> <hex>   — send a raw payload to one user or everyone
//   kick <user>           — forcibly disconnect a user
//   create <user> <token> — register a new player with an explicit token
//   delete <user>         — delete a player (refused if they are online)
//   spoof <user>          — inject a fake session for <user> (refused if online)
//   unspoof               — tear down the active fake session
//   recv <hex>            — feed a raw client packet to the spoofed user
//   reports               — list all player reports
//   db <base64-sql>       — run a raw SQL query against the database
//   restart               — kill the server process (systemd will restart it)
//
// Adding a new command
// ────────────────────
// 1. Add a match arm in `dispatch`.
// 2. Write the handler as `fn cmd_<name>(session: &mut AdminSession, state: &SharedState, args: &str) -> String`.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use base64::Engine as _;

use crate::config::Config;
use crate::friend_server::handle_packet;
use crate::packet::{craft_batch, to_hex_upper, ClientPacket, PushRemoved, ServerPacket, Str16};
use crate::state::{SessionConn, SharedState};

// ── Per-connection admin state ──────────────────────────────────────────────

struct AdminSession {
    /// The username currently being spoofed, if any.
    spoofed_user: Option<String>,
    /// The Sink connection registered in `SharedState::sessions` for the spoof.
    spoof_conn:   Option<Arc<SessionConn>>,
    /// Server config forwarded to handle_packet.
    cfg:          Config,
    /// Set to true by the `exit` command to break the session loop.
    exit:         bool,
}

impl AdminSession {
    fn new(cfg: Config) -> Self {
        Self { spoofed_user: None, spoof_conn: None, cfg, exit: false }
    }

    /// Tears down any active spoof: removes the session and broadcasts offline.
    fn cleanup_spoof(&mut self, state: &SharedState) {
        if let Some(ref user) = self.spoofed_user.take() {
            state.sessions.write().unwrap().remove(user);
            state.broadcast_status(user, false);
            self.spoof_conn = None;
        }
    }
}

// ── Entry point ────────────────────────────────────────────────────────────

pub fn run_terminal(cfg: Config, state: Arc<SharedState>) {
    if cfg.terminal_password.is_empty() {
        println!("[terminal] Password not set — remote terminal disabled.");
        return;
    }

    let addr = format!("0.0.0.0:{}", cfg.terminal_port);
    let listener = TcpListener::bind(&addr)
        .unwrap_or_else(|e| panic!("Failed to bind terminal to {}: {}", addr, e));
    println!("[terminal] Listening on {} ...", addr);

    for incoming in listener.incoming() {
        if let Ok(stream) = incoming {
            let state       = Arc::clone(&state);
            let password = cfg.terminal_password.clone();
            let cfg      = cfg.clone();
            std::thread::spawn(move || session(stream, password, state, cfg));
        }
    }
}

// ── Session ────────────────────────────────────────────────────────────────

fn session(mut stream: TcpStream, password: String, state: Arc<SharedState>, cfg: Config) {
    let peer = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let _ = stream.write_all(b"Password: ");
    match read_line(&mut stream) {
        Some(ref s) if s.trim() == password => {}
        _ => {
            let _ = stream.write_all(b"Denied.\n");
            println!("[terminal] Failed auth from {}", peer);
            return;
        }
    }

    let _ = stream.write_all(b"Hello. Run 'help' to get the current list of commands.\n");
    println!("[terminal] {} authenticated", peer);

    let mut adm = AdminSession::new(cfg);

    loop {
        let line = match read_line(&mut stream) {
            Some(s) => s,
            None    => break,
        };
        let line = line.trim();
        if line.is_empty() { continue; }

        let (cmd, args) = line.split_once(' ').unwrap_or((line, ""));
        let response = dispatch(cmd, args, &mut adm, &state);
        let _ = stream.write_all(response.as_bytes());
        if adm.exit {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            break;
        }
    }

    // On disconnect, clean up any active spoof.
    adm.cleanup_spoof(&state);
    println!("[terminal] {} disconnected", peer);
}

// ── Dispatch ───────────────────────────────────────────────────────────────

fn dispatch(cmd: &str, args: &str, adm: &mut AdminSession, state: &SharedState) -> String {
    match cmd.to_lowercase().as_str() {
        "list"    => cmd_list(state, args),
        "send"    => cmd_send(state, args),
        "kick"    => cmd_kick(state, args),
        "create"  => cmd_create(state, args),
        "delete"  => cmd_delete(adm, state, args),
        "spoof"   => cmd_spoof(adm, state, args),
        "unspoof" => cmd_unspoof(adm, state),
        "recv"    => cmd_recv(adm, state, args),
        "reports" => cmd_reports(state),
        "db"      => cmd_db(state, args),
        "restart" => cmd_restart(),
        "help"    => cmd_help(),
        "exit"    => cmd_exit(adm),
        other     => format!("[!] Unknown command '{}'\n", other),
    }
}

// ── Command handlers ───────────────────────────────────────────────────────

fn cmd_list(state: &SharedState, _args: &str) -> String {
    let sessions = state.sessions.read().unwrap();
    if sessions.is_empty() {
        return "No users online.\n".to_string();
    }
    let mut out = format!("{} online:\n", sessions.len());
    for (username, conn) in sessions.iter() {
        out.push_str(&format!("  {}  ({})\n", username, conn.peer_ip()));
    }
    out
}

fn cmd_send(state: &SharedState, args: &str) -> String {
    let (target, hex_str) = match args.split_once(' ') {
        Some(p) => p,
        None    => return "[!] Usage: send <user|*> <hex>\n".to_string(),
    };

    let payload = match parse_hex(hex_str) {
        Ok(b)  => b,
        Err(e) => return format!("[!] Bad hex: {}\n", e),
    };

    let sessions = state.sessions.read().unwrap();

    if target == "*" {
        let count = sessions.len();
        for conn in sessions.values() {
            conn.send(2, &payload, "TERMINAL_BROADCAST");
        }
        format!("Sent {} byte(s) to {} user(s).\n", payload.len(), count)
    } else {
        match sessions.get(target) {
            Some(conn) => {
                conn.send(2, &payload, "TERMINAL_SEND");
                format!("Sent {} byte(s) to {}.\n", payload.len(), target)
            }
            None => format!("[!] '{}' is not online.\n", target),
        }
    }
}

fn cmd_kick(state: &SharedState, args: &str) -> String {
    let input = args.trim();
    if input.is_empty() {
        return "[!] Usage: kick <username>\n".to_string();
    }
    // Resolve to canonical casing via DB, then look up in sessions.
    let canonical = state.db.get_player(input).map(|p| p.username).unwrap_or_else(|| input.to_string());
    match state.sessions.read().unwrap().get(&canonical).map(Arc::clone) {
        Some(conn) => { conn.disconnect(); format!("Kicked {}.\n", canonical) }
        None       => format!("[!] '{}' is not online.\n", canonical),
    }
}

fn cmd_create(state: &SharedState, args: &str) -> String {
    let (user_raw, token) = match args.split_once(' ') {
        Some(p) => p,
        None    => return "[!] Usage: create <username> <token>\n".to_string(),
    };
    let user  = user_raw.trim();
    let token = token.trim();
    if user.is_empty() || token.is_empty() {
        return "[!] Usage: create <username> <token>\n".to_string();
    }
    if state.db.create_player(user, token) {
        format!("Created player '{}' with token '{}'.\n", user, token)
    } else {
        format!("[!] Username '{}' is already taken.\n", user)
    }
}

fn cmd_delete(adm: &mut AdminSession, state: &SharedState, args: &str) -> String {
    let input = args.trim();
    let username = state.db.get_player(input).map(|p| p.username).unwrap_or_else(|| input.to_string());
    if username.is_empty() {
        return "[!] Usage: delete <username>\n".to_string();
    }

    // Refuse if this is the currently spoofed user.
    if adm.spoofed_user.as_deref() == Some(username.as_str()) {
        return format!("[!] '{}' is the active spoof — unspoof first.\n", username);
    }

    // Refuse if they are genuinely online.
    if state.sessions.read().unwrap().contains_key(username.as_str()) {
        return format!("[!] '{}' is currently online — kick them first.\n", username);
    }

    // Snapshot friends before deletion so we can notify them.
    let friends = state.db.get_friends(&username);

    if !state.db.delete_player(&username) {
        return format!("[!] Player '{}' not found.\n", username);
    }

    // Notify any online friends that this user was removed from their list.
    let pkt = PushRemoved { username: Str16::new(&username) };
    let sessions = state.sessions.read().unwrap();
    let mut notified = 0usize;
    for friend in &friends {
        if let Some(conn) = sessions.get(friend.as_str()) {
            conn.send_pkt(&pkt, "S->C [PUSH_REMOVED] (delete)");
            notified += 1;
        }
    }
    drop(sessions);

    if notified > 0 {
        format!("Deleted player '{}'. Notified {} online friend(s).\n", username, notified)
    } else {
        format!("Deleted player '{}'.\n", username)
    }
}

fn cmd_spoof(adm: &mut AdminSession, state: &SharedState, args: &str) -> String {
    let input = args.trim();
    let username = state.db.get_player(input).map(|p| p.username).unwrap_or_else(|| input.to_string());
    if username.is_empty() {
        return "[!] Usage: spoof <username>\n".to_string();
    }

    if adm.spoofed_user.is_some() {
        return "[!] Already spoofing — run 'unspoof' first.\n".to_string();
    }

    if !state.db.player_exists(&username) {
        return format!("[!] Player '{}' does not exist.\n", username);
    }

    if state.sessions.read().unwrap().contains_key(username.as_str()) {
        return format!("[!] '{}' is already online.\n", username);
    }

    let label = format!("SPOOF:{}", username);
    let conn  = SessionConn::new_sink(label);

    state.sessions.write().unwrap().insert(username.clone(), Arc::clone(&conn));
    state.broadcast_status(&username, true);

    adm.spoofed_user = Some(username.clone());
    adm.spoof_conn   = Some(conn);

    format!("Now spoofing as '{}'. Use 'recv <hex>' to inject packets.\n", username)
}

fn cmd_unspoof(adm: &mut AdminSession, state: &SharedState) -> String {
    if adm.spoofed_user.is_none() {
        return "[!] No active spoof.\n".to_string();
    }
    let name = adm.spoofed_user.clone().unwrap();
    adm.cleanup_spoof(state);
    format!("Spoof for '{}' removed.\n", name)
}

fn cmd_recv(adm: &mut AdminSession, state: &SharedState, args: &str) -> String {
    let (user, conn) = match (&adm.spoofed_user, &adm.spoof_conn) {
        (Some(u), Some(c)) => (u.clone(), Arc::clone(c)),
        _ => return "[!] No active spoof — run 'spoof <user>' first.\n".to_string(),
    };

    let payload = match parse_hex(args.trim()) {
        Ok(b)  => b,
        Err(e) => return format!("[!] Bad hex: {}\n", e),
    };

    // Wrap in a batch envelope so ClientPacket::parse can read the header.
    let framed = craft_batch(0, &payload);

    let packet = match ClientPacket::parse(&framed) {
        Some(p) => p,
        None    => return "[!] Could not parse packet — unknown or malformed ID.\n".to_string(),
    };

    let mut current_user: Option<String> = Some(user);
    handle_packet(packet, &conn, &mut current_user, state, &adm.cfg);

    let bytes = conn.drain_sink();
    if bytes.is_empty() {
        "OK (no response generated).\n".to_string()
    } else {
        format!("Response: {}\n", to_hex_upper(&bytes))
    }
}

fn cmd_reports(state: &SharedState) -> String {
    let reports = state.db.get_reports();
    if reports.is_empty() {
        return "No reports on file.\n".to_string();
    }
    let mut out = format!("{} report(s):\n", reports.len());
    for r in &reports {
        out.push_str(&format!(
            "  [{}] #{} {} reported {} — {}\n",
            r.timestamp, r.id, r.reporter, r.reported, r.reason
        ));
    }
    out
}

fn cmd_help() -> String {
    "\
Commands:\n\
  help                  — list all commands\n\
  list                  — show online users and their IPs\n\
  send <user|*> <hex>   — send a raw payload to one user or everyone\n\
  kick <user>           — forcibly disconnect a user\n\
  create <user> <token> — register a new player with an explicit token\n\
  delete <user>         — delete a player (refused if online)\n\
  spoof <user>          — inject a fake session for <user>\n\
  unspoof               — tear down the active fake session\n\
  recv <hex>            — feed a raw client packet to the spoofed user\n\
  reports               — list all player reports\n\
  db <base64-sql>       — run a raw SQL query against the database\n\
  restart               — kill the server (systemd will restart it)\n\
  exit                  — close this admin session\n\
".to_string()
}

fn cmd_exit(adm: &mut AdminSession) -> String {
    adm.exit = true;
    String::new()
}

fn cmd_restart() -> String {
    println!("[terminal] Restart requested — exiting.");
    std::process::exit(1);
}

fn cmd_db(state: &SharedState, args: &str) -> String {
    let b64 = args.trim();
    if b64.is_empty() {
        return "[!] Usage: db <base64-encoded-sql>\n".to_string();
    }

    let sql = match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s)  => s,
            Err(_) => return "[!] SQL is not valid UTF-8 after decoding.\n".to_string(),
        },
        Err(e) => return format!("[!] Base64 decode failed: {}\n", e),
    };

    state.db.run_sql(&sql)
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn read_line(stream: &mut TcpStream) -> Option<String> {
    let mut buf  = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) | Err(_) => return None,
            Ok(_) => match byte[0] {
                b'\n' => break,
                b'\r' => {}
                b     => buf.push(b),
            },
        }
    }
    String::from_utf8(buf).ok()
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.replace(' ', "");
    if s.len() % 2 != 0 {
        return Err("odd number of hex digits".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|_| format!("invalid hex byte '{}'", &s[i..i + 2])))
        .collect()
}
