// db.rs — SQLite-backed persistence via rusqlite.
//
// `Db` is a channel handle that dispatches to a dedicated worker thread that
// owns the `Connection` exclusively.  No mutex contention across handler
// threads: they send a request and optionally block on a one-shot response
// channel.  Multi-statement mutations are wrapped in explicit transactions so
// a crash mid-operation leaves the DB in a clean, consistent state.
//
// Username casing
// ───────────────
// Usernames are stored exactly as the player typed them at registration
// (e.g. "ILuv").  The primary key uses COLLATE NOCASE so lookups with any
// casing find the right row.  After any lookup callers should use the
// returned lowercase `PlayerRow.username` as the identity key for maps and packets.
// Stored casing is retained only as the default display name.

use std::sync::{Arc, mpsc};
use std::thread;

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

// ── Message protocol ───────────────────────────────────────────────────────

enum DbMsg {
    // Queries — caller blocks on the included sender for the result.
    GetPlayer           { username: String,                          tx: mpsc::Sender<Option<PlayerRow>> },
    GetDisplayName      { username: String,                          tx: mpsc::Sender<String> },
    PlayerExists        { username: String,                          tx: mpsc::Sender<bool> },
    GetFriends          { username: String,                          tx: mpsc::Sender<Vec<String>> },
    AreFriends          { a: String, b: String,                      tx: mpsc::Sender<bool> },
    GetPendingInbound   { username: String,                          tx: mpsc::Sender<Vec<String>> },
    GetPendingOutbound  { username: String,                          tx: mpsc::Sender<Vec<String>> },
    HasPending          { from: String, to: String,                  tx: mpsc::Sender<bool> },
    IsModerator         { username: String,                          tx: mpsc::Sender<bool> },
    GetReports          {                                            tx: mpsc::Sender<Vec<ReportRow>> },
    RunSql              { sql: String,                               tx: mpsc::Sender<String> },
    CleanupStalePending {                                            tx: mpsc::Sender<usize> },

    // Mutations with result — caller blocks for the bool/result.
    CreatePlayer        { username: String, token: String,           tx: mpsc::Sender<bool> },
    DeletePlayer        { username: String,                          tx: mpsc::Sender<bool> },
    AddFriendRequest    { from: String, to: String,                  tx: mpsc::Sender<bool> },
    AcceptFriend        { acceptor: String, requester: String,       tx: mpsc::Sender<bool> },
    SetModerator        { username: String, value: bool,             tx: mpsc::Sender<bool> },
    SetDisplayName      { username: String, display: String,         tx: mpsc::Sender<bool> },

    // Fire-and-forget writes — caller does not wait.
    RemoveFriend        { a: String, b: String },
    AddReport           { reporter: String, reported: String, reason: String },
}

// ── Db handle ──────────────────────────────────────────────────────────────

pub struct Db {
    tx: mpsc::Sender<DbMsg>,
}

impl Db {
    /// Opens (or creates) the SQLite database at `path`, runs any pending
    /// schema migrations, and spawns the DB worker thread.
    pub fn open(path: &str) -> SqlResult<Arc<Self>> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        migrate(&conn)?;
        conn.execute_batch(SCHEMA)?;

        let (tx, rx) = mpsc::channel::<DbMsg>();

        thread::Builder::new()
            .name("db-worker".to_string())
            .spawn(move || db_worker(conn, rx))
            .expect("failed to spawn db worker thread");

        Ok(Arc::new(Self { tx }))
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Sends a message and blocks until the worker replies.
    fn call<T>(&self, build: impl FnOnce(mpsc::Sender<T>) -> DbMsg) -> T {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(build(tx));
        rx.recv().expect("db worker died")
    }

    // ── Player queries ─────────────────────────────────────────────────────

    pub fn get_player(&self, username: &str) -> Option<PlayerRow> {
        self.call(|tx| DbMsg::GetPlayer { username: username.to_owned(), tx }).map(|mut row| {
            row.username = crate::utils::text::username_key(&row.username);
            row
        })
    }

