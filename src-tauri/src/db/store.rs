use rusqlite::{params, Connection};
use std::sync::Mutex;

use crate::commands::{
    mode::WorkLifeMode,
    snapshot::{ContextSnapshot, WindowState},
};

/// SQLite store — wraps a `rusqlite::Connection` behind a `Mutex` for shared access.
///
/// Production note: replace `Connection::open` with SQLCipher key derivation:
///   conn.execute_batch(&format!("PRAGMA key = '{}';", keychain_key))?;
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Open (or create) the database at `path` and run all migrations.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let store = Store {
            conn: Mutex::new(conn),
        };
        store.migrate()?;
        Ok(store)
    }

    /// Execute all `CREATE TABLE IF NOT EXISTS` statements from schema.sql.
    fn migrate(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(include_str!("schema.sql"))
            .map_err(|e| e.to_string())
    }

    /// Persist a snapshot. Uses INSERT OR REPLACE so re-triggering the same ID is idempotent.
    pub fn save_snapshot(&self, s: &ContextSnapshot) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let windows_json =
            serde_json::to_string(&s.open_windows).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO snapshots
             (id, timestamp, active_intent, next_immediate_action,
              cognitive_load_score, cursor_x, cursor_y,
              visual_context_ocr, open_windows_json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                s.id,
                s.timestamp,
                s.active_intent,
                s.next_immediate_action,
                s.cognitive_load_score as f64,
                s.cursor_position.0,
                s.cursor_position.1,
                s.visual_context_ocr,
                windows_json,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Return up to 50 most recent snapshots, newest first.
    pub fn list_snapshots(&self) -> Result<Vec<ContextSnapshot>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, active_intent, next_immediate_action,
                        cognitive_load_score, cursor_x, cursor_y,
                        visual_context_ocr, open_windows_json
                 FROM snapshots ORDER BY timestamp DESC LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i32>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut snapshots = Vec::new();
        for row in rows {
            let (id, timestamp, active_intent, next_immediate_action,
                 cognitive_load_score, cursor_x, cursor_y,
                 visual_context_ocr, windows_json) = row.map_err(|e| e.to_string())?;

            let open_windows: Vec<WindowState> =
                serde_json::from_str(&windows_json).unwrap_or_default();

            snapshots.push(ContextSnapshot {
                id,
                timestamp,
                active_intent,
                next_immediate_action,
                cognitive_load_score: cognitive_load_score as f32,
                cursor_position: (cursor_x, cursor_y),
                visual_context_ocr,
                open_windows,
            });
        }
        Ok(snapshots)
    }

    /// Load the persisted Work/Life mode, defaulting to Work if no row exists.
    pub fn get_mode(&self) -> Result<WorkLifeMode, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT value FROM app_state WHERE key = 'mode'",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(val) => match val.as_str() {
                "work" => Ok(WorkLifeMode::Work),
                "personal" => Ok(WorkLifeMode::Personal),
                _ => Ok(WorkLifeMode::default()),
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(WorkLifeMode::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Persist the Work/Life mode so it survives restarts.
    pub fn set_mode(&self, mode: WorkLifeMode) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO app_state (key, value) VALUES ('mode', ?1)",
            params![mode.to_string()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
