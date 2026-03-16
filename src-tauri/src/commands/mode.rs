use serde::{Deserialize, Serialize};

use crate::AppState;

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

/// Get the current Work/Life mode from AppState.
#[tauri::command]
pub fn get_mode(state: tauri::State<'_, AppState>) -> Result<WorkLifeMode, String> {
    let mode = state.mode.lock().map_err(|e| e.to_string())?;
    Ok(*mode)
}

/// Set the Work/Life mode in AppState and persist it to SQLite.
/// The MCP negotiator reads from AppState on every intercept, so this takes effect immediately.
#[tauri::command]
pub fn set_mode(mode: WorkLifeMode, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut current = state.mode.lock().map_err(|e| e.to_string())?;
    *current = mode;
    drop(current);
    if let Some(db) = state.db.get() {
        db.set_mode(mode)?;
    }
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
