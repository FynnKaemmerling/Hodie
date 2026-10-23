//! Opens the vault at start-up: its folder comes from the settings, or a new
//! one in the documents folder. On first start, boards from the database of
//! earlier versions are moved into it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, Result};
use crate::store::{legacy, settings};
use crate::vault::files::visible_entries;
use crate::vault::import::import;
use crate::vault::store::VaultStore;
use crate::vault::{META_DIR, VAULT_FILE, Vault};

const PATH_KEY: &str = "vault_path";
/// Set once the database's boards were moved into a vault.
const IMPORTED_KEY: &str = "legacy_imported";
const DEFAULT_DIR: &str = "Hodie";

/// The open vault, or the error code it failed to open with. Commands on a
/// vault that failed to open return that code.
pub struct Library(Mutex<std::result::Result<VaultStore, &'static str>>);

impl Library {
    pub fn open(app: &AppHandle, conn: &Connection) -> Library {
        let opened = open_store(app, conn).map_err(|error| {
            eprintln!("vault: {error}");
            error.code()
        });
        Library(Mutex::new(opened))
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut VaultStore) -> Result<T>) -> Result<T> {
        let mut opened = self
            .0
            .lock()
            .map_err(|_| AppError::Internal("vault lock poisoned"))?;
        match &mut *opened {
            Ok(store) => f(store),
            Err(code) => Err(AppError::Vault(code)),
        }
    }
}

fn open_store(app: &AppHandle, conn: &Connection) -> Result<VaultStore> {
    let root = match settings::get(conn, PATH_KEY)? {
        Some(path) => PathBuf::from(path),
        None => {
            let root = default_location(app)?;
            fs::create_dir_all(&root)?;
            let text = root
                .to_str()
                .ok_or(AppError::Internal("vault path is not valid UTF-8"))?;
            settings::set(conn, PATH_KEY, text)?;
            root
        }
    };
    let vault = Vault::open(&root)?;
    if settings::get(conn, IMPORTED_KEY)?.is_none() {
        if legacy::has_boards(conn)? {
            import(&vault, legacy::export(conn)?)?;
        }
        settings::set(conn, IMPORTED_KEY, "1")?;
    }
    VaultStore::load(vault)
}

/// `Hodie` in the documents folder, or `Hodie 2`, … if that name is taken by
/// something that is not a vault.
fn default_location(app: &AppHandle) -> Result<PathBuf> {
    let base = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|_| AppError::Internal("no documents directory"))?;
    let mut counter = 1;
    loop {
        let name = match counter {
            1 => DEFAULT_DIR.to_owned(),
            n => format!("{DEFAULT_DIR} {n}"),
        };
        let path = base.join(name);
        if can_hold_vault(&path) {
            return Ok(path);
        }
        counter += 1;
    }
}

fn can_hold_vault(path: &Path) -> bool {
    !path.exists()
        || path.join(META_DIR).join(VAULT_FILE).is_file()
        || visible_entries(path).is_ok_and(|entries| entries.is_empty())
}
