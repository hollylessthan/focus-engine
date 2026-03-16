use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::screenpipe::{
    browser,
    client::{OcrFrame, ScreenpipeClient},
    parsers::{clean_ocr, filter_frames},
    vscode,
};
use crate::AppState;

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
    #[zeroize(skip)]
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
/// Queries Screenpipe's local API for recent OCR frames, applies privacy
/// filters, then computes a heuristic cognitive load score. When Screenpipe
/// is offline, returns a graceful snapshot indicating context is unavailable.
#[tauri::command]
pub async fn freeze_frame(state: tauri::State<'_, AppState>) -> Result<ContextSnapshot, String> {
    if state.incognito() {
        return Err("Incognito mode is active — capture disabled".to_string());
    }

    let privacy_config = state
        .privacy_config
        .lock()
        .map_err(|e| e.to_string())?
        .clone();

    let client = ScreenpipeClient::new();

    let frames: Vec<OcrFrame> = match client.recent_ocr_frames(20).await {
        Ok(raw) => filter_frames(raw, &privacy_config),
        Err(_) => vec![], // Screenpipe offline — proceed with empty frames
    };

    let ocr_text = frames.iter().map(|f| f.text.as_str()).collect::<Vec<_>>().join("\n");
    let cleaned_ocr = clean_ocr(&ocr_text);
    // Score only the 5 most recent frames (~1 min of activity) to avoid
    // inflating complexity from stale historical app switches.
    let recent = &frames[..frames.len().min(5)];
    let cognitive_load_score = compute_load_score(recent);
    let active_intent = infer_intent(&frames);
    let next_immediate_action = infer_next_action(&frames);

    Ok(ContextSnapshot {
        id: Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64,
        active_intent,
        open_windows: vec![], // populated in Milestone 3 via DB
        cursor_position: (0, 0),
        visual_context_ocr: cleaned_ocr,
        next_immediate_action,
        cognitive_load_score,
    })
}