    pub fn get_display_name(&self, username: &str) -> String {
        self.call(|tx| DbMsg::GetDisplayName { username: username.to_owned(), tx })
    }

    #[allow(dead_code)]
    pub fn is_moderator(&self, username: &str) -> bool {
        self.call(|tx| DbMsg::IsModerator { username: username.to_owned(), tx })
    }

    pub fn set_moderator(&self, username: &str, value: bool) -> bool {
        self.call(|tx| DbMsg::SetModerator { username: username.to_owned(), value, tx })
    }

    pub fn set_display_name(&self, username: &str, display: &str) -> bool {
        self.call(|tx| DbMsg::SetDisplayName {
            username: username.to_owned(),
            display:  display.to_owned(),
            tx,
        })
    }

    pub fn player_exists(&self, username: &str) -> bool {
        self.call(|tx| DbMsg::PlayerExists { username: username.to_owned(), tx })
    }

    pub fn create_player(&self, username: &str, token: &str) -> bool {
        self.call(|tx| DbMsg::CreatePlayer {
            username: username.to_owned(),
            token:    token.to_owned(),
            tx,
        })
    }

    pub fn delete_player(&self, username: &str) -> bool {
        self.call(|tx| DbMsg::DeletePlayer { username: username.to_owned(), tx })
    }

    // ── Friend queries ─────────────────────────────────────────────────────

    pub fn get_friends(&self, username: &str) -> Vec<String> {
        self.call(|tx| DbMsg::GetFriends { username: username.to_owned(), tx }).into_iter().map(|user| crate::utils::text::username_key(&user)).collect()
    }

    pub fn are_friends(&self, a: &str, b: &str) -> bool {
        self.call(|tx| DbMsg::AreFriends { a: a.to_owned(), b: b.to_owned(), tx })
    }

    pub fn get_pending_inbound(&self, username: &str) -> Vec<String> {
        self.call(|tx| DbMsg::GetPendingInbound { username: username.to_owned(), tx }).into_iter().map(|user| crate::utils::text::username_key(&user)).collect()
    }

    pub fn get_pending_outbound(&self, username: &str) -> Vec<String> {
        self.call(|tx| DbMsg::GetPendingOutbound { username: username.to_owned(), tx }).into_iter().map(|user| crate::utils::text::username_key(&user)).collect()
    }

    pub fn has_pending(&self, from: &str, to: &str) -> bool {
        self.call(|tx| DbMsg::HasPending { from: from.to_owned(), to: to.to_owned(), tx })
    }

    // ── Social graph mutations ─────────────────────────────────────────────

    pub fn add_friend_request(&self, from: &str, to: &str) -> bool {
        self.call(|tx| DbMsg::AddFriendRequest { from: from.to_owned(), to: to.to_owned(), tx })
    }

    pub fn accept_friend(&self, acceptor: &str, requester: &str) -> bool {
        self.call(|tx| DbMsg::AcceptFriend {
            acceptor:  acceptor.to_owned(),
            requester: requester.to_owned(),
            tx,
        })
    }

    /// Fire-and-forget: enqueues the removal and returns immediately.
    pub fn remove_friend(&self, a: &str, b: &str) {
        let _ = self.tx.send(DbMsg::RemoveFriend { a: a.to_owned(), b: b.to_owned() });
    }

    // ── Maintenance ───────────────────────────────────────────────────────

    pub fn cleanup_stale_pending(&self) -> usize {
        self.call(|tx| DbMsg::CleanupStalePending { tx })
    }

    // ── Reports ────────────────────────────────────────────────────────────

    /// Fire-and-forget: enqueues the report write and returns immediately.
    pub fn add_report(&self, reporter: &str, reported: &str, reason: &str) {
        let _ = self.tx.send(DbMsg::AddReport {
            reporter: reporter.to_owned(),
            reported: reported.to_owned(),
            reason:   reason.to_owned(),
        });
    }

    pub fn get_reports(&self) -> Vec<ReportRow> {
        self.call(|tx| DbMsg::GetReports { tx })
    }

    // ── Raw SQL (admin terminal) ───────────────────────────────────────────

