/// Screenpipe local API client.
///
/// Screenpipe exposes a local HTTP API on 127.0.0.1:3030.
/// All traffic is loopback-only; no data ever leaves the machine.
use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A single OCR frame returned by Screenpipe's /search endpoint.
/// `text` is PII-adjacent and zeroized on drop.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct OcrFrame {
    pub text: String,
    pub app_name: String,
    pub window_name: String,
    #[zeroize(skip)]
    pub focused: bool,
}

// --- Screenpipe response deserialization types ---

#[derive(Deserialize)]
struct RawOcrContent {
    text: String,
    app_name: String,
    window_name: String,
    #[serde(default)]
    focused: bool,
}

#[derive(Deserialize)]
struct RawSearchItem {
    content: RawOcrContent,
}

#[derive(Deserialize)]
struct RawSearchResponse {
    data: Vec<RawSearchItem>,
}

pub struct ScreenpipeClient {
    base_url: String,
}

impl ScreenpipeClient {
    pub fn new() -> Self {
        ScreenpipeClient {
            base_url: "http://127.0.0.1:3030".to_string(),
        }
    }

    /// Returns true if Screenpipe is running and responsive.
    pub async fn health_check(&self) -> bool {
        reqwest::Client::new()
            .get(format!("{}/health", self.base_url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Fetch the most recent `limit` OCR frames from Screenpipe.
    /// Returns structured frames with app/window metadata.
    pub async fn recent_ocr_frames(&self, limit: u32) -> Result<Vec<OcrFrame>, String> {
        let url = format!(
            "{}/search?content_type=ocr&limit={}&offset=0",
            self.base_url, limit
        );

        let resp = reqwest::Client::new()
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: RawSearchResponse = resp.json().await.map_err(|e| e.to_string())?;

        Ok(body
            .data
            .into_iter()
            .map(|item| OcrFrame {
                text: item.content.text,
                app_name: item.content.app_name,
                window_name: item.content.window_name,
                focused: item.content.focused,
            })
            .collect())
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for ScreenpipeClient {
    fn default() -> Self {
        Self::new()
    }
}
