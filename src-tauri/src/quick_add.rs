//! A system-wide shortcut that opens the quick entry window, even while the
//! main window is minimised or in the background. The shortcut is a setting
//! and can be changed at runtime.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{App, AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::commands::Db;
use crate::error::{AppError, Result};
use crate::store::settings;

/// Label of the quick entry window (see tauri.conf.json).
const WINDOW: &str = "quick-add";
pub const SETTING_KEY: &str = "quick_add_shortcut";

/// Written in the format of `Shortcut::from_str`, e.g. "Control+Shift+Space".
fn default_shortcut() -> &'static str {
    if cfg!(target_os = "macos") {
        "Super+Shift+Space"
    } else {
        "Control+Shift+Space"
    }
}

struct Current {
    text: String,
    shortcut: Option<Shortcut>,
    /// False when another program owns the shortcut.
    registered: bool,
}

pub struct QuickAddState(Mutex<Current>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutInfo {
    shortcut: String,
    default_shortcut: String,
    registered: bool,
}

impl Current {
    fn info(&self) -> ShortcutInfo {
        ShortcutInfo {
            shortcut: self.text.clone(),
            default_shortcut: default_shortcut().to_owned(),
            registered: self.registered,
        }
    }
}

fn parse(text: &str) -> Result<Shortcut> {
    text.parse()
        .map_err(|_| AppError::Validation("shortcut.invalid"))
}

/// Registers the stored shortcut, or the default if none or an invalid one is stored.
pub fn setup(app: &App, stored: Option<String>) -> tauri::Result<()> {
    let text = stored
        .filter(|text| parse(text).is_ok())
        .unwrap_or_else(|| default_shortcut().to_owned());
    let shortcut = parse(&text).ok();
    app.manage(QuickAddState(Mutex::new(Current {
        text,
        shortcut,
        registered: false,
    })));

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, pressed, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                let state = app.state::<QuickAddState>();
                let is_quick_add = state
                    .0
                    .lock()
                    .is_ok_and(|current| current.shortcut.as_ref() == Some(pressed));
                if is_quick_add {
                    show(app);
                }
            })
            .build(),
    )?;

    let state = app.state::<QuickAddState>();
    let mut current = state.0.lock().expect("quick add state");
    if let Some(shortcut) = current.shortcut {
        // Another program may own the shortcut; the app still works without it.
        match app.global_shortcut().register(shortcut) {
            Ok(()) => current.registered = true,
            Err(error) => eprintln!("quick add shortcut unavailable: {error}"),
        }
    }
    Ok(())
}

fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.center();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub async fn get_quick_add_shortcut(state: State<'_, QuickAddState>) -> Result<ShortcutInfo> {
    let current = state
        .0
        .lock()
        .map_err(|_| AppError::Internal("quick add lock poisoned"))?;
    Ok(current.info())
}

/// Switches to a new shortcut. If it cannot be registered, the old one stays
/// active and `shortcut.unavailable` is returned.
#[tauri::command]
pub async fn set_quick_add_shortcut(
    app: AppHandle,
    db: State<'_, Db>,
    state: State<'_, QuickAddState>,
    shortcut: String,
) -> Result<ShortcutInfo> {
    let new = parse(&shortcut)?;
    let mut current = state
        .0
        .lock()
        .map_err(|_| AppError::Internal("quick add lock poisoned"))?;
    let global = app.global_shortcut();

    if let Some(old) = current.shortcut.filter(|_| current.registered) {
        let _ = global.unregister(old);
    }
    if global.register(new).is_err() {
        if let Some(old) = current.shortcut {
            current.registered = global.register(old).is_ok();
        }
        return Err(AppError::Conflict("shortcut.unavailable"));
    }

    current.text = shortcut;
    current.shortcut = Some(new);
    current.registered = true;
    db.with(|conn| settings::set(conn, SETTING_KEY, &current.text))?;
    Ok(current.info())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_plugin_global_shortcut::{Code, Modifiers};

    #[test]
    fn parses_the_stored_format() {
        assert_eq!(
            parse("Control+Shift+Space").unwrap(),
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space)
        );
        assert_eq!(
            parse("Super+Alt+KeyK").unwrap(),
            Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyK)
        );
        assert_eq!(
            parse("Control+Digit1").unwrap(),
            Shortcut::new(Some(Modifiers::CONTROL), Code::Digit1)
        );
        assert!(parse(default_shortcut()).is_ok());
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(
            parse("Control+Nonsense"),
            Err(AppError::Validation("shortcut.invalid"))
        ));
    }
}
