/// Local LLM inference engine — uses Ollama's localhost HTTP API.
///
/// Zero-cloud: all requests go to 127.0.0.1:11434.
/// Requires Ollama to be running: `ollama serve && ollama pull llama3.2:3b`
///
/// When Ollama is offline or `ollama_model` is empty, falls back to
/// keyword-aware heuristic analysis with no user-visible error.
use serde::{Deserialize, Serialize};

use crate::ai::config::AiConfig;

/// Output of one analysis pass (LLM or heuristic).
#[derive(Debug, Clone)]
pub struct LlmAnalysis {
    /// One sentence describing what the user was working on.
    pub intent: String,
    /// Concrete next micro-step to reduce return friction.
    pub next_action: String,
    /// Cognitive complexity 0.0–1.0 (the only value shared via MCP).
    pub complexity: f32,
}

// ── Ollama API types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Internal JSON structure expected from the LLM response.
#[derive(Deserialize)]
struct LlmJson {
    intent: String,
    next_action: String,
    complexity: f32,
}

// ── LocalEngine ───────────────────────────────────────────────────────────────

/// Thin wrapper around AiConfig. Stateless — no model loaded in memory.
/// Each `analyze` call makes one HTTP request to the local Ollama daemon.
pub struct LocalEngine {
    config: AiConfig,
    client: reqwest::Client,
}

impl LocalEngine {
    pub fn new(config: AiConfig) -> Self {
        LocalEngine {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Analyze recent OCR text. Calls Ollama if available, otherwise heuristics.
    pub async fn analyze(&self, ocr_text: &str, primary_app: &str) -> LlmAnalysis {
        if self.config.is_enabled() {
            let prompt = build_prompt(ocr_text, primary_app);
            if let Ok(analysis) = self.call_ollama(&prompt).await {
                return analysis;
            }
            // Ollama offline or returned bad JSON — fall through to heuristics
        }
        heuristic_analysis(ocr_text, primary_app)
    }

    async fn call_ollama(&self, prompt: &str) -> Result<LlmAnalysis, String> {
        let url = format!("{}/api/generate", self.config.ollama_url);
        let body = OllamaRequest {
            model: &self.config.ollama_model,
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let ollama_resp: OllamaResponse =
            resp.json().await.map_err(|e| e.to_string())?;

        parse_response(&ollama_resp.response)
            .ok_or_else(|| "LLM returned unparseable JSON".to_string())
    }
}

// ── Prompt ────────────────────────────────────────────────────────────────────

fn build_prompt(ocr_text: &str, primary_app: &str) -> String {
    let ocr_truncated = if ocr_text.len() > 800 {
        &ocr_text[..800]
    } else {
        ocr_text
    };

    format!(
        r#"You are a concise focus assistant. Analyze this screen activity and respond with JSON only. No explanation, no markdown.

Primary app: {primary_app}
Recent screen text:
{ocr_truncated}

Respond with exactly this JSON (no other text):
{{"intent": "one sentence of what the user was working on", "next_action": "one specific next step", "complexity": 0.0}}

complexity: 0.0 = trivial, 1.0 = maximum. IDE + terminal + docs = high. Single browser tab = low."#
    )
}

// ── Response parser ───────────────────────────────────────────────────────────

fn parse_response(raw: &str) -> Option<LlmAnalysis> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    let resp: LlmJson = serde_json::from_str(&raw[start..=end]).ok()?;
    Some(LlmAnalysis {
        intent: resp.intent,
        next_action: resp.next_action,
        complexity: resp.complexity.clamp(0.0, 1.0),
    })
}

// ── Heuristic fallback ────────────────────────────────────────────────────────

pub(crate) fn heuristic_analysis(ocr_text: &str, primary_app: &str) -> LlmAnalysis {
    let text_lower = ocr_text.to_lowercase();

    let intent = if text_lower.contains("fn ")
        || text_lower.contains("impl ")
        || text_lower.contains("struct ")
    {
        format!("Writing Rust code in {}", primary_app)
    } else if text_lower.contains("function ")
        || text_lower.contains("const ")
        || text_lower.contains("interface ")
    {
        format!("Writing TypeScript in {}", primary_app)
    } else if text_lower.contains("select ")
        || text_lower.contains("from ")
        || text_lower.contains("where ")
    {
        format!("Working on a SQL query in {}", primary_app)
    } else if text_lower.contains("git ")
        || text_lower.contains("cargo ")
        || text_lower.contains("npm ")
    {
        format!("Running terminal commands in {}", primary_app)
    } else {
        format!("Working in {}", primary_app)
    };

    let next_action = format!("Return to {} and resume your last action", primary_app);

    let word_count = ocr_text.split_whitespace().count();
    let density_score = (word_count as f32 / 300.0).min(0.5);
    let code_bonus: f32 = if text_lower.contains("fn ")
        || text_lower.contains("impl ")
        || text_lower.contains("function ")
    {
        0.3
    } else {
        0.0
    };
    let complexity = (density_score + code_bonus).clamp(0.1, 1.0);

    LlmAnalysis { intent, next_action, complexity }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cognitive_load_bounds() {
        let a = heuristic_analysis("fn main() {}", "Code");
        assert!(a.complexity >= 0.0 && a.complexity <= 1.0);
    }

    #[test]
    fn ide_plus_terminal_is_high_load() {
        let ocr = "fn freeze_frame() impl ContextSnapshot cargo build --release";
        let a = heuristic_analysis(ocr, "Code");
        assert!(a.complexity >= 0.3, "got {}", a.complexity);
    }

    #[test]
    fn parse_valid_json_response() {
        let raw = r#"{"intent": "Writing Rust async code", "next_action": "Fix borrow error on line 42", "complexity": 0.75}"#;
        let a = parse_response(raw).unwrap();
        assert_eq!(a.intent, "Writing Rust async code");
        assert!((a.complexity - 0.75).abs() < 0.001);
    }

    #[test]
    fn parse_response_with_prose() {
        let raw = r#"Here is the JSON: {"intent": "Reviewing PR", "next_action": "Add test", "complexity": 0.4} Done."#;
        let a = parse_response(raw).unwrap();
        assert_eq!(a.intent, "Reviewing PR");
    }

    #[test]
    fn parse_response_rejects_garbage() {
        assert!(parse_response("not json").is_none());
    }

    #[test]
    fn ai_config_default_disabled() {
        assert!(!AiConfig::default().is_enabled());
    }
}
