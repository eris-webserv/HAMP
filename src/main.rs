// main.rs — entry point.
//
// Default: starts a standalone game server.
// With --frsv: starts the friend server + admin terminal instead.

mod defs;
mod server;
mod utils;

use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = utils::config::load();
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--frsv") {
        run_friend_server(cfg)
    } else {
        run_game_server(cfg)
    }
}

fn run_game_server(cfg: utils::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GAME SERVER ===");
    println!("  Port: {}", cfg.game_port);
    server::game_server::run(&cfg);
    Ok(())
}

fn run_friend_server(cfg: utils::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or("executable has no parent directory")?
        .to_owned();

    let db = utils::db::Db::open(
        exe_dir
            .join(&cfg.db_path)
            .to_str()
            .ok_or("db path is not valid UTF-8")?,
    )?;

    // Clean up stale pending requests (friends who still have a pending row).
    let cleaned = db.cleanup_stale_pending();
    if cleaned > 0 {
        println!("[DB] Cleaned {} stale pending request(s)", cleaned);
    }

    let state = defs::state::SharedState::new(db);

    println!("=== FRIEND SERVER ===");
    println!("  DB:          {}", cfg.db_path);
    println!("  Friend port: {}", cfg.friend_port);
    if !cfg.terminal_password.is_empty() {
        println!("  Terminal:    0.0.0.0:{}", cfg.terminal_port);
    }

    let t_state = Arc::clone(&state);
    let t_cfg = cfg.clone();
    std::thread::spawn(move || utils::admin::run_terminal(t_cfg, t_state));

    server::friend_server::run(&cfg, state);
    Ok(())
}
