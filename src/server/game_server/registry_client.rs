// registry_client.rs — game server → friend server registry client.
//
// Maintains a persistent connection to the friend server's registry listener.
// The connection is bidirectional:
//
//   GS→FS (existing):
//     0x01  Auth:     [Str(secret)]
//     0x02  Register: [Str(name)][Str(desc1..4)][i16(max_players)]
//                     [Str(game_mode)][Str(public_ip)][u16(port)][Str(room_token)]
//     0x03  Update:   [i16(n_online)]
//     0x04  Ping      (keepalive, no payload)
//
//   FS→GS (existing):
//     0x01  Auth OK
//     0x00  Auth fail
//     0x04  Pong      (response to 0x04 Ping)
//
//   GS→FS (new — RPC requests):
//     0x05  RpcReq:   [u16(request_id)][u8(method)][...payload]
//           method 0x01  GetDisplayName: [Str(username)]
//
//   FS→GS (new — RPC responses):
//     0x05  RpcResp:  [u16(request_id)][...payload]
//           (GetDisplayName response payload): [Str(display_name)]
//
//   FS→GS (new — server push, reserved for future TGS-style control):
//     0x06  Push:     [u8(push_type)][...payload]
//           (no push types defined yet)
//
// On disconnect the client reconnects with exponential backoff (5s→60s).
//
// The `RegistryHandle` is cheaply clonable (`Arc` inside) and safe to share
// across packet handler threads.  `get_display_name` blocks up to 5 seconds
// for a response and returns `None` on timeout or disconnect.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use super::Session;

// ── Wire helpers ──────────────────────────────────────────────────────────

fn write_str(s: &mut impl Write, v: &str) -> std::io::Result<()> {
    let b = v.as_bytes();
    s.write_all(&(b.len() as u16).to_le_bytes())?;
    s.write_all(b)
}

fn read_u8(s: &mut impl Read) -> Option<u8> {
    let mut b = [0u8; 1];
    s.read_exact(&mut b).ok()?;
    Some(b[0])
}

fn read_u16(s: &mut impl Read) -> Option<u16> {
    let mut b = [0u8; 2];
    s.read_exact(&mut b).ok()?;
    Some(u16::from_le_bytes(b))
}

