use serde::{Deserialize, Serialize};

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
/// - Screenpipe polling stops immediately
/// - Any buffered OCR is zeroized
/// - Tray icon turns red (TODO: tray icon color change)
///
/// Stub: returns current state. Full implementation updates AppState.incognito_active.
#[tauri::command]
pub fn toggle_incognito() -> Result<bool, String> {
    // TODO: flip AppState.incognito_active, stop/start Screenpipe polling, zeroize buffers
    Ok(false) // placeholder: always returns false (not active)
}

/// Get current incognito state.
#[tauri::command]
pub fn get_incognito_status() -> Result<bool, String> {
    // TODO: read from AppState.incognito_active
    Ok(false)
}

/// Get the active privacy configuration.
#[tauri::command]
pub fn get_privacy_config() -> Result<PrivacyConfig, String> {
    // TODO: read from $APPLOCALDATA/focus-engine/privacy_config.json
    Ok(PrivacyConfig::default())
}

/// Update the privacy config and persist to disk.
#[tauri::command]
pub fn update_privacy_config(config: PrivacyConfig) -> Result<(), String> {
    // TODO: validate, serialize to JSON, write to $APPLOCALDATA/focus-engine/privacy_config.json
    let _ = config;
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
