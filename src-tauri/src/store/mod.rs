//! The app's own SQLite database: settings and colour presets. Boards live in
//! the vault (`crate::vault`); their old tables are only read once by
//! `legacy` to move them there. Knows nothing about Tauri or the UI.

pub mod legacy;
pub mod presets;
pub mod settings;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::error::{AppError, Result};

/// Applied in order; the database's `user_version` is the number applied.
/// Never edit a released migration, always append a new one.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/001_initial.sql"),
    include_str!("migrations/002_settings.sql"),
    include_str!("migrations/003_color_presets.sql"),
];

/// File name of the database in the app data directory.
pub const DB_FILE: &str = "hodie.db";

/// Bytes the database takes on disk, including SQLite's WAL side files.
pub fn disk_usage(dir: &Path) -> u64 {
    ["", "-wal", "-shm"]
        .iter()
        .filter_map(|suffix| std::fs::metadata(dir.join(format!("{DB_FILE}{suffix}"))).ok())
        .map(|meta| meta.len())
        .sum()
}

pub fn open(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    prepare(&mut conn)?;
    purge_deleted(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Connection {
    let mut conn = Connection::open_in_memory().expect("open in-memory database");
    prepare(&mut conn).expect("prepare in-memory database");
    conn
}

fn prepare(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate(conn)
}

fn migrate(conn: &mut Connection) -> Result<()> {
    let applied: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let applied = usize::try_from(applied).map_err(|_| AppError::DatabaseTooNew)?;
    if applied > MIGRATIONS.len() {
        return Err(AppError::DatabaseTooNew);
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)?;
        tx.commit()?;
    }
    Ok(())
}

/// Removes rows that were soft-deleted for undo in an earlier session.
fn purge_deleted(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM cards WHERE deleted_at IS NOT NULL;
         DELETE FROM board_columns WHERE deleted_at IS NOT NULL;
         DELETE FROM boards WHERE deleted_at IS NOT NULL;",
    )?;
    Ok(())
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as i64)
}

/// Trims a user-entered name or title and rejects empty ones.
pub(crate) fn require_text(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::Validation("validation.empty"))
    } else {
        Ok(trimmed.to_owned())
    }
}

/// Turns "no row changed" into `NotFound`.
fn expect_changed(changed: usize) -> Result<()> {
    if changed == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_usage_counts_database_files_only() {
        let dir = std::env::temp_dir().join(format!("hodie-usage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(disk_usage(&dir), 0);
        std::fs::write(dir.join(DB_FILE), [0u8; 100]).unwrap();
        std::fs::write(dir.join(format!("{DB_FILE}-wal")), [0u8; 20]).unwrap();
        std::fs::write(dir.join("other.txt"), [0u8; 5]).unwrap();
        assert_eq!(disk_usage(&dir), 120);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn migrations_set_user_version() {
        let conn = open_in_memory();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as i64);
    }

    #[test]
    fn migrating_twice_is_a_no_op() {
        let mut conn = open_in_memory();
        migrate(&mut conn).unwrap();
    }

    #[test]
    fn refuses_database_from_newer_version() {
        let mut conn = open_in_memory();
        conn.pragma_update(None, "user_version", MIGRATIONS.len() as i64 + 1)
            .unwrap();
        assert!(matches!(migrate(&mut conn), Err(AppError::DatabaseTooNew)));
    }

    #[test]
    fn open_creates_file_and_purges_deleted_rows() {
        let dir = std::env::temp_dir().join(format!("hodie-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("purge.db");
        let _ = std::fs::remove_file(&path);

        let conn = open(&path).unwrap();
        conn.execute(
            "INSERT INTO boards (name, position, created_at, deleted_at) VALUES ('B', 'V', 1, 2)",
            [],
        )
        .unwrap();
        drop(conn);

        let conn = open(&path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM boards", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn require_text_trims_and_rejects_empty() {
        assert_eq!(require_text("  a b ").unwrap(), "a b");
        assert!(matches!(
            require_text("   "),
            Err(AppError::Validation("validation.empty"))
        ));
    }
}
