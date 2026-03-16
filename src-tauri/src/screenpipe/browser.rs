/// Browser tab context extractor (Chrome / Arc).
///
/// Parses browser window titles to extract page title and applies URL redaction
/// rules from the privacy config. Full URL extraction requires querying Screenpipe
/// for address bar OCR frames.
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct BrowserTab {
    /// Page title extracted from window title or Screenpipe OCR
    pub title: String,
    /// URL — may be [REDACTED] if it matches a privacy_config redact pattern
    pub url: String,
    pub is_active: bool,
}

/// Try to extract the active tab title from a browser window title.
///
/// Chrome format:  `Page Title - Google Chrome`
/// Arc format:     `Page Title — Arc`
/// Safari format:  `Page Title — Safari`
pub fn extract_active_tab_title(window_title: &str) -> Option<String> {
    let browsers = [" - Google Chrome", " — Arc", " — Safari", " - Chromium"];
    for suffix in &browsers {
        if let Some(title) = window_title.strip_suffix(suffix) {
            return Some(title.trim().to_string());
        }
    }
    None
}

/// Redact a URL if it matches any of the given patterns.
/// Returns `[REDACTED]` if matched, otherwise returns the original URL.
pub fn redact_url(url: &str, patterns: &[String]) -> String {
    let lower = url.to_lowercase();
    if patterns.iter().any(|p| lower.contains(p.as_str())) {
        "[REDACTED]".to_string()
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_chrome_title() {
        let title = extract_active_tab_title("GitHub - Focus Engine - Google Chrome");
        assert_eq!(title.unwrap(), "GitHub - Focus Engine");
    }

    #[test]
    fn extracts_arc_title() {
        let title = extract_active_tab_title("Focus Engine Architecture — Arc");
        assert_eq!(title.unwrap(), "Focus Engine Architecture");
    }

    #[test]
    fn redacts_bank_url() {
        let patterns = vec!["bank".to_string(), "paypal".to_string()];
        assert_eq!(redact_url("https://chase.bank.com/login", &patterns), "[REDACTED]");
    }

    #[test]
    fn passes_safe_url() {
        let patterns = vec!["bank".to_string()];
        assert_eq!(redact_url("https://github.com/focus-engine", &patterns), "https://github.com/focus-engine");
    }
}
