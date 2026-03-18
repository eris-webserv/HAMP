// db.rs — SQLite-backed persistence via rusqlite.
//
// `Db` wraps a `Mutex<Connection>` so a single `Arc<Db>` can be shared
// safely across every handler thread.  All methods acquire the lock for the
// duration of a single logical operation (one or more SQL statements that
// belong together in a transaction).

use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, Result as SqlResult};

// ── Row types returned by DB queries ──────────────────────────────────────

/// Minimal player record returned from the `players` table.
#[derive(Debug, Clone)]
pub struct PlayerRow {
    pub username: String,
    pub display:  String,
    pub token:    String,
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
    /// Opens (or creates) the SQLite database at `path` and ensures the
    /// schema is up to date.
    pub fn open(path: &str) -> SqlResult<Arc<Self>> {
        let conn = Connection::open(path)?;
        // WAL mode: writers do not block readers across threads.
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Arc::new(Self(Mutex::new(conn))))
    }

    // ── Player queries ─────────────────────────────────────────────────────

    /// Returns the full player record for `username`, or `None` if unknown.
    pub fn get_player(&self, username: &str) -> Option<PlayerRow> {
        let conn = self.0.lock().unwrap();
        conn.query_row(
            "SELECT username, display, token FROM players WHERE username = ?1",
            params![username],
            |row| Ok(PlayerRow {
                username: row.get(0)?,
                display:  row.get(1)?,
                token:    row.get(2)?,
            }),
        ).ok()
    }

    /// Returns `true` if a player with the given username exists.
    pub fn player_exists(&self, username: &str) -> bool {
        self.get_player(username).is_some()
    }

    /// Returns only the display name for `username`.
    pub fn get_display(&self, username: &str) -> Option<String> {
        self.get_player(username).map(|p| p.display)
    }

    /// Inserts a new player. Returns `false` (no-op) if the username is taken.
    pub fn create_player(&self, username: &str, display: &str, token: &str) -> bool {
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO players (username, display, token) VALUES (?1, ?2, ?3)",
            params![username, display, token],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Returns all registered usernames.
    pub fn get_all_usernames(&self) -> Vec<String> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT username FROM players ORDER BY username")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect()
    }

    // ── Friend queries ─────────────────────────────────────────────────────

    /// Returns `(username, display)` for every confirmed friend of `username`.
    pub fn get_friends(&self, username: &str) -> Vec<(String, String)> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.username, p.display
             FROM friends f
             JOIN players p ON p.username = f.user_b
             WHERE f.user_a = ?1
             ORDER BY p.username",
        ).unwrap();
        stmt.query_map(params![username], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .flatten()
            .collect()
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

    /// Returns `(from_username, from_display)` for every inbound request
    /// aimed at `username`.
    pub fn get_pending_inbound(&self, username: &str) -> Vec<(String, String)> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.username, p.display
             FROM pending pr
             JOIN players p ON p.username = pr.from_user
             WHERE pr.to_user = ?1
             ORDER BY p.username",
        ).unwrap();
        stmt.query_map(params![username], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .flatten()
            .collect()
    }

    /// Returns `(to_username, to_display)` for every outbound request sent by
    /// `username`.
    pub fn get_pending_outbound(&self, username: &str) -> Vec<(String, String)> {
        let conn = self.0.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.username, p.display
             FROM pending pr
             JOIN players p ON p.username = pr.to_user
             WHERE pr.from_user = ?1
             ORDER BY p.username",
        ).unwrap();
        stmt.query_map(params![username], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .flatten()
            .collect()
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

    /// Sends a friend request from `from` to `to`.
    ///
    /// Returns `false` if either player is unknown, they are already friends,
    /// or a pending request in this direction already exists.
    pub fn add_friend_request(&self, from: &str, to: &str) -> bool {
        if !self.player_exists(from) || !self.player_exists(to) {
            return false;
        }
        if self.are_friends(from, to) || self.has_pending(from, to) {
            return false;
        }
        let conn = self.0.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO pending (from_user, to_user) VALUES (?1, ?2)",
            params![from, to],
        ).map(|n| n > 0).unwrap_or(false)
    }

    /// Accepts the pending request where `requester → acceptor`.
    ///
    /// Removes the pending row and inserts bidirectional friend rows.
    /// Returns `false` if no such pending request exists.
    pub fn accept_friend(&self, acceptor: &str, requester: &str) -> bool {
        let conn = self.0.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM pending WHERE from_user = ?1 AND to_user = ?2",
            params![requester, acceptor],
        ).unwrap_or(0);
        if n == 0 { return false; }
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

    /// Removes all relationship ties (friends and pending in both directions)
    /// between `a` and `b`.
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
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, reporter, reported, reason
                 FROM reports ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |row| {
            Ok(ReportRow {
                id:        row.get(0)?,
                timestamp: row.get(1)?,
                reporter:  row.get(2)?,
                reported:  row.get(3)?,
                reason:    row.get(4)?,
            })
        })
        .unwrap()
        .flatten()
        .collect()
    }
}

// ── Schema ─────────────────────────────────────────────────────────────────

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS players (
    username TEXT PRIMARY KEY,
    display  TEXT NOT NULL,
    token    TEXT NOT NULL
);

-- Bidirectional: both (a,b) and (b,a) are inserted on accept so either side
-- can query their friend list with a simple WHERE user_a = ?.
CREATE TABLE IF NOT EXISTS friends (
    user_a TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    user_b TEXT NOT NULL REFERENCES players(username) ON DELETE CASCADE,
    PRIMARY KEY (user_a, user_b)
);

-- Directional: (from_user → to_user) is a pending outbound request.
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
