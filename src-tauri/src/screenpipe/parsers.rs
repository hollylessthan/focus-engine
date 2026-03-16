/// OCR text parsers — stub.
///
/// Cleans and normalizes raw Screenpipe OCR output before it is
/// passed to the local LLM for intent inference.
///
/// Rules:
/// - Strip UI chrome artifacts (menu bar labels, window decorations)
/// - Remove repeated whitespace and control characters
/// - Truncate to a maximum token budget for the LLM context window

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
}