    pub fn run_sql(&self, sql: &str) -> String {
        self.call(|tx| DbMsg::RunSql { sql: sql.to_owned(), tx })
    }
}

// ── Worker thread ──────────────────────────────────────────────────────────

fn db_worker(conn: Connection, rx: mpsc::Receiver<DbMsg>) {
    for msg in rx {
        match msg {
            DbMsg::GetPlayer { username, tx } => {
                let _ = tx.send(worker_get_player(&conn, &username));
            }
            DbMsg::GetDisplayName { username, tx } => {
                let _ = tx.send(worker_get_display_name(&conn, &username));
            }
            DbMsg::PlayerExists { username, tx } => {
                let _ = tx.send(worker_get_player(&conn, &username).is_some());
            }
            DbMsg::IsModerator { username, tx } => {
                let _ = tx.send(worker_is_moderator(&conn, &username));
            }
            DbMsg::SetModerator { username, value, tx } => {
                let exists = worker_get_player(&conn, &username).is_some();
                let ok = exists && conn.execute(
                    "UPDATE players SET is_moderator = ?1 WHERE username = ?2",
                    params![value as i64, username],
                ).map(|n| n > 0).unwrap_or(false);
                let _ = tx.send(ok);
            }
            DbMsg::SetDisplayName { username, display, tx } => {
                let exists = worker_get_player(&conn, &username).is_some();
                let val: Option<&str> = if display.is_empty() { None } else { Some(&display) };
                let ok = exists && conn.execute(
                    "UPDATE players SET display_name = ?1 WHERE username = ?2",
                    params![val, username],
                ).map(|n| n > 0).unwrap_or(false);
                let _ = tx.send(ok);
            }
            DbMsg::CreatePlayer { username, token, tx } => {
                let ok = conn.execute(
                    "INSERT OR IGNORE INTO players (username, token) VALUES (?1, ?2)",
                    params![username, token],
                ).map(|n| n > 0).unwrap_or(false);
                let _ = tx.send(ok);
            }
            DbMsg::DeletePlayer { username, tx } => {
                let ok = conn.execute(
                    "DELETE FROM players WHERE username = ?1",
                    params![username],
                ).map(|n| n > 0).unwrap_or(false);
                let _ = tx.send(ok);
            }
            DbMsg::GetFriends { username, tx } => {
                let mut stmt = conn.prepare(
                    "SELECT user_b FROM friends WHERE user_a = ?1 COLLATE NOCASE ORDER BY user_b",
                ).unwrap();
                let v: Vec<String> = stmt
                    .query_map(params![username], |row| row.get(0))
                    .unwrap().flatten().collect();
                let _ = tx.send(v);
            }
            DbMsg::AreFriends { a, b, tx } => {
                let ok = conn.query_row(
                    "SELECT 1 FROM friends WHERE user_a = ?1 COLLATE NOCASE AND user_b = ?2 COLLATE NOCASE",
                    params![a, b],
                    |_| Ok(()),
                ).is_ok();
                let _ = tx.send(ok);
            }
            DbMsg::GetPendingInbound { username, tx } => {
                let mut stmt = conn.prepare(
                    "SELECT from_user FROM pending WHERE to_user = ?1 COLLATE NOCASE ORDER BY from_user",
                ).unwrap();
                let v: Vec<String> = stmt
                    .query_map(params![username], |row| row.get(0))
                    .unwrap().flatten().collect();
                let _ = tx.send(v);
            }
            DbMsg::GetPendingOutbound { username, tx } => {
                let mut stmt = conn.prepare(
                    "SELECT to_user FROM pending WHERE from_user = ?1 COLLATE NOCASE ORDER BY to_user",
                ).unwrap();
                let v: Vec<String> = stmt
                    .query_map(params![username], |row| row.get(0))
                    .unwrap().flatten().collect();
                let _ = tx.send(v);
            }
            DbMsg::HasPending { from, to, tx } => {
                let ok = conn.query_row(
                    "SELECT 1 FROM pending WHERE from_user = ?1 COLLATE NOCASE AND to_user = ?2 COLLATE NOCASE",
                    params![from, to],
                    |_| Ok(()),
                ).is_ok();
                let _ = tx.send(ok);
            }
            DbMsg::AddFriendRequest { from, to, tx } => {
                let ok = worker_add_friend_request(&conn, &from, &to);
                let _ = tx.send(ok);
            }
            DbMsg::AcceptFriend { acceptor, requester, tx } => {
                let ok = worker_accept_friend(&conn, &acceptor, &requester);
                let _ = tx.send(ok);
            }
            DbMsg::RemoveFriend { a, b } => {
                worker_remove_friend(&conn, &a, &b);
            }
            DbMsg::CleanupStalePending { tx } => {
                let n = conn.execute(
                    "DELETE FROM pending WHERE EXISTS (
                        SELECT 1 FROM friends
                        WHERE friends.user_a = pending.from_user COLLATE NOCASE
                          AND friends.user_b = pending.to_user COLLATE NOCASE
                    )",
                    [],
                ).unwrap_or(0);
                let _ = tx.send(n);
            }
            DbMsg::AddReport { reporter, reported, reason } => {
                let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                conn.execute(
                    "INSERT INTO reports (timestamp, reporter, reported, reason) VALUES (?1, ?2, ?3, ?4)",
                    params![ts, reporter, reported, reason],
                ).unwrap_or(0);
            }
            DbMsg::GetReports { tx } => {
                let mut stmt = conn.prepare(
                    "SELECT id, timestamp, reporter, reported, reason FROM reports ORDER BY id",
                ).unwrap();
                let v: Vec<ReportRow> = stmt.query_map([], |row| Ok(ReportRow {
                    id:        row.get(0)?,
                    timestamp: row.get(1)?,
                    reporter:  row.get(2)?,
                    reported:  row.get(3)?,
                    reason:    row.get(4)?,
                })).unwrap().flatten().collect();
                let _ = tx.send(v);
            }
            DbMsg::RunSql { sql, tx } => {
                let _ = tx.send(worker_run_sql(&conn, &sql));
            }
        }
    }
}

