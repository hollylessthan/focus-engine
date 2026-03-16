mod ai;
mod commands;
mod db;
mod mcp;
mod screenpipe;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
