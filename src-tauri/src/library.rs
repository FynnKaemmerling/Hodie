//! Opens the vault at start-up: its folder comes from the settings, or a new
//! one in the documents folder. On first start, boards from the database of
//! earlier versions are moved into it. The user can switch to another folder
//! at runtime.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;
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
pub struct Library(Mutex<Opened>);

struct Opened {
    /// None only if not even a default folder could be determined.
    path: Option<PathBuf>,
    store: std::result::Result<VaultStore, &'static str>,
}

/// What the frontend shows about the vault: where it is and, if it could not
/// be opened, why.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    pub path: Option<String>,
    pub error: Option<&'static str>,
}

impl Library {
    pub fn open(app: &AppHandle, conn: &Connection) -> Library {
        Library(Mutex::new(open_configured(app, conn)))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Opened>> {
        self.0
            .lock()
            .map_err(|_| AppError::Internal("vault lock poisoned"))
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut VaultStore) -> Result<T>) -> Result<T> {
        match &mut self.lock()?.store {
            Ok(store) => f(store),
            Err(code) => Err(AppError::Vault(code)),
        }
    }

    pub fn info(&self) -> Result<VaultInfo> {
        Ok(self.lock()?.info())
    }

    /// Opens the vault in `root` (an empty folder becomes a new vault) and
    /// remembers it. On failure the current vault stays open.
    pub fn switch(&self, conn: &Connection, root: &Path) -> Result<VaultInfo> {
        let text = root
            .to_str()
            .ok_or(AppError::Internal("vault path is not valid UTF-8"))?;
        let store = VaultStore::load(Vault::open(root)?)?;
        settings::set(conn, PATH_KEY, text)?;
        let mut opened = self.lock()?;
        *opened = Opened {
            path: Some(root.to_owned()),
            store: Ok(store),
        };
        Ok(opened.info())
    }

    /// Tries the configured folder again, e.g. after a drive was connected.
    pub fn reopen(&self, app: &AppHandle, conn: &Connection) -> Result<VaultInfo> {
        let mut opened = self.lock()?;
        *opened = open_configured(app, conn);
        Ok(opened.info())
    }
}

impl Opened {
    fn info(&self) -> VaultInfo {
        VaultInfo {
            path: self
                .path
                .as_deref()
                .map(|path| path.to_string_lossy().into_owned()),
            error: self.store.as_ref().err().copied(),
        }
    }
}

fn open_configured(app: &AppHandle, conn: &Connection) -> Opened {
    let path = match configured_path(app, conn) {
        Ok(path) => path,
        Err(error) => return failed(None, error),
    };
    match open_store(conn, &path) {
        Ok(store) => Opened {
            path: Some(path),
            store: Ok(store),
        },
        Err(error) => failed(Some(path), error),
    }
}

fn failed(path: Option<PathBuf>, error: AppError) -> Opened {
    eprintln!("vault: {error}");
    Opened {
        path,
        store: Err(error.code()),
    }
}

/// The folder from the settings. Without one, a new default folder is
/// created and remembered.
fn configured_path(app: &AppHandle, conn: &Connection) -> Result<PathBuf> {
    if let Some(path) = settings::get(conn, PATH_KEY)? {
        return Ok(PathBuf::from(path));
    }
    let root = default_location(app)?;
    fs::create_dir_all(&root)?;
    let text = root
        .to_str()
        .ok_or(AppError::Internal("vault path is not valid UTF-8"))?;
    settings::set(conn, PATH_KEY, text)?;
    Ok(root)
}

fn open_store(conn: &Connection, root: &Path) -> Result<VaultStore> {
    let vault = Vault::open(root)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::open_in_memory;
    use crate::vault::tests::TestDir;

    fn failed_library() -> Library {
        Library(Mutex::new(Opened {
            path: Some(PathBuf::from("/nirgends")),
            store: Err("vault.not_found"),
        }))
    }

    #[test]
    fn a_failed_vault_reports_its_code() {
        let library = failed_library();
        let info = library.info().unwrap();
        assert_eq!(info.path.as_deref(), Some("/nirgends"));
        assert_eq!(info.error, Some("vault.not_found"));
        let result = library.with(|store| Ok(store.list_boards()));
        assert_eq!(result.unwrap_err().code(), "vault.not_found");
    }

    #[test]
    fn switching_opens_an_empty_folder_and_remembers_it() {
        let conn = open_in_memory();
        let dir = TestDir::new();
        let library = failed_library();

        let info = library.switch(&conn, dir.path()).unwrap();
        assert_eq!(info.error, None);
        assert!(dir.path().join(".hodie/vault.json").is_file());
        assert_eq!(
            settings::get(&conn, PATH_KEY).unwrap().as_deref(),
            dir.path().to_str()
        );
        library
            .with(|store| store.create_board("B", &["Offen".into()]))
            .unwrap();
        assert!(dir.path().join("B/board.json").is_file());
    }

    #[test]
    fn a_failed_switch_keeps_the_open_vault() {
        let conn = open_in_memory();
        let good = TestDir::new();
        let foreign = TestDir::new();
        fs::write(foreign.path().join("Steuern.pdf"), "").unwrap();
        let library = failed_library();
        library.switch(&conn, good.path()).unwrap();

        let error = library.switch(&conn, foreign.path()).unwrap_err();
        assert_eq!(error.code(), "vault.not_a_vault");
        let info = library.info().unwrap();
        assert_eq!(info.path.as_deref(), good.path().to_str());
        assert_eq!(
            settings::get(&conn, PATH_KEY).unwrap().as_deref(),
            good.path().to_str()
        );
    }
}