fn read_str(s: &mut impl Read) -> Option<String> {
    let len = read_u16(s)? as usize;
    let mut buf = vec![0u8; len];
    s.read_exact(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

// ── Connection params ─────────────────────────────────────────────────────

pub struct RegistryParams {
    pub registry_addr:  String,
    pub secret:         String,
    pub server_name:    String,
    pub server_desc:    String,
    pub server_desc2:   String,
    pub server_desc3:   String,
    pub server_desc4:   String,
    pub max_players:    i16,
    pub game_mode:      String,
    pub public_ip:      String,
    pub game_port:      u16,
    pub room_token:     String,
}

// ── RegistryHandle ────────────────────────────────────────────────────────

/// Cloneable handle for making RPC calls to the friend server over the
/// registry connection.  Methods return `None` when disconnected or on timeout.
#[derive(Clone)]
pub struct RegistryHandle {
    write_tx: mpsc::Sender<OutMsg>,
    pending:  Arc<Mutex<HashMap<u16, mpsc::Sender<Option<String>>>>>,
    next_id:  Arc<AtomicU16>,
}

impl RegistryHandle {
    /// Asks the friend server for the display name of `username`.
    ///
    /// Blocks up to 5 seconds waiting for the response.
    /// Returns `None` on timeout, disconnect, or if the friend server is
    /// not configured — callers should fall back to the raw username.
    pub fn get_display_name(&self, username: &str) -> Option<String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();

        self.pending.lock().unwrap().insert(id, tx);

        let mut buf = vec![0x05u8];
        buf.extend_from_slice(&id.to_le_bytes());
        buf.push(0x01); // method: GetDisplayName
        write_str(&mut buf, username).ok()?;

        if self.write_tx.send(OutMsg::Bytes(buf)).is_err() {
            self.pending.lock().unwrap().remove(&id);
            return None;
        }

        rx.recv_timeout(Duration::from_secs(5)).ok().flatten()
    }
}

// ── Internal message types ────────────────────────────────────────────────

enum OutMsg {
    /// Raw bytes to write to the TCP stream.
    Bytes(Vec<u8>),
}

// ── Spawn ─────────────────────────────────────────────────────────────────

/// Spawns the registry client background threads and returns a `RegistryHandle`.
/// Does nothing (returns `None`) if any of host/port/secret is absent.
pub fn spawn(params: RegistryParams, session: Arc<Session>) -> Option<RegistryHandle> {
    if params.registry_addr.starts_with(':') || params.registry_addr.is_empty() {
        return None;
    }

    let (write_tx, write_rx) = mpsc::channel::<OutMsg>();
    let pending = Arc::new(Mutex::new(HashMap::<u16, mpsc::Sender<Option<String>>>::new()));
    let next_id = Arc::new(AtomicU16::new(1));

    let handle = RegistryHandle {
        write_tx: write_tx.clone(),
        pending: Arc::clone(&pending),
        next_id: Arc::clone(&next_id),
    };

    std::thread::Builder::new()
        .name("registry-client".to_string())
        .spawn(move || run_loop(params, session, write_tx, write_rx, pending))
        .expect("failed to spawn registry-client thread");

    Some(handle)
}

// ── Reconnect loop ────────────────────────────────────────────────────────

fn run_loop(
    params:   RegistryParams,
    session:  Arc<Session>,
    write_tx: mpsc::Sender<OutMsg>,
    write_rx: mpsc::Receiver<OutMsg>,
    pending:  Arc<Mutex<HashMap<u16, mpsc::Sender<Option<String>>>>>,
) {
    let mut backoff = Duration::from_secs(5);

    // `write_rx` is consumed by the writer thread for each connection attempt.
    // We wrap it in an `Option` so we can move it into the first writer thread
    // and recover it on disconnect via a channel.
    let (rx_return_tx, rx_return_rx) = mpsc::channel::<mpsc::Receiver<OutMsg>>();
    let mut write_rx_slot: Option<mpsc::Receiver<OutMsg>> = Some(write_rx);

    loop {
        println!("[REGISTRY] Connecting to {} ...", params.registry_addr);

        let stream = match TcpStream::connect(&params.registry_addr) {
            Ok(s)  => s,
            Err(e) => {
                eprintln!("[REGISTRY] Connect failed: {e}");
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(60));
                continue;
            }
        };

        backoff = Duration::from_secs(5);

        // Grab write_rx for this session; we'll get it back via rx_return_rx
        // when the writer exits.
        let write_rx = write_rx_slot.take()
            .expect("write_rx missing — logic error");

        let result = run_session(
            stream,
            &params,
            &session,
            write_tx.clone(),
            write_rx,
            &rx_return_tx,
            &pending,
        );

        // Reclaim the write_rx for the next reconnect attempt.
        if let Ok(rx) = rx_return_rx.recv_timeout(Duration::from_secs(2)) {
            write_rx_slot = Some(rx);
        }

        // Cancel any callers still waiting on the old connection.
        let mut p = pending.lock().unwrap();
        for (_, tx) in p.drain() {
            let _ = tx.send(None);
        }

        match result {
            Ok(()) => println!("[REGISTRY] Disconnected — reconnecting in {}s", backoff.as_secs()),
            Err(e) => eprintln!("[REGISTRY] Error: {e} — reconnecting in {}s", backoff.as_secs()),
        }

        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(Duration::from_secs(60));
    }
}

// ── Single connection ─────────────────────────────────────────────────────

