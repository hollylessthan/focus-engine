/// Screenpipe local API client — stub.
///
/// Screenpipe exposes a local HTTP API on 127.0.0.1:3030.
/// This client fetches recent OCR frames and window state logs.
/// All traffic is loopback-only; no data leaves the machine.
pub struct ScreenpipeClient {
    base_url: String,
}

impl ScreenpipeClient {
    pub fn new() -> Self {
        ScreenpipeClient {
            base_url: "http://127.0.0.1:3030".to_string(),
        }
    }

    /// Fetch recent OCR text from the last `seconds` seconds.
    pub fn recent_ocr(&self, _seconds: u32) -> Result<String, String> {
        // TODO: GET /frames?limit=20&offset=0 and concatenate text fields
        Ok("[Screenpipe OCR stub — not yet connected]".to_string())
    }

    /// Check if Screenpipe is running and responsive.
    pub fn health_check(&self) -> bool {
        // TODO: GET /health
        false
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
