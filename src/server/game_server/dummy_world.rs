// dummy_world.rs — admin-spawned managed worlds with auto-accept.
//
// Two modes:
//
//   `start_world_for` — attaches a managed world to an existing spoofed user.
//     When someone sends a JoinReq to that user, the server auto-accepts and
//     sends JumpToGame pointing to the managed game session. This is the
//     normal workflow: `spoof eris` → `startworld` → friends join Eris.
//
//   `start_world` — creates a standalone bot session. Useful when no spoof
//     is active. Players must add the bot as a friend first.
//
// Admin commands:
//   startworld [name]  — host a world on the spoofed user (or create a bot)
//   stopworld  [name]  — tear it down

use std::sync::Arc;

use crate::defs::packet::{Str16, DEFAULT_WORLD};
use crate::defs::state::{SessionConn, SharedState};
use crate::server::friend_server::packets_server::{JoinGrantHostClear, JumpToGame, ServerPacket};
use super::{Session, world_state::WorldState};
use crate::utils::config::Config;

/// Runtime state for a single admin-spawned world.
pub struct DummyWorld {
    /// The game server port this world is listening on.
    pub port: u16,
    /// The world state backing the managed session.
    pub world: Arc<WorldState>,
    /// Handle to the game session — used to shut it down.
    pub session: Arc<Session>,
    /// Whether this world created its own bot session (true) or reuses a spoof (false).
    pub owns_session: bool,
}

/// Attaches a managed world to an existing spoofed user session.
///
/// The dummy world is keyed by `host_user` in `dummy_worlds`, so when a
/// JoinReq targets that user, `handle_auto_accept` fires automatically.
pub fn start_world_for(host_user: &str, state: &SharedState, cfg: &Config) -> String {
    if state.dummy_worlds.read().unwrap().contains_key(host_user) {
        return format!("[!] '{}' is already hosting a world.\n", host_user);
    }

    if !state.sessions.read().unwrap().contains_key(host_user) {
        return format!("[!] '{}' is not online (spoof them first).\n", host_user);
    }

    let world = Arc::new(WorldState::new(host_user, 5));

    let (port, session) = match super::spawn_managed_session(host_user.to_string(), cfg, Arc::clone(&world)) {
        Some(r) => r,
        None => return "[!] No free port available for the game session.\n".to_string(),
    };

    let dummy = DummyWorld { port, world, session, owns_session: false };
    state.dummy_worlds.write().unwrap().insert(host_user.to_string(), dummy);

    // Show the host as "in Personal World" so friends know they can join.
    state.world_states.write().unwrap()
        .insert(host_user.to_string(), DEFAULT_WORLD.to_vec());
    state.broadcast_status(host_user, true);

    format!("World hosted by '{}' on port {}.\n", host_user, port)
}

/// Standalone mode: creates a bot session and hosts a world under that name.
pub fn start_world(name: &str, state: &SharedState, cfg: &Config) -> String {
    if state.dummy_worlds.read().unwrap().contains_key(name) {
        return format!("[!] World '{}' is already running.\n", name);
    }

    if state.sessions.read().unwrap().contains_key(name) {
        return format!("[!] '{}' is already online as a real user.\n", name);
    }

    let world = Arc::new(WorldState::new(name, 5));

    let (port, session) = match super::spawn_managed_session(name.to_string(), cfg, Arc::clone(&world)) {
        Some(r) => r,
        None => return "[!] No free port available for the game session.\n".to_string(),
    };

    // Create a spoofed bot session so the bot appears online.
    let bot_conn = SessionConn::new_sink(format!("WORLD:{}", name));
    state.sessions.write().unwrap().insert(name.to_string(), Arc::clone(&bot_conn));
    state.world_states.write().unwrap()
        .insert(name.to_string(), DEFAULT_WORLD.to_vec());
    state.broadcast_status(name, true);

    let dummy = DummyWorld { port, world, session, owns_session: true };
    state.dummy_worlds.write().unwrap().insert(name.to_string(), dummy);

    format!("World '{}' started on port {}.\n", name, port)
}

/// Tears down a dummy world. If the world created its own bot session,
/// that session is removed too. If it was attached to a spoof, the
/// spoof session is left intact.
pub fn stop_world(name: &str, state: &SharedState) -> String {
    let removed = state.dummy_worlds.write().unwrap().remove(name);
    match removed {
        None => format!("[!] No world hosted by '{}' is running.\n", name),
        Some(dw) => {
            // Disconnect all players and stop the accept loop.
            dw.session.stop();

            if dw.owns_session {
                state.sessions.write().unwrap().remove(name);
                state.world_states.write().unwrap().remove(name);
                state.broadcast_status(name, false);
            } else {
                // Spoof stays online — just clear the world state back to idle.
                state.world_states.write().unwrap().remove(name);
                state.broadcast_status(name, true);
            }
            format!("World hosted by '{}' stopped.\n", name)
        }
    }
}

/// Called by the JoinReq handler when the target is a dummy world bot.
/// Spawns a thread that sleeps 3 seconds, then sends JoinGrantHostClear
/// and JumpToGame to the joiner.
pub fn handle_auto_accept(
    joiner: &str,
    world_name: &str,
    state: &Arc<SharedState>,
    cfg: &Config,
) {
    let joiner = joiner.to_string();
    let world_name = world_name.to_string();
    let state = Arc::clone(state);
    let ip = if cfg.public_ip.is_empty() { cfg.host.clone() } else { cfg.public_ip.clone() };

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));

        let port = match state.dummy_worlds.read().unwrap().get(&world_name) {
            Some(dw) => dw.port,
            None => {
                println!("[DUMMY] World '{}' gone before auto-accept fired", world_name);
                return;
            }
        };

        let joiner_conn = match state.sessions.read().unwrap().get(&joiner) {
            Some(c) => Arc::clone(c),
            None => {
                println!("[DUMMY] Joiner '{}' went offline before auto-accept", joiner);
                return;
            }
        };

        // 1. Unfreeze the joiner's UI.
        joiner_conn.send_pkt(&JoinGrantHostClear, "S->C [DUMMY_UNFREEZE]");

        // 2. Send JumpToGame to redirect them to the managed session.
        let jump = JumpToGame {
            display:       Str16::new(&world_name),
            token:         Str16::new(&world_name),
            host_ip:       Str16::new(&ip),
            mode:          Str16::new(&ip),  // fallback IP (same as primary)
            port,
            password_flag: 0x00,
        };
        joiner_conn.send_pkt(&jump, "S->C [DUMMY_JUMP]");

        println!("[DUMMY] Auto-accepted '{}' into world '{}'", joiner, world_name);
    });
}