fn run_session(
    stream:        TcpStream,
    p:             &RegistryParams,
    session:       &Arc<Session>,
    write_tx:      mpsc::Sender<OutMsg>,
    write_rx:      mpsc::Receiver<OutMsg>,
    rx_return_tx:  &mpsc::Sender<mpsc::Receiver<OutMsg>>,
    pending:       &Arc<Mutex<HashMap<u16, mpsc::Sender<Option<String>>>>>,
) -> std::io::Result<()> {
    // ── Auth (synchronous, before splitting the stream) ───────────────────
    let mut s = stream;
    let auth_result = (|| -> std::io::Result<()> {
        s.write_all(&[0x01])?;
        write_str(&mut s, &p.secret)?;
        s.flush()?;

        match read_u8(&mut s) {
            Some(0x01) => println!("[REGISTRY] Authenticated"),
            Some(0x00) | None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "registry auth rejected",
                ));
            }
            Some(b) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unexpected auth response 0x{b:02X}"),
                ));
            }
        }

        // ── Register ──────────────────────────────────────────────────────
        s.write_all(&[0x02])?;
        write_str(&mut s, &p.server_name)?;
        write_str(&mut s, &p.server_desc)?;
        write_str(&mut s, &p.server_desc2)?;
        write_str(&mut s, &p.server_desc3)?;
        write_str(&mut s, &p.server_desc4)?;
        s.write_all(&p.max_players.to_le_bytes())?;
        write_str(&mut s, &p.game_mode)?;
        write_str(&mut s, &p.public_ip)?;
        s.write_all(&p.game_port.to_le_bytes())?;
        write_str(&mut s, &p.room_token)?;
        s.flush()?;
        println!("[REGISTRY] Registered as '{}'", p.server_name);
        Ok(())
    })();

    if let Err(e) = auth_result {
        // Writer thread was never spawned — return write_rx directly so the
        // reconnect loop doesn't get stuck with an empty slot.
        let _ = rx_return_tx.send(write_rx);
        return Err(e);
    }

    // ── Split stream ──────────────────────────────────────────────────────
    let read_half = match s.try_clone() {
        Ok(h) => h,
        Err(e) => {
            let _ = rx_return_tx.send(write_rx);
            return Err(e);
        }
    };
    let write_half = s;

    // Signal writer exit → give back write_rx to reconnect loop.
    let rx_return_tx = rx_return_tx.clone();

    // ── Writer thread ─────────────────────────────────────────────────────
    // Drives the heartbeat (ping every 15 s, player-count update every 30 s)
    // and flushes any queued outbound RPC requests between ticks.
    let session_clone = Arc::clone(session);
    let write_tx_clone = write_tx.clone();
    std::thread::Builder::new()
        .name("registry-writer".to_string())
        .spawn(move || {
            let result = writer_loop(write_half, write_rx, &session_clone);
            // Return the receiver so the reconnect loop can reuse it.
            let _ = rx_return_tx.send(result.1);
            if let Err(e) = result.0 {
                eprintln!("[REGISTRY] Writer error: {e}");
                // Wake up any callers blocked on get_display_name by sending a
                // dummy msg; they'll time out naturally, but this speeds it up.
                let _ = write_tx_clone.send(OutMsg::Bytes(vec![]));
            }
        })
        .expect("failed to spawn registry-writer thread");

    // ── Reader loop (runs on this thread) ─────────────────────────────────
    let mut read_half = read_half;
    read_half.set_read_timeout(Some(Duration::from_secs(25)))?;

    loop {
        let msg = match read_u8(&mut read_half) {
            Some(v) => v,
            None    => break,
        };
        match msg {
            // Pong — keepalive reply, nothing to do.
            0x04 => {}

            // RPC response.
            0x05 => {
                let req_id = match read_u16(&mut read_half) {
                    Some(id) => id,
                    None     => break,
                };
                let payload = match read_str(&mut read_half) {
                    Some(s) => s,
                    None    => break,
                };
                let mut p = pending.lock().unwrap();
                if let Some(tx) = p.remove(&req_id) {
                    let _ = tx.send(Some(payload));
                }
            }

            // Push from friend server (reserved for future TGS-style control).
            0x06 => {
                let _push_type = read_u8(&mut read_half);
                // No push types defined yet — ignore.
            }

            _ => {
                eprintln!("[REGISTRY] Unexpected byte 0x{msg:02X} from friend server");
                break;
            }
        }
    }

    Ok(())
}

// ── Writer loop ───────────────────────────────────────────────────────────

fn writer_loop(
    mut stream:  TcpStream,
    write_rx:    mpsc::Receiver<OutMsg>,
    session:     &Session,
) -> (std::io::Result<()>, mpsc::Receiver<OutMsg>) {
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let mut tick: u32 = 0;
    loop {
        // Block for up to 15 seconds waiting for queued outbound messages.
        // On timeout, fire the next heartbeat tick.
        match write_rx.recv_timeout(Duration::from_secs(15)) {
            Ok(OutMsg::Bytes(bytes)) => {
                if bytes.is_empty() {
                    // Sentinel from a dying reader — exit.
                    break;
                }
                if let Err(e) = stream.write_all(&bytes).and_then(|_| stream.flush()) {
                    return (Err(e), write_rx);
                }
                continue; // don't advance the heartbeat tick
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        tick += 1;

        let result = if tick % 2 == 0 {
            // Even ticks: player-count update.
            let n = session.player_count() as i16;
            stream.write_all(&[0x03])
                .and_then(|_| stream.write_all(&n.to_le_bytes()))
                .and_then(|_| stream.flush())
        } else {
            // Odd ticks: ping.
            stream.write_all(&[0x04]).and_then(|_| stream.flush())
        };

        if let Err(e) = result {
            return (Err(e), write_rx);
        }
    }

    (Ok(()), write_rx)
}
