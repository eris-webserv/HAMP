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

    /// Directory containing server icon PNG files.  Each file should be named
    /// `{server_name}.png` (case-insensitive match).  Relative paths are
    /// resolved from the working directory.
    pub icons_dir: String,

    /// Directory where the managed game server stores world state.
    /// Contains `world.hws` and future per-world subdirectories.
    pub world_data_dir: String,

    /// Seed for world generation.  If omitted (or 0), a random seed is chosen
    /// on first generation and saved into `world.hws` so it persists across
    /// restarts.  Has no effect once a save file already exists.
    pub world_seed: Option<u64>,

    // ── World generation — biome commonness ────────────────────────────────
    //
    // Relative weights for each biome. Values are ratios (e.g. 1.00, 0.50);
    // the generator normalizes the set across the 36 blobs that cover one
    // sector. A weight of 0 disables that biome entirely.
    //
    // Mirrors the `Biome '<Name>' commonness` options in the original
    // HybridsPublicServer config. Woodlands and Sakura did not exist in
    // the public server and are exposed here for parity with the
    // reimplementation's extra biomes.
    pub biome_grass_commonness:     f32,
    pub biome_snow_commonness:      f32,
    pub biome_desert_commonness:    f32,
    pub biome_evergreen_commonness: f32,
    pub biome_ocean_commonness:     f32,
    pub biome_swamp_commonness:     f32,
    pub biome_woodlands_commonness: f32,
    pub biome_sakura_commonness:    f32,

    /// Biome name forced within `start_biome_radius` of (0, 0).
    /// Valid values: "Grassland", "Snow", "Desert", "Evergreen", "Ocean",
    /// "Swamp", "Woodlands", "Sakura". Empty string disables the override.
    pub start_biome: String,

    /// Half-extent (in chunks) of the forced start-area biome box around
    /// the world origin. 0 disables.
    pub start_biome_radius: i16,

    /// Whether PVP (player-vs-player damage) is enabled on this game server.
    /// When false (default), `CombatControl$HitAllowed` blocks all player
    /// damage on the client side.  Set to true to enable combat between players.
    pub pvp_enabled: bool,

    /// Print a hex dump of every C→S packet to stdout.  Useful for debugging
    /// protocol issues.  Can be noisy on busy servers.
    pub log_packets: bool,

    // ── Friend-server registry (game server → friend server) ──────────────
    /// Hostname/IP of the friend server's registry listener.
    /// Leave empty to disable registry registration.
    pub friend_registry_host: String,

    /// Port of the friend server's registry listener (matches `registry_port`
    /// on the friend server). Set to 0 to disable.
    pub friend_registry_port: u16,

    /// Shared secret matching `registry_secret` on the friend server.
    /// Leave empty to disable.
    pub friend_registry_secret: String,

    /// Name shown in the server list on the friend server.
    pub server_name: String,

    /// Description lines shown in the server list (desc1–desc4 slots).
    pub server_desc: String,
    pub server_desc2: String,
    pub server_desc3: String,
    pub server_desc4: String,

    /// Maximum concurrent players advertised to the friend server.
    pub server_max_players: i16,

    /// Game-mode string advertised to the friend server (e.g. "survival").
    pub server_game_mode: String,

    /// Room token sent in the JumpToGame packet so the game server can
    /// identify the session. Leave empty to use a random token.
    pub server_room_token: String,
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
            icons_dir:     "icons".to_string(),
            world_data_dir:"world_data".to_string(),
            world_seed:     None,

            biome_grass_commonness:     1.00,
            biome_snow_commonness:      0.50,
            biome_desert_commonness:    0.50,
            biome_evergreen_commonness: 0.50,
            biome_ocean_commonness:     1.00,
            biome_swamp_commonness:     0.50,
            biome_woodlands_commonness: 0.50,
            biome_sakura_commonness:    0.50,
            start_biome:               "Grassland".to_string(),
            start_biome_radius:         3,

            pvp_enabled:    false,
            log_packets:    true,

            friend_registry_host:   String::new(),
            friend_registry_port:   0,
            friend_registry_secret: String::new(),
            server_name:        "HAMP Server".to_string(),
            server_desc:        String::new(),
            server_desc2:       String::new(),
            server_desc3:       String::new(),
            server_desc4:       String::new(),
            server_max_players: 50,
            server_game_mode:   String::new(),
            server_room_token:  String::new(),
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

# Directory containing server icon PNGs, named "{server_name}.png".
icons_dir = "icons"

# Directory where the managed game server stores world state (world.hws).
world_data_dir = "world_data"

# Optional fixed seed for world generation (u64).  Omit or set to 0 for a
# random seed chosen at first startup.  Ignored if a save file already exists.
# world_seed = 12345678

# ── World generation — biome commonness ────────────────────────────────────
# Relative weights for each biome. Values are ratios (e.g. 1.00, 0.50); the
# generator normalizes them across the 36-blob pool that covers each sector.
# A weight of 0 disables that biome.  Only read on first world generation —
# after a save file exists these values come from world.hws.
biome_grass_commonness     = 1.00
biome_snow_commonness      = 0.50
biome_desert_commonness    = 0.50
biome_evergreen_commonness = 0.50
biome_ocean_commonness     = 1.00
biome_swamp_commonness     = 0.50
# Extra biomes specific to this reimplementation (not in the original server).
biome_woodlands_commonness = 0.50
biome_sakura_commonness    = 0.50

# Forced biome in the start area (|chunk_x| <= radius && |chunk_z| <= radius).
# Valid: "Grassland", "Snow", "Desert", "Evergreen", "Ocean", "Swamp",
#        "Woodlands", "Sakura".  Empty string disables.
start_biome        = "Grassland"
start_biome_radius = 3

# Enable player-vs-player combat (default: false).
# pvp_enabled = false

# ── Friend-server registry ──────────────────────────────────────────────────
# Uncomment all three to have the game server register itself with a friend
# server so it appears in the public server list.
#
# friend_registry_host   = "3.5.3.2"
# friend_registry_port   = 7004
# friend_registry_secret = "change_me"
#
# What to show in the server list:
# server_name        = "My Server"
# server_desc        = ""
# server_desc2       = ""
# server_desc3       = ""
# server_desc4       = ""
# server_max_players = 50
# server_game_mode   = ""
# server_room_token  = ""
"#;
