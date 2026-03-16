/**
 * Typed wrappers around Tauri IPC commands.
 * Components must NOT call invoke() directly — use these functions.
 */
import { invoke } from "@tauri-apps/api/core";
import type { ContextSnapshot, WorkLifeMode, PrivacyConfig } from "./types";

/** Trigger a Freeze Frame: captures current cognitive context and saves a snapshot. */
export async function freezeFrame(): Promise<ContextSnapshot> {
  return invoke<ContextSnapshot>("freeze_frame");
}

/** Toggle macOS/Windows/Linux Do Not Disturb mode. */
export async function toggleDoNotDisturb(enable: boolean): Promise<string> {
  return invoke<string>("toggle_do_not_disturb", { enable });
}

/** Retrieve all saved snapshots from the local DB. */
export async function listSnapshots(): Promise<ContextSnapshot[]> {
  return invoke<ContextSnapshot[]>("list_snapshots");
}

/** Get the current Work/Life mode. */
export async function getMode(): Promise<WorkLifeMode> {
  return invoke<WorkLifeMode>("get_mode");
}

/** Set the Work/Life mode. */
export async function setMode(mode: WorkLifeMode): Promise<void> {
  return invoke<void>("set_mode", { mode });
}

/** Toggle Incognito Mode — kills all data capture and turns tray icon red. */
export async function toggleIncognito(): Promise<boolean> {
  return invoke<boolean>("toggle_incognito");
}

/** Get current incognito status. */
export async function getIncognitoStatus(): Promise<boolean> {
  return invoke<boolean>("get_incognito_status");
}

/** Get the active privacy configuration. */
export async function getPrivacyConfig(): Promise<PrivacyConfig> {
  return invoke<PrivacyConfig>("get_privacy_config");
}
