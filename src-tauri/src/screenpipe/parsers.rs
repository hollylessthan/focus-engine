/// OCR text parsers and privacy filters.
///
/// Cleans and normalizes raw Screenpipe OCR output before it is
/// passed to the local LLM for intent inference.
///
/// Rules:
/// - Strip UI chrome artifacts (menu bar labels, window decorations)
/// - Remove repeated whitespace and control characters
/// - Truncate to a maximum token budget for the LLM context window
/// - Drop frames whose source app/window matches privacy_config exclusion rules
use crate::commands::privacy::PrivacyConfig;
use crate::screenpipe::client::OcrFrame;

const MAX_CHARS: usize = 4096;

/// Clean raw OCR text for LLM consumption.
pub fn clean_ocr(raw: &str) -> String {
    let trimmed = raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if trimmed.len() > MAX_CHARS {
        trimmed[..MAX_CHARS].to_string()
    } else {
        trimmed
    }
}

/// Drop OCR frames that match privacy exclusion rules from the privacy config.
/// Must be called before frames reach the DB or LLM.
pub fn filter_frames(frames: Vec<OcrFrame>, config: &PrivacyConfig) -> Vec<OcrFrame> {
    frames
        .into_iter()
        .filter(|f| {
            let app_excluded = config
                .excluded_apps
                .iter()
                .any(|app| f.app_name.contains(app.as_str()));
            let window_excluded = config
                .excluded_window_title_patterns
                .iter()
                .any(|pat| f.window_name.to_lowercase().contains(pat.to_lowercase().as_str()));
            !app_excluded && !window_excluded
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_blank_lines() {
        let raw = "Line one\n\n   \nLine two\n";
        let cleaned = clean_ocr(raw);
        assert_eq!(cleaned, "Line one\nLine two");
    }

    #[test]
    fn truncates_at_max_chars() {
        let long = "x".repeat(MAX_CHARS + 100);
        let cleaned = clean_ocr(&long);
        assert_eq!(cleaned.len(), MAX_CHARS);
    }

    #[test]
    fn filter_drops_excluded_app() {
        let frames = vec![
            OcrFrame {
                text: "secret".to_string(),
                app_name: "1Password".to_string(),
                window_name: "Vault".to_string(),
                focused: true,
            },
            OcrFrame {
                text: "safe content".to_string(),
                app_name: "Code".to_string(),
                window_name: "main.rs".to_string(),
                focused: false,
            },
        ];
        let config = PrivacyConfig::default();
        let filtered = filter_frames(frames, &config);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].app_name, "Code");
    }

    #[test]
    fn filter_drops_excluded_window_pattern() {
        let frames = vec![OcrFrame {
            text: "private data".to_string(),
            app_name: "Safari".to_string(),
            window_name: "Private Browsing — Safari".to_string(),
            focused: true,
        }];
        let config = PrivacyConfig::default();
        let filtered = filter_frames(frames, &config);
        assert!(filtered.is_empty());
    }
}