// ── Worker helpers ─────────────────────────────────────────────────────────

fn worker_get_player(conn: &Connection, username: &str) -> Option<PlayerRow> {
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

fn worker_get_display_name(conn: &Connection, username: &str) -> String {
    let row = conn.query_row(
        "SELECT display_name, COALESCE(is_moderator, 0), username FROM players WHERE username = ?1",
        params![username],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?)),
    ).ok();
    let (raw, is_mod) = match row {
        Some((r, m, original)) => (r.unwrap_or(original), m != 0),
        None         => (username.to_string(), false),
    };
    if is_mod { format!("{} <color=#FFFF00>★</color>", raw) } else { raw }
}

fn worker_is_moderator(conn: &Connection, username: &str) -> bool {
    conn.query_row(
        "SELECT COALESCE(is_moderator, 0) FROM players WHERE username = ?1",
        params![username],
        |row| row.get::<_, i64>(0),
    ).ok().map(|v| v != 0).unwrap_or(false)
}

fn worker_add_friend_request(conn: &Connection, from: &str, to: &str) -> bool {
    let from_exists = worker_get_player(conn, from).is_some();
    let to_exists   = worker_get_player(conn, to).is_some();
    if !from_exists || !to_exists { return false; }

    let already_friends = conn.query_row(
        "SELECT 1 FROM friends WHERE user_a = ?1 COLLATE NOCASE AND user_b = ?2 COLLATE NOCASE",
        params![from, to], |_| Ok(()),
    ).is_ok();
    let has_pending = conn.query_row(
        "SELECT 1 FROM pending WHERE from_user = ?1 COLLATE NOCASE AND to_user = ?2 COLLATE NOCASE",
        params![from, to], |_| Ok(()),
    ).is_ok();
    if already_friends || has_pending { return false; }

    conn.execute(
        "INSERT OR IGNORE INTO pending (from_user, to_user) VALUES (?1, ?2)",
        params![from, to],
    ).map(|n| n > 0).unwrap_or(false)
}

