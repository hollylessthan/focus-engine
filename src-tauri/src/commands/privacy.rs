use serde::{Deserialize, Serialize};

use crate::AppState;

/// User-editable privacy exclusion rules.
/// Loaded from `$APPLOCALDATA/focus-engine/privacy_config.json` on startup.
/// Any OCR frame whose source window matches these rules is dropped before DB/LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// App names to fully exclude from capture (e.g. "1Password", "Keychain Access").
    pub excluded_apps: Vec<String>,
    /// Window title substrings that trigger exclusion (e.g. "Private", "Incognito").
    pub excluded_window_title_patterns: Vec<String>,
    /// URL substrings that trigger redaction in browser tab capture.
    pub redact_urls_matching: Vec<String>,
    pub version: u32,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        PrivacyConfig {
            excluded_apps: vec![
                "1Password".to_string(),
                "Keychain Access".to_string(),
                "Finder".to_string(),
            ],
            excluded_window_title_patterns: vec![
                "Private".to_string(),
                "Incognito".to_string(),
                "1Password".to_string(),
            ],
            redact_urls_matching: vec![
                "bank".to_string(),
                "paypal".to_string(),
                "health".to_string(),
            ],
            version: 1,
        }
    }
}

/// Toggle Incognito Mode.
///
/// When active:
/// - Screenpipe polling stops (freeze_frame returns an error)
/// - Any buffered OCR is zeroized
/// - Tray icon turns red (TODO: tray icon color change via Tauri tray plugin)
#[tauri::command]
pub fn toggle_incognito(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let new_state = !state.incognito();
    state.set_incognito(new_state);
    Ok(new_state)
}

/// Get current incognito state.
#[tauri::command]
pub fn get_incognito_status(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(state.incognito())
}

/// Get the active privacy configuration.
#[tauri::command]
pub fn get_privacy_config(state: tauri::State<'_, AppState>) -> Result<PrivacyConfig, String> {
    let config = state.privacy_config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Update the privacy config in memory.
/// TODO (Milestone 3): persist to $APPLOCALDATA/focus-engine/privacy_config.json.
#[tauri::command]
pub fn update_privacy_config(
    config: PrivacyConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut current = state.privacy_config.lock().map_err(|e| e.to_string())?;
    *current = config;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_excludes_1password() {
        let cfg = PrivacyConfig::default();
        assert!(cfg.excluded_apps.contains(&"1Password".to_string()));
    }

    #[test]
    fn default_config_redacts_bank_urls() {
        let cfg = PrivacyConfig::default();
        assert!(cfg.redact_urls_matching.contains(&"bank".to_string()));
    }
}
