mod ai;
mod commands;
mod db;
mod mcp;
mod screenpipe;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use commands::mode::WorkLifeMode;
use commands::privacy::PrivacyConfig;

/// Shared application state — managed by Tauri and injected into commands via State<AppState>.
pub struct AppState {
    /// True when incognito mode is active. Screenpipe polling is suspended.
    pub incognito_active: AtomicBool,
    /// Active privacy exclusion rules loaded from privacy_config.json.
    pub privacy_config: Mutex<PrivacyConfig>,
    /// Current Work/Life mode — controls which signals are interruptions.
    pub mode: Mutex<WorkLifeMode>,
}

impl AppState {
    pub fn incognito(&self) -> bool {
        self.incognito_active.load(Ordering::Relaxed)
    }

    pub fn set_incognito(&self, active: bool) {
        self.incognito_active.store(active, Ordering::Relaxed);
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            incognito_active: AtomicBool::new(false),
            privacy_config: Mutex::new(PrivacyConfig::default()),
            mode: Mutex::new(WorkLifeMode::Work),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::snapshot::freeze_frame,
            commands::snapshot::list_snapshots,
            commands::os_system::toggle_do_not_disturb,
            commands::mode::get_mode,
            commands::mode::set_mode,
            commands::privacy::toggle_incognito,
            commands::privacy::get_incognito_status,
            commands::privacy::get_privacy_config,
            commands::privacy::update_privacy_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
