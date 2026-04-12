// server_registry.rs — external game server registry.
//
// External game servers (potentially on separate machines) open a persistent
// TCP connection here to register themselves and stream player-count updates.
// The friend server maintains a live list from these connections; the public
// server list (S→C 0x1D) is built directly from it.
//
// Internal wire protocol — all numbers little-endian:
//   Strings: [u16 byte_len][UTF-8 bytes]
//
//   Game server → friend server:
//     0x01  Auth:     [Str(secret)]
//     0x02  Register: [Str(name)][Str(desc1)][Str(desc2)][Str(desc3)][Str(desc4)]
//                     [i16(max_players)][Str(game_mode)][Str(public_ip)][u16(port)]
//                     [Str(room_token)]
//     0x03  Update:   [i16(n_online)]
//
//   Friend server → game server:
//     0x01  Auth OK
//     0x00  Auth fail (server then closes connection)
//
// On disconnect the server entry is removed from the list immediately.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

use crate::utils::config::Config;

// ── Registered server entry ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RegisteredServer {
    pub name:        String,
    pub desc1:       String,
    pub desc2:       String,
    pub desc3:       String,
    pub desc4:       String,
    pub max_players: i16,
    pub game_mode:   String,
    /// Public IP the client should connect to.
    pub public_ip:   String,
    /// Game server port.
    pub port:        u16,
    /// Room token sent in the JumpToGame packet so the game server can
    /// identify the session.
    pub room_token:  String,
    pub n_online:    i16,
}

// ── Internal wire helpers ──────────────────────────────────────────────────

fn read_u8(s: &mut TcpStream) -> Option<u8> {
    let mut b = [0u8; 1];
    s.read_exact(&mut b).ok()?;
    Some(b[0])
}

fn read_i16(s: &mut TcpStream) -> Option<i16> {
    let mut b = [0u8; 2];
    s.read_exact(&mut b).ok()?;
    Some(i16::from_le_bytes(b))
}

fn read_u16(s: &mut TcpStream) -> Option<u16> {
    let mut b = [0u8; 2];
    s.read_exact(&mut b).ok()?;
    Some(u16::from_le_bytes(b))
}

fn read_str(s: &mut TcpStream) -> Option<String> {
    let len = read_u16(s)? as usize;
    let mut buf = vec![0u8; len];
    s.read_exact(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

// ── Per-connection handler ─────────────────────────────────────────────────

/// Drives one game-server connection. Returns the registered server name so
/// the caller can remove it from the list on disconnect (or `None` if the
/// server never fully registered).
fn handle_connection(
    stream: &mut TcpStream,
    addr:   std::net::SocketAddr,
    secret: &str,
    list:   &Arc<RwLock<Vec<RegisteredServer>>>,
) -> Option<String> {
    // ── Auth ──────────────────────────────────────────────────────────────
    if read_u8(stream)? != 0x01 {
        let _ = stream.write_all(&[0x00]);
        return None;
    }
    let token = read_str(stream)?;
    if token != secret {
        let _ = stream.write_all(&[0x00]);
        println!("[REGISTRY] {} rejected — bad secret", addr);
        return None;
    }
    if stream.write_all(&[0x01]).is_err() { return None; }
    println!("[REGISTRY] {} authenticated", addr);

    // ── Register ──────────────────────────────────────────────────────────
    if read_u8(stream)? != 0x02 { return None; }

    let name        = read_str(stream)?;
    let desc1       = read_str(stream)?;
    let desc2       = read_str(stream)?;
    let desc3       = read_str(stream)?;
    let desc4       = read_str(stream)?;
    let max_players = read_i16(stream)?;
    let game_mode   = read_str(stream)?;
    let public_ip   = read_str(stream)?;
    let port        = read_u16(stream)?;
    let room_token  = read_str(stream)?;

    let server = RegisteredServer {
        name: name.clone(),
        desc1, desc2, desc3, desc4,
        max_players,
        game_mode,
        public_ip,
        port,
        room_token,
        n_online: 0,
    };

    list.write().unwrap().push(server);
    println!("[REGISTRY] '{}' registered (max {} players, port {})", name, max_players, port);

    // ── Update loop ───────────────────────────────────────────────────────
    loop {
        let msg = match read_u8(stream) {
            Some(v) => v,
            None    => break,
        };
        if msg != 0x03 { break; }
        let n = match read_i16(stream) {
            Some(v) => v,
            None    => break,
        };
        let mut servers = list.write().unwrap();
        if let Some(s) = servers.iter_mut().find(|s| s.name == name) {
            s.n_online = n;
        }
    }

    Some(name)
}

// ── Listener ──────────────────────────────────────────────────────────────

/// Spawns the registry listener thread. Blocks until the listener binds, then
/// returns immediately — the actual accept loop runs on a background thread.
pub fn run(cfg: &Config, list: Arc<RwLock<Vec<RegisteredServer>>>) {
    if cfg.registry_secret.is_empty() || cfg.registry_port == 0 {
        println!("[REGISTRY] Disabled (set registry_port and registry_secret to enable)");
        return;
    }

    let addr = format!("{}:{}", cfg.host, cfg.registry_port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l)  => l,
        Err(e) => {
            eprintln!("[REGISTRY] Failed to bind on {}: {}", addr, e);
            return;
        }
    };
    println!("[REGISTRY] Listening on {} ...", addr);

    let secret = cfg.registry_secret.clone();
    std::thread::spawn(move || {
        for incoming in listener.incoming() {
            let stream = match incoming {
                Ok(s)  => s,
                Err(e) => { eprintln!("[REGISTRY] Accept error: {}", e); continue; }
            };
            let peer = stream.peer_addr()
                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
            let list   = Arc::clone(&list);
            let secret = secret.clone();
            std::thread::spawn(move || {
                let mut stream = stream;
                let name = handle_connection(&mut stream, peer, &secret, &list);
                if let Some(ref n) = name {
                    list.write().unwrap().retain(|s| s.name != *n);
                    println!("[REGISTRY] '{}' disconnected — removed from server list", n);
                }
            });
        }
    });
}