/// Returns all saved snapshots from the local DB.
/// Stub: returns empty list until DB layer is implemented (Milestone 3).
#[tauri::command]
pub async fn list_snapshots(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ContextSnapshot>, String> {
    if state.incognito() {
        return Ok(vec![]);
    }
    Ok(vec![])
}

/// Heuristic cognitive load score — replaced by LLM inference in Milestone 4.
///
/// Scoring rules:
/// - Each unique app adds 0.15 to base score
/// - IDE presence (VS Code, Xcode, vim, IntelliJ) adds up to 0.3
/// - Terminal presence adds up to 0.2
/// - Clamped to [0.1, 1.0]
fn compute_load_score(frames: &[OcrFrame]) -> f32 {
    if frames.is_empty() {
        return 0.1;
    }

    let unique_apps: HashSet<&str> = frames.iter().map(|f| f.app_name.as_str()).collect();

    let ide_count = frames
        .iter()
        .filter(|f| {
            let app = f.app_name.to_lowercase();
            app.contains("code") || app.contains("xcode") || app.contains("vim")
                || app.contains("intellij") || app.contains("cursor")
        })
        .count();

    let terminal_count = frames
        .iter()
        .filter(|f| {
            let app = f.app_name.to_lowercase();
            app.contains("terminal") || app.contains("iterm") || app.contains("warp")
                || app.contains("ghostty")
        })
        .count();

    let base = unique_apps.len() as f32 * 0.15;
    let ide_bonus = (ide_count as f32 * 0.1).min(0.3);
    let term_bonus = (terminal_count as f32 * 0.05).min(0.2);

    (base + ide_bonus + term_bonus).clamp(0.1, 1.0)
}

/// Priority order for determining the "primary" work context.
/// Lower index = higher priority. If a frame's app matches, it wins.
const APP_PRIORITY: &[&str] = &[
    "Code",       // VS Code
    "Cursor",     // Cursor IDE
    "Xcode",      // Xcode
    "IntelliJ",   // JetBrains IDEs
    "Ghostty",    // Ghostty terminal
    "iTerm2",     // iTerm2
    "Terminal",   // macOS Terminal
    "Warp",       // Warp terminal
    "Google Chrome",
    "Arc",
    "Safari",
    "Firefox",
];

/// Select the most relevant OCR frame using app priority, then focused, then recency.
fn select_primary_frame(frames: &[OcrFrame]) -> Option<&OcrFrame> {
    for priority_app in APP_PRIORITY {
        if let Some(f) = frames.iter().find(|f| f.app_name.contains(priority_app)) {
            return Some(f);
        }
    }
    // Fall back to focused, then most recent
    frames.iter().find(|f| f.focused).or_else(|| frames.first())
}

/// Infer user intent from the highest-priority app in recent frames.
fn infer_intent(frames: &[OcrFrame]) -> String {
    match select_primary_frame(frames) {
        Some(f) => {
            if let Some(ctx) = vscode::extract_from_title(&f.window_name) {
                let dirty = if ctx.has_unsaved_changes { " (unsaved)" } else { "" };
                format!("Editing {}{} in {}", ctx.active_file, dirty, ctx.workspace)
            } else if f.app_name == "Code" {
                // VS Code title was OCR-truncated — extract workspace from what we have.
                // Truncated format: "filename — workspace" (Visual Studio Code cut off)
                let parts: Vec<&str> = f.window_name.splitn(2, " \u{2014} ").collect();
                if parts.len() == 2 {
                    format!("Editing {} in {}", parts[0].trim(), parts[1].trim())
                } else {
                    format!("Working in VS Code — {}", f.window_name)
                }
            } else if let Some(tab) = browser::extract_active_tab_title(&f.window_name) {
                format!("Browsing: {}", tab)
            } else {
                format!("Working in {}", f.app_name)
            }
        }
        None => "Context unavailable — Screenpipe offline".to_string(),
    }
}

/// Suggest the most actionable next micro-step.
fn infer_next_action(frames: &[OcrFrame]) -> String {
    match select_primary_frame(frames) {
        Some(f) => {
            if let Some(ctx) = vscode::extract_from_title(&f.window_name) {
                if ctx.has_unsaved_changes {
                    format!("Save and continue editing {}", ctx.active_file)
                } else {
                    format!("Resume editing {} in {}", ctx.active_file, ctx.workspace)
                }
            } else if f.app_name == "Code" {
                let parts: Vec<&str> = f.window_name.splitn(2, " \u{2014} ").collect();
                if parts.len() == 2 {
                    format!("Resume editing {} in {}", parts[0].trim(), parts[1].trim())
                } else {
                    "Return to VS Code and resume your last action".to_string()
                }
            } else {
                format!("Return to {} and resume your last action", f.app_name)
            }
        }
        None => "Review your last action and continue.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_score_empty_frames_returns_minimum() {
        assert_eq!(compute_load_score(&[]), 0.1);
    }

    #[test]
    fn load_score_ide_and_terminal_is_high() {
        let frames = vec![
            OcrFrame {
                text: String::new(),
                app_name: "Code".to_string(),
                window_name: "main.rs — focus-engine — Visual Studio Code".to_string(),
                focused: true,
            },
            OcrFrame {
                text: String::new(),
                app_name: "iTerm2".to_string(),
                window_name: "zsh".to_string(),
                focused: false,
            },
        ];
        let score = compute_load_score(&frames);
        assert!(score > 0.3, "IDE+terminal should score above 0.3, got {}", score);
    }

    #[test]
    fn infer_intent_vscode_focused() {
        let frames = vec![OcrFrame {
            text: String::new(),
            app_name: "Code".to_string(),
            window_name: "snapshot.rs — focus-engine — Visual Studio Code".to_string(),
            focused: true,
        }];
        let intent = infer_intent(&frames);
        assert!(intent.contains("snapshot.rs"));
        assert!(intent.contains("focus-engine"));
    }
}
