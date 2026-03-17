mod ai;
mod commands;
mod db;
mod mcp;
mod screenpipe;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};

use ai::{config::AiConfig, engine::LocalEngine};
use commands::mode::WorkLifeMode;
use commands::privacy::PrivacyConfig;
use db::store::Store;
use tauri::Manager;

/// Shared application state — managed by Tauri and injected into commands via State<AppState>.
pub struct AppState {
    /// True when incognito mode is active. Screenpipe polling is suspended.
    pub incognito_active: AtomicBool,
    /// Active privacy exclusion rules loaded from privacy_config.json.
    pub privacy_config: Mutex<PrivacyConfig>,
    /// Current Work/Life mode — controls which signals are interruptions.
    pub mode: Mutex<WorkLifeMode>,
    /// SQLite store — initialized in setup(), available to all commands thereafter.
    pub db: OnceLock<Store>,
    /// AI config loaded from ai_config.json — controls model path and parameters.
    pub ai_config: Mutex<AiConfig>,
    /// Local LLM engine — present when ollama_model is configured.
    /// Stateless (no model in memory) — each call hits Ollama's localhost API.
    pub ai_engine: OnceLock<LocalEngine>,
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
            db: OnceLock::new(),
            ai_config: Mutex::new(AiConfig::default()),
            ai_engine: OnceLock::new(),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("focus.db");
            let store = Store::open(db_path.to_str().unwrap_or("focus.db"))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let state = app.state::<AppState>();
            // Restore persisted mode so the toggle survives restarts.
            if let Ok(mode) = store.get_mode() {
                *state.mode.lock().unwrap() = mode;
            }
            state.db.set(store).ok();

            // Load AI config and optionally initialize the local LLM engine.
            let ai_config_path = data_dir.join("ai_config.json");
            // Write default config on first launch so users know what to edit.
            if !ai_config_path.exists() {
                let default_json = serde_json::to_string_pretty(&AiConfig::default())
                    .unwrap_or_default();
                let _ = std::fs::write(&ai_config_path, default_json);
            }
            let ai_config = AiConfig::load(&ai_config_path);
            *state.ai_config.lock().unwrap() = ai_config.clone();

            if ai_config.is_enabled() {
                state.ai_engine.set(LocalEngine::new(ai_config.clone())).ok();
                eprintln!("[focus-engine] Ollama LLM enabled: {}", ai_config.ollama_model);
            }

            Ok(())
        })
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
