use serde::{Deserialize, Serialize};

/// Configuration for the local LLM inference engine.
/// Stored at `$APPLOCALDATA/focus-engine/ai_config.json`.
///
/// Uses Ollama as the inference backend (http://127.0.0.1:11434).
/// Zero-cloud compliant — all traffic stays on localhost.
///
/// Setup:
///   1. `brew install ollama && ollama serve`
///   2. `ollama pull llama3.2:3b`  (or any instruct model)
///   3. Set `ollama_model` to the model name, e.g. "llama3.2:3b"
///   4. Leave `ollama_model` empty to use heuristic analysis only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Ollama model name. Must match output of `ollama list`.
    /// Leave empty to disable LLM and use heuristics only.
    /// Examples: "llama3.2:3b", "phi3:mini", "mistral:7b"
    pub ollama_model: String,
    /// Ollama API base URL. Change only if Ollama runs on a non-default port.
    pub ollama_url: String,
    /// Maximum tokens to generate per inference call.
    pub max_tokens: u32,
    /// Sampling temperature. Lower = more deterministic JSON output.
    pub temperature: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            ollama_model: String::new(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            max_tokens: 256,
            temperature: 0.15,
        }
    }
}

impl AiConfig {
    /// Load from a JSON file, falling back to defaults on any error.
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// True if an Ollama model name is configured.
    pub fn is_enabled(&self) -> bool {
        !self.ollama_model.is_empty()
    }
}
