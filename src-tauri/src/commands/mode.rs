use serde::{Deserialize, Serialize};

/// The user's current operating mode — determines which incoming signals are interruptions.
///
/// Work:     Discord/iMessage → queued. VS Code → expected (high-focus context).
/// Personal: Discord/iMessage → allowed. VS Code → mild interruption signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkLifeMode {
    Work,
    Personal,
}

impl Default for WorkLifeMode {
    fn default() -> Self {
        WorkLifeMode::Work
    }
}

impl std::fmt::Display for WorkLifeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkLifeMode::Work => write!(f, "work"),
            WorkLifeMode::Personal => write!(f, "personal"),
        }
    }
}

/// Get the current Work/Life mode.
///
/// Stub: reads from in-memory default. Full implementation reads from AppState + SQLite.
#[tauri::command]
pub fn get_mode() -> Result<WorkLifeMode, String> {
    Ok(WorkLifeMode::default())
}

/// Set the Work/Life mode.
///
/// Stub: no-op. Full implementation updates AppState + persists to SQLite `app_state` table.
/// The MCP negotiator reads from AppState on every intercept, so this takes effect immediately.
#[tauri::command]
pub fn set_mode(mode: WorkLifeMode) -> Result<(), String> {
    // TODO: update tauri::State<AppState> and persist to DB
    let _ = mode;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_work() {
        assert_eq!(WorkLifeMode::default(), WorkLifeMode::Work);
    }

    #[test]
    fn mode_serializes_lowercase() {
        let json = serde_json::to_string(&WorkLifeMode::Personal).unwrap();
        assert_eq!(json, "\"personal\"");
    }
}
