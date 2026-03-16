// Mirror of Rust structs — keep in sync with src-tauri/src/commands/snapshot.rs

export interface WindowState {
  title: string;
  app_name: string;
  z_order: number;
  bounds: { x: number; y: number; width: number; height: number };
}

export interface ContextSnapshot {
  id: string;
  timestamp: number;
  /** Anchoring intent: what was the user doing? (AI-inferred from OCR) */
  active_intent: string;
  /** Reconstructed window environment */
  open_windows: WindowState[];
  cursor_position: [number, number];
  /** Raw OCR text from Screenpipe — never leaves this machine */
  visual_context_ocr: string;
  /** Exact next micro-step to reduce return friction */
  next_immediate_action: string;
  /** 0.0–1.0 cognitive complexity score */
  cognitive_load_score: number;
}

export type WorkLifeMode = "work" | "personal";

export type ShieldStatus = "inactive" | "active" | "frozen";

export interface ScreenpipeStatus {
  connected: boolean;
  last_frame_at: number | null;
}

export interface VsCodeContext {
  active_file: string;
  workspace: string;
  has_unsaved_changes: boolean;
}

export interface BrowserTab {
  title: string;
  url: string;
  is_active: boolean;
}

export interface PrivacyConfig {
  excluded_apps: string[];
  excluded_window_title_patterns: string[];
  redact_urls_matching: string[];
  version: number;
}
