//! Watches the vault folder for changes from outside (sync clients, other
//! people, editors) and tells the windows when the boards changed.

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

use crate::library::Library;

/// Sent to all windows when files changed from outside (see src/api/events.ts).
pub const FILES_CHANGED: &str = "vault-files-changed";

/// Sync clients write many files in a burst; reload once it has calmed down.
const QUIET: Duration = Duration::from_millis(300);

/// Stops watching when dropped.
pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
}

/// None if the system cannot watch the folder (e.g. some network drives);
/// the windows still check when they get focus, see `refresh`.
pub fn start(app: &AppHandle, root: &Path) -> Option<VaultWatcher> {
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if event.is_ok() {
            let _ = sender.send(());
        }
    })
    .map_err(|error| eprintln!("vault: cannot watch: {error}"))
    .ok()?;
    watcher
        .watch(root, RecursiveMode::Recursive)
        .map_err(|error| eprintln!("vault: cannot watch {}: {error}", root.display()))
        .ok()?;

    let app = app.clone();
    // Ends when the watcher, and with it the sender, is dropped.
    thread::spawn(move || {
        while receiver.recv().is_ok() {
            while receiver.recv_timeout(QUIET).is_ok() {}
            refresh(&app);
        }
    });
    Some(VaultWatcher { _watcher: watcher })
}

/// Reloads the vault and tells the windows if anything changed.
pub fn refresh(app: &AppHandle) -> bool {
    // The watcher starts during set-up, just before the library is managed.
    let Some(library) = app.try_state::<Library>() else {
        return false;
    };
    match library.refresh() {
        Ok(true) => {
            let _ = app.emit(FILES_CHANGED, ());
            true
        }
        Ok(false) => false,
        Err(error) => {
            eprintln!("vault: cannot reload: {error}");
            false
        }
    }
}
