use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Represents the OS-level state of a single window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub title: String,
    pub app_name: String,
    pub z_order: u32,
    pub bounds: WindowBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// The cognitive save-state captured by "Freeze Frame."
///
/// Designed per the neuro-cognitive spec:
/// - `active_intent`: anchors the user to what they were doing (Zeigarnik closure)
/// - `next_immediate_action`: breadcrumb for return-friction reduction
/// - `cognitive_load_score`: shared with MCP responders (never the raw OCR)
///
/// OCR fields are `zeroize`-on-drop to prevent memory-scraping.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ContextSnapshot {
    #[zeroize(skip)] // UUIDs are not sensitive
    pub id: String,
    #[zeroize(skip)]
    pub timestamp: i64,
    /// AI-inferred intent from Screenpipe OCR history (last 5 min).
    pub active_intent: String,
    #[zeroize(skip)]
    pub open_windows: Vec<WindowState>,
    #[zeroize(skip)]
    pub cursor_position: (i32, i32),
    /// Raw OCR text — sensitive PII, zeroized on drop.
    pub visual_context_ocr: String,
    /// User's next micro-step — zeroized on drop.
    pub next_immediate_action: String,
    /// 0.0–1.0. The only value shared externally via MCP responses.
    #[zeroize(skip)]
    pub cognitive_load_score: f32,
}

/// Freeze Frame: captures the current cognitive context as a snapshot.
///
/// In this stub, returns a placeholder snapshot. The full implementation
/// will query Screenpipe's local API and run inference via the local LLM.
#[tauri::command]
pub fn freeze_frame() -> Result<ContextSnapshot, String> {
    let snapshot = ContextSnapshot {
        id: Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64,
        active_intent: "Implementing the Focus Engine Tauri bootstrap.".to_string(),
        open_windows: vec![
            WindowState {
                title: "focus-engine — VSCode".to_string(),
                app_name: "Code".to_string(),
                z_order: 0,
                bounds: WindowBounds { x: 0, y: 0, width: 1440, height: 900 },
            },
            WindowState {
                title: "zsh — Terminal".to_string(),
                app_name: "Terminal".to_string(),
                z_order: 1,
                bounds: WindowBounds { x: 0, y: 0, width: 800, height: 400 },
            },
        ],
        cursor_position: (720, 450),
        visual_context_ocr: "[Screenpipe OCR stub — not yet connected]".to_string(),
        next_immediate_action: "Run `cargo tauri dev` and verify the window opens.".to_string(),
        cognitive_load_score: 0.72, // IDE + Terminal = high complexity
    };

    Ok(snapshot)
}

/// Returns all saved snapshots from the local DB.
///
/// Stub: returns empty list until DB layer is implemented.
#[tauri::command]
pub fn list_snapshots() -> Result<Vec<ContextSnapshot>, String> {
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freeze_frame_returns_valid_snapshot() {
        let result = freeze_frame();
        assert!(result.is_ok());
        let snap = result.unwrap();
        assert!(!snap.id.is_empty());
        assert!(snap.cognitive_load_score >= 0.0 && snap.cognitive_load_score <= 1.0);
        assert!(!snap.active_intent.is_empty());
    }
}
