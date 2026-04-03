mod commands;
mod models;

use models::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_state = AppState::new();
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat commands
            commands::send_chat_message,
            commands::stream_chat_message,
            commands::get_conversations,
            commands::get_conversation,
            commands::create_conversation,
            commands::delete_conversation,
            commands::clear_conversation,
            // Task commands
            commands::get_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::generate_subtasks,
            // Model commands
            commands::get_model_providers,
            commands::get_models,
            commands::set_default_model,
            commands::test_model,
            commands::update_provider_config,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
            // System commands
            commands::get_health,
            commands::open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
