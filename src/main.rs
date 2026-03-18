mod admin;
mod config;
mod db;
mod friend_server;
mod game_server;
mod packet;
mod state;
mod structs;

use std::sync::Arc;

use crate::{db::Db, state::SharedState};

fn main() {
    let cfg = config::load();
    let args: Vec<String> = std::env::args().collect();

    // ── Console mode (separate window) ────────────────────────────────────
    if args.get(1).map(|s| s == "console").unwrap_or(false) {
        // admin::command_console();
        return;
    }

    let db_path = std::env::current_exe()
        .expect("could not locate executable")
        .parent()
        .expect("executable has no parent directory")
        .join(&cfg.db_path);

    let db = Db::open(db_path.to_str().expect("db path is not valid UTF-8"))
        .expect("failed to open database");

    // ── Server mode ───────────────────────────────────────────────────────
    let shared = SharedState::new(db);

    println!("[HAMP] Launching!");

    // Game server runs on a background thread; friend server runs here on
    // the main thread so the process lives as long as the friend server does.
    //let gs_state = Arc::clone(&shared);
    //std::thread::spawn(move || game_server::run_game_server(gs_state));

    friend_server::run(&cfg, shared);
}
