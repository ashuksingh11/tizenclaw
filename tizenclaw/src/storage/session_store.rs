//! Session store — manages conversation sessions in SQLite.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::sqlite;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_requests: i64,
    pub entries: Vec<Value>,
}

pub struct SessionStore {
    db: rusqlite::Connection,
}

impl SessionStore {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = sqlite::open_database(db_path).map_err(|e| format!("DB open: {}", e))?;
        let store = SessionStore { db };
        store.init_tables().map_err(|e| format!("DB init: {}", e))?;
        Ok(store)
    }

    fn init_tables(&self) -> rusqlite::Result<()> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT DEFAULT (datetime('now')),
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                prompt_tokens INTEGER DEFAULT 0,
                completion_tokens INTEGER DEFAULT 0,
                model TEXT DEFAULT '',
                timestamp TEXT DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_usage_session ON token_usage(session_id);
            CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON token_usage(timestamp);"
        )
    }

    pub fn ensure_session(&self, session_id: &str) {
        let _ = self.db.execute(
            "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
            params![session_id],
        );
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) {
        self.ensure_session(session_id);
        let _ = self.db.execute(
            "INSERT INTO messages (session_id, role, content) VALUES (?1, ?2, ?3)",
            params![session_id, role, content],
        );
        let _ = self.db.execute(
            "UPDATE sessions SET updated_at = datetime('now') WHERE id = ?1",
            params![session_id],
        );
    }

    pub fn get_messages(&self, session_id: &str, limit: usize) -> Vec<SessionMessage> {
        let mut stmt = match self.db.prepare(
            "SELECT role, content, timestamp FROM messages
             WHERE session_id = ?1 ORDER BY id DESC LIMIT ?2"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            Ok(SessionMessage {
                role: row.get(0)?,
                text: row.get(1)?,
                timestamp: row.get(2)?,
            })
        });
        match rows {
            Ok(iter) => {
                let mut msgs: Vec<SessionMessage> = iter.filter_map(|r| r.ok()).collect();
                msgs.reverse();
                msgs
            }
            Err(_) => vec![],
        }
    }

    pub fn record_usage(&self, session_id: &str, prompt_tokens: i32, completion_tokens: i32, model: &str) {
        let _ = self.db.execute(
            "INSERT INTO token_usage (session_id, prompt_tokens, completion_tokens, model)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, prompt_tokens, completion_tokens, model],
        );
    }

    pub fn load_token_usage(&self, session_id: &str) -> TokenUsage {
        let mut usage = TokenUsage::default();
        if let Ok(mut stmt) = self.db.prepare(
            "SELECT COALESCE(SUM(prompt_tokens),0), COALESCE(SUM(completion_tokens),0), COUNT(*)
             FROM token_usage WHERE session_id = ?1"
        ) {
            let _ = stmt.query_row(params![session_id], |row| {
                usage.total_prompt_tokens = row.get(0)?;
                usage.total_completion_tokens = row.get(1)?;
                usage.total_requests = row.get(2)?;
                Ok(())
            });
        }
        usage
    }

    pub fn load_daily_usage(&self, date: &str) -> TokenUsage {
        let date_filter = if date.is_empty() {
            "date('now')".to_string()
        } else {
            format!("'{}'", date)
        };
        let sql = format!(
            "SELECT COALESCE(SUM(prompt_tokens),0), COALESCE(SUM(completion_tokens),0), COUNT(*)
             FROM token_usage WHERE date(timestamp) = {}", date_filter
        );
        let mut usage = TokenUsage::default();
        if let Ok(mut stmt) = self.db.prepare(&sql) {
            let _ = stmt.query_row([], |row| {
                usage.total_prompt_tokens = row.get(0)?;
                usage.total_completion_tokens = row.get(1)?;
                usage.total_requests = row.get(2)?;
                Ok(())
            });
        }
        usage
    }
}
