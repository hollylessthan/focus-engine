-- Focus Engine — SQLite schema
-- All tables use IF NOT EXISTS for safe re-runs on startup.
-- Managed by db/store.rs::migrate()

CREATE TABLE IF NOT EXISTS snapshots (
    id                   TEXT PRIMARY KEY,
    timestamp            INTEGER NOT NULL,
    active_intent        TEXT NOT NULL,
    next_immediate_action TEXT NOT NULL,
    cognitive_load_score REAL NOT NULL,
    cursor_x             INTEGER NOT NULL,
    cursor_y             INTEGER NOT NULL,
    -- Raw OCR stored encrypted at rest (SQLCipher handles this at the file level)
    visual_context_ocr   TEXT NOT NULL,
    -- JSON array of WindowState objects
    open_windows_json    TEXT NOT NULL
);

-- Persists app-level key/value state (e.g. work/life mode) across restarts.
CREATE TABLE IF NOT EXISTS app_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS interruptions (
    id         TEXT PRIMARY KEY,
    source     TEXT NOT NULL,  -- 'slack' | 'jira' | 'email'
    preview    TEXT NOT NULL,  -- truncated, non-identifying summary
    priority   INTEGER NOT NULL DEFAULT 0,
    queued_at  INTEGER NOT NULL,
    handled_at INTEGER
);
