/// VS Code context extractor.
///
/// Parses VS Code window titles to extract active file path and dirty state.
/// Window title format: `● filename.rs — workspace — Visual Studio Code`
///                  or: `filename.rs — workspace — Visual Studio Code`
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsCodeContext {
    /// The active file name (e.g. "main.rs")
    pub active_file: String,
    /// Workspace/folder name
    pub workspace: String,
    /// True if the file has unsaved changes (● prefix)
    pub has_unsaved_changes: bool,
}

/// Try to extract VS Code context from a window title.
/// Returns None if the title is not a VS Code window.
pub fn extract_from_title(title: &str) -> Option<VsCodeContext> {
    // Must end with "Visual Studio Code"
    if !title.contains("Visual Studio Code") {
        return None;
    }

    let has_unsaved = title.starts_with('●') || title.starts_with("\u{25CF}");
    let cleaned = title
        .trim_start_matches('●')
        .trim_start_matches('\u{25CF}')
        .trim();

    // Format: "filename — workspace — Visual Studio Code"
    let parts: Vec<&str> = cleaned.splitn(3, " \u{2014} ").collect(); // — = em dash
    if parts.len() >= 2 {
        Some(VsCodeContext {
            active_file: parts[0].trim().to_string(),
            workspace: parts[1].trim().to_string(),
            has_unsaved_changes: has_unsaved,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_title() {
        let title = "main.rs — focus-engine — Visual Studio Code";
        let ctx = extract_from_title(title).unwrap();
        assert_eq!(ctx.active_file, "main.rs");
        assert_eq!(ctx.workspace, "focus-engine");
        assert!(!ctx.has_unsaved_changes);
    }

    #[test]
    fn parses_dirty_title() {
        let title = "● snapshot.rs — focus-engine — Visual Studio Code";
        let ctx = extract_from_title(title).unwrap();
        assert_eq!(ctx.active_file, "snapshot.rs");
        assert!(ctx.has_unsaved_changes);
    }

    #[test]
    fn ignores_non_vscode_title() {
        assert!(extract_from_title("Google Chrome").is_none());
    }
}
