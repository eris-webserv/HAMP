// config.rs — server configuration loaded from `config.toml`.
//
// On first run the file does not exist; `load` writes a well-commented
// default and returns the same defaults so the server starts immediately.
// Edit the file and restart to apply changes.

use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    // ── Network ────────────────────────────────────────────────────────────
    /// Interface address to bind. Use `"0.0.0.0"` for all interfaces.
    pub host: String,

    /// Friend-list / social server port.
    pub friend_port: u16,

    /// Lower bound of the dynamic port range for per-session game servers.
    pub game_port: u16,

    /// Upper bound of the dynamic port range for per-session game servers.
    pub game_port_max: u16,

    /// Public-facing IP address to embed in game-session redirect packets.
    /// Leave empty to use the value of `host`.
    pub public_ip: String,

    /// TCP port for the remote admin terminal.
    pub terminal_port: u16,

    /// Password required to authenticate to the remote terminal.
    /// Set to an empty string to disable the terminal entirely.
    pub terminal_password: String,

    // ── Public server registry ─────────────────────────────────────────────
    /// TCP port that external game servers connect to for registry.
    /// Set to 0 to disable the registry listener.
    pub registry_port: u16,

    /// Shared secret that external game servers must present to authenticate.
    /// Leave empty to disable the registry (no servers can register).
    pub registry_secret: String,

    // ── Persistence ────────────────────────────────────────────────────────
    /// Path to the SQLite database file.  Relative paths are resolved from
    /// the working directory where the server binary is run.
    pub db_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host:          "0.0.0.0".to_string(),
            friend_port:   7002,
            game_port:     7100,
            game_port_max: 7800,
            public_ip:     String::new(),
            terminal_port:     7006,
            terminal_password: String::new(),
            registry_port:     7004,
            registry_secret:   String::new(),
            db_path:       "friend_server.db".to_string(),
        }
    }
}

/// Resolves `config.toml` next to the running binary so it lands in
/// `target/debug/` (or `target/release/`) and is never committed by git.
fn config_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_owned()))
        .unwrap_or_default()
        .join("config.toml")
}

/// Loads `config.toml` from the same directory as the binary.
///
/// If the file does not exist a default one is written and the defaults are
/// returned.  Parse errors are reported to stderr and defaults are used so
/// the server can always start.
pub fn load() -> Config {
    let path = config_path();
    if !path.exists() {
        let _ = fs::write(&path, DEFAULT_TOML);
        eprintln!("[config] No config.toml found — wrote defaults to {}", path.display());
        return Config::default();
    }

    match fs::read_to_string(&path) {
        Err(e) => {
            eprintln!("[config] Cannot read {}: {} — using defaults", path.display(), e);
            Config::default()
        }
        Ok(raw) => match toml::from_str(&raw) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[config] Parse error in {}: {} — using defaults", path.display(), e);
                Config::default()
            }
        },
    }
}

const DEFAULT_TOML: &str = r#"# Friend server configuration
# Edit and restart the server to apply changes.

# ── Network ────────────────────────────────────────────────────────────────
# Interface to bind on.  "0.0.0.0" listens on all interfaces.
host = "0.0.0.0"

# Friend-list / social server (clients connect here).
friend_port = 7002

# Public IP to embed in game-session redirect packets (leave empty to use host).
public_ip = ""

# Dynamic port range for per-session game servers [game_port, game_port_max].
game_port     = 7100
game_port_max = 7800

# ── Remote admin terminal ──────────────────────────────────────────────────
# TCP port for the remote admin terminal (telnet / netcat).
terminal_port = 7006

# Password required to log in. Leave empty ("") to disable the terminal.
terminal_password = ""

# ── Public server registry ──────────────────────────────────────────────────
# Port that external game servers connect to for registry.
# Set to 0 to disable.
registry_port = 7004

# Shared secret external game servers must present on connect.
# Leave empty ("") to disable the registry entirely.
registry_secret = ""

# ── Persistence ────────────────────────────────────────────────────────────
# Path to the SQLite database file (relative to the server's working dir).
db_path = "friend_server.db"
"#;
