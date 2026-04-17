// db.rs — SQLite-backed persistence via rusqlite.
//
// `Db` wraps a `Mutex<Connection>` so a single `Arc<Db>` can be shared
// safely across every handler thread.  All methods acquire the lock for the
// duration of a single logical operation (one or more SQL statements that
// belong together in a transaction).
//
// Username casing
// ───────────────
// Usernames are stored exactly as the player typed them at registration
// (e.g. "ILuv").  The primary key uses COLLATE NOCASE so lookups with any
// casing find the right row.  After any lookup callers should use the
// returned `PlayerRow.username` as the canonical key for in-memory maps.

use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, Result as SqlResult};

// ── Row types returned by DB queries ──────────────────────────────────────

/// Minimal player record returned from the `players` table.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerRow {
    pub username:     String,
    pub token:        String,
    pub display_name: Option<String>,
}

/// A report entry returned from the `reports` table.
#[derive(Debug, Clone)]
pub struct ReportRow {
    pub id:        i64,
    pub timestamp: String,
    pub reporter:  String,
    pub reported:  String,
    pub reason:    String,
}

// ── Db ─────────────────────────────────────────────────────────────────────

pub struct Db(Mutex<Connection>);

impl Db {
    /// Opens (or creates) the SQLite database at `path`, runs any pending
    /// schema migrations, and returns a shared handle.
    pub fn open(path: &str) -> SqlResult<Arc<Self>> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        migrate(&conn)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Arc::new(Self(Mutex::new(conn))))
    }

    // ── Player queries ─────────────────────────────────────────────────────

    /// Returns the player record for `username` (case-insensitive).
    /// `PlayerRow.username` is the canonical stored casing.
    pub fn get_player(&self, username: &str) -> Option<PlayerRow> {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT username, token, display_name FROM players WHERE username = ?1",
            params![username],
            |row| Ok(PlayerRow {
                username:     row.get(0)?,
                token:        row.get(1)?,
                display_name: row.get(2)?,
            }),
        ).ok()
    }

    /// Returns the effective display name for `username`, falling back to
    /// `username` itself if none has been set.  Moderator accounts are
    /// automatically wrapped in `<i>…★</i>` Unity Rich Text.
    pub fn get_display_name(&self, username: &str) -> String {
        let conn = self.0.lock().unwrap();
        let row = conn.query_row(
            "SELECT display_name, COALESCE(is_moderator, 0) FROM players WHERE username = ?1",
            params![username],
            |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?)),
        ).ok();
        let (raw, is_mod) = match row {
            Some((r, m)) => (r.unwrap_or_else(|| username.to_string()), m != 0),
            None         => (username.to_string(), false),
        };
        if is_mod { format!("<i>{}★</i>", raw) } else { raw }
    }

    #[allow(dead_code)]
    /// Returns `true` if `username` has the moderator flag set.
    pub fn is_moderator(&self, username: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT COALESCE(is_moderator, 0) FROM players WHERE username = ?1",
            params![username],
            |row| row.get::<_, i64>(0),
        ).ok().map(|v| v != 0).unwrap_or(false)
    }

    /// Sets or clears the moderator flag for `username`.
    /// Returns `false` if the player does not exist.
    pub fn set_moderator(&self, username: &str, value: bool) -> bool {
        if !self.player_exists(username) { return false; }
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE players SET is_moderator = ?1 WHERE username = ?2",
            params![value as i64, username],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Sets a custom display name for `username`.
    /// Pass an empty string to clear the display name (revert to username).
    /// Returns `false` if the player does not exist.
    pub fn set_display_name(&self, username: &str, display: &str) -> bool {
        if !self.player_exists(username) { return false; }
        let val: Option<&str> = if display.is_empty() { None } else { Some(display) };
        let conn = self.0.lock().unwrap();
        conn.execute(
            "UPDATE players SET display_name = ?1 WHERE username = ?2",
            params![val, username],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Returns `true` if a player with the given username exists (case-insensitive).
    pub fn player_exists(&self, username: &str) -> bool {
        self.get_player(username).is_some()
    }

    /// Inserts a new player with the exact casing provided.
    /// Returns `false` (no-op) if the username is already taken (case-insensitive).
    pub fn create_player(&self, username: &str, token: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO players (username, token) VALUES (?1, ?2)",
            params![username, token],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Deletes a player and all their associated data.
    /// Returns `false` if the player did not exist.
    pub fn delete_player(&self, username: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.execute("DELETE FROM players WHERE username = ?1", params![username])
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    // ── Friend queries ─────────────────────────────────────────────────────

    /// Returns the username of every confirmed friend of `username`.
    pub fn get_friends(&self, username: &str) -> Vec<String> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT user_b FROM friends WHERE user_a = ?1 ORDER BY user_b",
        ).unwrap();
        stmt.query_map(params![username], |row| row.get(0))
            .unwrap().flatten().collect()
    }

    /// Returns `true` if `a` and `b` are mutual friends.
    pub fn are_friends(&self, a: &str, b: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT 1 FROM friends WHERE user_a = ?1 AND user_b = ?2",
            params![a, b],
            |_| Ok(()),
        ).is_ok()
    }

    // ── Pending request queries ────────────────────────────────────────────

    /// Returns every from_user of inbound pending requests aimed at `username`.
    pub fn get_pending_inbound(&self, username: &str) -> Vec<String> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT from_user FROM pending WHERE to_user = ?1 ORDER BY from_user",
        ).unwrap();
        stmt.query_map(params![username], |row| row.get(0))
            .unwrap().flatten().collect()
    }

    /// Returns every to_user of outbound pending requests sent by `username`.
    pub fn get_pending_outbound(&self, username: &str) -> Vec<String> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT to_user FROM pending WHERE from_user = ?1 ORDER BY to_user",
        ).unwrap();
        stmt.query_map(params![username], |row| row.get(0))
            .unwrap().flatten().collect()
    }

    /// Returns `true` if a pending request from `from` to `to` exists.
    pub fn has_pending(&self, from: &str, to: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT 1 FROM pending WHERE from_user = ?1 AND to_user = ?2",
            params![from, to],
            |_| Ok(()),
        ).is_ok()
    }

    // ── Social graph mutations ─────────────────────────────────────────────

    /// Sends a friend request from `from` to `to` (both must be canonical usernames).
    pub fn add_friend_request(&self, from: &str, to: &str) -> bool {
        if !self.player_exists(from) || !self.player_exists(to) { return false; }
        if self.are_friends(from, to) || self.has_pending(from, to) { return false; }
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO pending (from_user, to_user) VALUES (?1, ?2)",
            params![from, to],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Accepts the pending request where `requester → acceptor`.
    /// Returns `false` if no such pending request exists.
    pub fn accept_friend(&self, acceptor: &str, requester: &str) -> bool {
        let conn = self.0.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM pending WHERE from_user = ?1 AND to_user = ?2",
            params![requester, acceptor],
        ).unwrap_or(0);
        if n == 0 { return false; }
        // Also remove any reverse-direction request (acceptor had also sent one to requester),
        // preventing a ghost outbound entry from surviving on the acceptor's next login.
        conn.execute(
            "DELETE FROM pending WHERE from_user = ?1 AND to_user = ?2",
            params![acceptor, requester],
        ).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO friends (user_a, user_b) VALUES (?1, ?2)",
            params![acceptor, requester],
        ).unwrap_or(0);
        conn.execute(
            "INSERT OR IGNORE INTO friends (user_a, user_b) VALUES (?1, ?2)",
            params![requester, acceptor],
        ).unwrap_or(0);
        true
    }

    /// Removes all relationship ties between `a` and `b`.
    pub fn remove_friend(&self, a: &str, b: &str) {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "DELETE FROM friends
             WHERE (user_a = ?1 AND user_b = ?2) OR (user_a = ?2 AND user_b = ?1)",
            params![a, b],
        ).unwrap_or(0);
        conn.execute(
            "DELETE FROM pending
             WHERE (from_user = ?1 AND to_user = ?2) OR (from_user = ?2 AND to_user = ?1)",
            params![a, b],
        ).unwrap_or(0);
    }

    // ── Maintenance ───────────────────────────────────────────────────────

    /// Deletes any pending request where the two users are already friends.
    /// Returns the number of stale rows removed.
    pub fn cleanup_stale_pending(&self) -> usize {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "DELETE FROM pending WHERE EXISTS (
                SELECT 1 FROM friends
                WHERE friends.user_a = pending.from_user
                  AND friends.user_b = pending.to_user
            )",
            [],
        ).unwrap_or(0)
    }

    // ── Raw SQL (admin terminal) ───────────────────────────────────────────

    /// Executes an arbitrary SQL statement and returns results as a
    /// plain-text table, or just "OK\n" for non-SELECT statements.
    pub fn run_sql(&self, sql: &str) -> String {
        let conn = self.0.lock().unwrap();
        match conn.prepare(sql) {
            Err(e) => return format!("[!] SQL error: {}\n", e),
            Ok(mut stmt) => {
                let col_names: Vec<String> = stmt.column_names()
                    .iter().map(|s| s.to_string()).collect();

                if col_names.is_empty() {
                    // Non-SELECT (INSERT/UPDATE/DELETE/etc.)
                    match stmt.execute([]) {
                        Ok(n)  => return format!("OK ({} row(s) affected)\n", n),
                        Err(e) => return format!("[!] SQL error: {}\n", e),
                    }
                }

                let mut rows_out = vec![col_names.join(" | ")];
                let mut rows = match stmt.query([]) {
                    Ok(r)  => r,
                    Err(e) => return format!("[!] SQL error: {}\n", e),
                };
                loop {
                    match rows.next() {
                        Ok(Some(row)) => {
                            let cols: Vec<String> = (0..rows_out[0].split(" | ").count())
                                .map(|i| {
                                    row.get::<_, rusqlite::types::Value>(i)
                                        .map(|v| match v {
                                            rusqlite::types::Value::Null       => "NULL".to_string(),
                                            rusqlite::types::Value::Integer(n) => n.to_string(),
                                            rusqlite::types::Value::Real(f)    => f.to_string(),
                                            rusqlite::types::Value::Text(s)    => s,
                                            rusqlite::types::Value::Blob(b)    => format!("<blob {} bytes>", b.len()),
                                        })
                                        .unwrap_or_else(|_| "?".to_string())
                                })
                                .collect();
                            rows_out.push(cols.join(" | "));
                        }
                        Ok(None) => break,
                        Err(e)   => return format!("[!] SQL error reading row: {}\n", e),
                    }
                }
                format!("{}\n", rows_out.join("\n"))
            }
        }
    }

    // ── Reports ────────────────────────────────────────────────────────────

    /// Appends a new report entry.
    pub fn add_report(&self, reporter: &str, reported: &str, reason: &str) {
        let conn = self.0.lock().unwrap();
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO reports (timestamp, reporter, reported, reason) VALUES (?1, ?2, ?3, ?4)",
            params![ts, reporter, reported, reason],
        ).unwrap_or(0);
    }

    /// Returns all report rows ordered by insertion time.
    pub fn get_reports(&self) -> Vec<ReportRow> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, reporter, reported, reason FROM reports ORDER BY id",
        ).unwrap();
        stmt.query_map([], |row| Ok(ReportRow {
            id:        row.get(0)?,
            timestamp: row.get(1)?,
            reporter:  row.get(2)?,
            reported:  row.get(3)?,
            reason:    row.get(4)?,
        })).unwrap().flatten().collect()
    }
}

