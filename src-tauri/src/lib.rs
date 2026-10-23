mod commands;
mod error;
mod library;
mod model;
#[cfg(target_os = "macos")]
mod panel;
mod position;
mod quick_add;
mod store;
mod vault;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = store::open(&dir.join(store::DB_FILE))?;
            let shortcut = store::settings::get(&conn, quick_add::SETTING_KEY)?;
            app.manage(library::Library::open(app.handle(), &conn));
            app.manage(commands::Db::new(conn));
            quick_add::setup(app, shortcut)?;
            Ok(())
        })
        // The hidden quick entry window would otherwise keep the app running
        // invisibly after the main window is closed.
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_vault,
            commands::open_vault,
            commands::reopen_vault,
            commands::list_boards,
            commands::get_board,
            commands::create_board,
            commands::rename_board,
            commands::reorder_board,
            commands::delete_board,
            commands::restore_board,
            commands::create_column,
            commands::rename_column,
            commands::reorder_column,
            commands::set_done_column,
            commands::delete_column,
            commands::restore_column,
            commands::search_cards,
            commands::get_card,
            commands::create_card,
            commands::update_card,
            commands::move_card,
            commands::delete_card,
            commands::restore_card,
            commands::archive_card,
            commands::unarchive_card,
            commands::archive_done_cards,
            commands::list_archived_cards,
            commands::set_card_tags,
            commands::create_subtask,
            commands::update_subtask,
            commands::reorder_subtask,
            commands::delete_subtask,
            commands::list_tags,
            commands::create_tag,
            commands::update_tag,
            commands::count_tag_cards,
            commands::delete_tag,
            commands::list_color_presets,
            commands::create_color_preset,
            commands::update_color_preset,
            commands::delete_color_preset,
            commands::get_color_preset_choice,
            commands::set_color_preset_choice,
            commands::get_storage_usage,
            quick_add::get_quick_add_shortcut,
            quick_add::set_quick_add_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
