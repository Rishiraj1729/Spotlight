use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

/// A single indexed filesystem entry (file or folder). `is_dir` is implied
/// by which `search_*` method returned the entry, but kept on the struct so
/// callers don't have to track that context separately.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

/// Thin wrapper around a single SQLite connection shared across the app.
/// `rusqlite::Connection` isn't `Sync`, so we guard it with a `Mutex` - all
/// operations here are short (indexed lookups / single-row writes), so lock
/// contention is not a concern at our result-count scale.
pub struct Db {
    conn: Mutex<Connection>,
}

/// A small "used this recently/often" boost folded into ranking, capped so
/// it nudges ties rather than overriding a clearly better textual match.
const MAX_FRECENCY_BONUS: f32 = 8.0;

impl Db {
    pub fn open(data_dir: &Path) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let db_path: PathBuf = data_dir.join("spotlight.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 2000;
            ",
        )?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                is_dir INTEGER NOT NULL,
                modified INTEGER NOT NULL DEFAULT 0
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
                name,
                path,
                content='entries',
                content_rowid='id',
                tokenize='trigram'
            );

            CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entries_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;

            CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, name, path) VALUES ('delete', old.id, old.name, old.path);
            END;

            CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, name, path) VALUES ('delete', old.id, old.name, old.path);
                INSERT INTO entries_fts(rowid, name, path) VALUES (new.id, new.name, new.path);
            END;

            CREATE TABLE IF NOT EXISTS launch_history (
                result_id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                use_count INTEGER NOT NULL DEFAULT 0,
                last_used_at INTEGER NOT NULL DEFAULT 0
            );
            ",
        )?;
        Ok(())
    }

    /// Insert or refresh a single filesystem entry. Used by both the
    /// initial background crawl and the incremental file-watcher.
    pub fn upsert_entry(&self, path: &str, name: &str, is_dir: bool, modified: i64) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO entries (path, name, is_dir, modified) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET name = excluded.name, is_dir = excluded.is_dir, modified = excluded.modified",
            (path, name, is_dir as i64, modified),
        );
    }

    pub fn remove_entry(&self, path: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM entries WHERE path = ?1", [path]);
    }

    fn search(&self, raw_query: &str, is_dir: bool, limit: usize) -> Vec<IndexEntry> {
        let Some(match_expr) = build_match_expr(raw_query) else {
            return Vec::new();
        };
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT e.path, e.name FROM entries_fts f
             JOIN entries e ON e.id = f.rowid
             WHERE f.entries_fts MATCH ?1 AND e.is_dir = ?2
             ORDER BY rank
             LIMIT ?3",
        ) {
            Ok(stmt) => stmt,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map((match_expr, is_dir as i64, limit as i64), |row| {
            Ok(IndexEntry {
                path: row.get(0)?,
                name: row.get(1)?,
                is_dir,
            })
        });

        match rows {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn search_files(&self, query: &str, limit: usize) -> Vec<IndexEntry> {
        self.search(query, false, limit)
    }

    pub fn search_folders(&self, query: &str, limit: usize) -> Vec<IndexEntry> {
        self.search(query, true, limit)
    }

    pub fn record_launch(&self, result_id: &str, provider: &str) {
        let now = now_unix();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO launch_history (result_id, provider, use_count, last_used_at) VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(result_id) DO UPDATE SET use_count = use_count + 1, last_used_at = ?3",
            (result_id, provider, now),
        );
    }

    /// Small ranking boost from usage frequency + recency. Deliberately
    /// capped and cheap (single indexed lookup) since it runs once per
    /// result on every keystroke.
    pub fn frecency_bonus(&self, result_id: &str) -> f32 {
        let conn = self.conn.lock().unwrap();
        let row: Option<(i64, i64)> = conn
            .query_row(
                "SELECT use_count, last_used_at FROM launch_history WHERE result_id = ?1",
                [result_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        let Some((use_count, last_used_at)) = row else {
            return 0.0;
        };

        let age_days = ((now_unix() - last_used_at).max(0) as f32) / 86_400.0;
        let recency_factor = (1.0 - (age_days / 30.0).min(1.0)).max(0.0);
        let frequency_factor = (use_count as f32).min(10.0) / 10.0;

        MAX_FRECENCY_BONUS * (0.5 * recency_factor + 0.5 * frequency_factor)
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Turns free-text user input into an FTS5 MATCH expression. Each token is
/// treated as a required substring match (trigram tokenizer already gives
/// us substring/fuzzy-ish matching without needing a trailing `*`).
fn build_match_expr(raw_query: &str) -> Option<String> {
    let tokens: Vec<String> = raw_query
        .split_whitespace()
        .map(|t| t.replace('"', ""))
        .filter(|t| !t.is_empty())
        .collect();

    if tokens.is_empty() {
        return None;
    }

    Some(
        tokens
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(" AND "),
    )
}
