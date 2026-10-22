mod commands;
mod error;
mod model;
mod position;
mod store;

use tauri::Manager;

/// Label of the small window for quick entry (see tauri.conf.json).
const QUICK_ADD_WINDOW: &str = "quick-add";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = store::open(&dir.join("hodie.db"))?;
            app.manage(commands::Db::new(conn));
            #[cfg(desktop)]
            quick_add::setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            commands::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// A system-wide shortcut that opens the quick entry window, even while the
/// main window is minimised or in the background.
#[cfg(desktop)]
mod quick_add {
    use tauri::{App, AppHandle, Manager};
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    use super::QUICK_ADD_WINDOW;

    /// Cmd+Shift+Space on macOS, Ctrl+Shift+Space elsewhere.
    fn shortcut() -> Shortcut {
        let modifier = if cfg!(target_os = "macos") {
            Modifiers::SUPER
        } else {
            Modifiers::CONTROL
        };
        Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::Space)
    }

    pub fn setup(app: &App) -> tauri::Result<()> {
        let quick_add = shortcut();
        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, pressed, event| {
                    if pressed == &quick_add && event.state() == ShortcutState::Pressed {
                        show(app);
                    }
                })
                .build(),
        )?;
        // Another program may own the shortcut; the app still works without it.
        if let Err(error) = app.global_shortcut().register(quick_add) {
            eprintln!("quick add shortcut unavailable: {error}");
        }
        Ok(())
    }

    fn show(app: &AppHandle) {
        if let Some(window) = app.get_webview_window(QUICK_ADD_WINDOW) {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