fn worker_accept_friend(conn: &Connection, acceptor: &str, requester: &str) -> bool {
    conn.execute_batch("BEGIN;").ok();

    let n = conn.execute(
        "DELETE FROM pending WHERE from_user = ?1 COLLATE NOCASE AND to_user = ?2 COLLATE NOCASE",
        params![requester, acceptor],
    ).unwrap_or(0);

    if n == 0 {
        conn.execute_batch("ROLLBACK;").ok();
        return false;
    }

    // Remove any reverse-direction pending to avoid ghost outbound entries.
    conn.execute(
        "DELETE FROM pending WHERE from_user = ?1 COLLATE NOCASE AND to_user = ?2 COLLATE NOCASE",
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

    conn.execute_batch("COMMIT;").ok();
    true
}

fn worker_remove_friend(conn: &Connection, a: &str, b: &str) {
    conn.execute_batch("BEGIN;").ok();
    conn.execute(
        "DELETE FROM friends
         WHERE (user_a = ?1 COLLATE NOCASE AND user_b = ?2 COLLATE NOCASE) OR (user_a = ?2 COLLATE NOCASE AND user_b = ?1 COLLATE NOCASE)",
        params![a, b],
    ).unwrap_or(0);
    conn.execute(
        "DELETE FROM pending
         WHERE (from_user = ?1 COLLATE NOCASE AND to_user = ?2 COLLATE NOCASE) OR (from_user = ?2 COLLATE NOCASE AND to_user = ?1 COLLATE NOCASE)",
        params![a, b],
    ).unwrap_or(0);
    conn.execute_batch("COMMIT;").ok();
}

fn worker_run_sql(conn: &Connection, sql: &str) -> String {
    match conn.prepare(sql) {
        Err(e) => return format!("[!] SQL error: {}\n", e),
        Ok(mut stmt) => {
            let col_names: Vec<String> = stmt.column_names()
                .iter().map(|s| s.to_string()).collect();

            if col_names.is_empty() {
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

// ── Migration ──────────────────────────────────────────────────────────────

fn migrate(conn: &Connection) -> SqlResult<()> {
    let cols: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(players)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .flatten()
            .collect()
    };

    if cols.is_empty() { return Ok(()); } // Fresh database: SCHEMA creates the table.

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
        return Ok(());
    }

    if !cols.iter().any(|c| c == "display_name") {
        println!("[DB] Migrating schema v2→v3: adding display_name column …");
        conn.execute_batch("ALTER TABLE players ADD COLUMN display_name TEXT;")?;
        println!("[DB] Migration v2→v3 complete.");
    }

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

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn legacy_mixed_case_accounts_have_lowercase_identities_and_original_displays() {
        let db = Db::open(":memory:").unwrap();
        assert!(db.create_player("User", "token"));
        assert!(db.create_player("user2", "token2"));
        assert_eq!(db.get_player("USER").unwrap().username, "user");
        assert_eq!(db.get_player("user2").unwrap().username, "user2");
        assert_eq!(db.get_display_name("user"), "User");
        assert!(db.set_display_name("USER", "<color=yellow>User</color>"));
        assert_eq!(db.get_player("user").unwrap().username, "user");
        assert_eq!(db.get_display_name("user"), "<color=yellow>User</color>");
    }

    #[test]
    fn legacy_friend_edges_work_after_session_names_are_normalized() {
        let db = Db::open(":memory:").unwrap();
        assert!(db.create_player("User", "token"));
        assert!(db.create_player("User2", "token2"));
        assert!(db.add_friend_request("User", "User2"));
        assert_eq!(db.get_pending_inbound("user2"), vec!["user"]);
        assert_eq!(db.get_pending_outbound("user"), vec!["user2"]);
        assert!(db.has_pending("USER", "USER2"));
        assert!(db.accept_friend("user2", "user"));
        assert!(db.are_friends("USER", "USER2"));
        assert_eq!(db.get_friends("USER"), vec!["user2"]);
        db.remove_friend("USER", "USER2");
        assert!(db.get_friends("user").is_empty());
    }
}