// ── Migration ──────────────────────────────────────────────────────────────

/// Upgrades an older schema to the current layout.
///
/// v1 → v2: drop the `display` column; add `COLLATE NOCASE` to the primary
///           key so lookups are case-insensitive while storing chosen casing.
/// v2 → v3: add `display_name TEXT` column for cosmetic names.
/// v3 → v4: add `is_moderator INTEGER NOT NULL DEFAULT 0`.
fn migrate(conn: &Connection) -> SqlResult<()> {
    let cols: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(players)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .flatten()
            .collect()
    };

    // v1 → v2: drop the old `display` column, rebuild with COLLATE NOCASE.
    if cols.iter().any(|c| c == "display") {
        println!("[DB] Migrating schema v1→v2: dropping display column, adding COLLATE NOCASE …");
        conn.execute_batch("
            PRAGMA foreign_keys = OFF;
            BEGIN;

            CREATE TABLE players_new (
                username     TEXT PRIMARY KEY COLLATE NOCASE,
                token        TEXT NOT NULL,
                display_name TEXT
            );
            INSERT INTO players_new (username, token)
                SELECT username, token FROM players;
            DROP TABLE players;
            ALTER TABLE players_new RENAME TO players;

            COMMIT;
            PRAGMA foreign_keys = ON;
        ")?;
        println!("[DB] Migration v1→v2 complete.");
        return Ok(()); // display_name already added above
    }

    // v2 → v3: add display_name column if it doesn't exist yet.
    if !cols.iter().any(|c| c == "display_name") {
        println!("[DB] Migrating schema v2→v3: adding display_name column …");
        conn.execute_batch("ALTER TABLE players ADD COLUMN display_name TEXT;")?;
        println!("[DB] Migration v2→v3 complete.");
    }

    // v3 → v4: add is_moderator flag.
    if !cols.iter().any(|c| c == "is_moderator") {
        println!("[DB] Migrating schema v3→v4: adding is_moderator column …");
        conn.execute_batch(
            "ALTER TABLE players ADD COLUMN is_moderator INTEGER NOT NULL DEFAULT 0;"
        )?;
        println!("[DB] Migration v3→v4 complete.");
    }

    Ok(())
}

// ── Schema ─────────────────────────────────────────────────────────────────

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS players (
    username     TEXT PRIMARY KEY COLLATE NOCASE,
    token        TEXT NOT NULL,
    display_name TEXT,
    is_moderator INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS friends (
    user_a TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    user_b TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    PRIMARY KEY (user_a, user_b)
);

CREATE TABLE IF NOT EXISTS pending (
    from_user TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    to_user   TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    PRIMARY KEY (from_user, to_user)
);

CREATE TABLE IF NOT EXISTS reports (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT    NOT NULL,
    reporter  TEXT    NOT NULL,
    reported  TEXT    NOT NULL,
    reason    TEXT    NOT NULL
);
";
