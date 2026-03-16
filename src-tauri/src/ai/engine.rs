/// Local LLM inference engine — stub.
///
/// Full implementation: load Llama-3-8B via `candle` or `llama-cpp-rs`,
/// run inference locally. No network calls. No API keys.
///
/// Responsibilities:
/// - Score cognitive complexity from OCR text + window list
/// - Infer `active_intent` from recent Screenpipe history
/// - Generate `next_immediate_action` breadcrumb
pub struct LocalEngine;

impl LocalEngine {
    pub fn new() -> Self {
        LocalEngine
    }

    /// Estimate cognitive load from window count and app types.
    /// Returns 0.0 (minimal) to 1.0 (maximum).
    pub fn score_cognitive_load(&self, window_count: usize, has_terminal: bool, has_ide: bool) -> f32 {
        let base = (window_count as f32 * 0.1).min(0.5);
        let bonus = match (has_ide, has_terminal) {
            (true, true) => 0.4,
            (true, false) | (false, true) => 0.2,
            _ => 0.0,
        };
        (base + bonus).min(1.0)
    }

    /// Infer the user's active intent from OCR text (stub).
    pub fn infer_intent(&self, _ocr_text: &str) -> String {
        // TODO: run prompt through local LLM
        "Inferred intent placeholder — LLM not yet connected.".to_string()
    }
}

impl Default for LocalEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cognitive_load_bounds() {
        let engine = LocalEngine::new();
        let score = engine.score_cognitive_load(5, true, true);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn ide_plus_terminal_is_high_load() {
        let engine = LocalEngine::new();
        let score = engine.score_cognitive_load(2, true, true);
        assert!(score >= 0.6, "IDE+Terminal should produce high load score");
    }
}
