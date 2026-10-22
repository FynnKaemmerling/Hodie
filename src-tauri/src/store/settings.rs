use rusqlite::{Connection, OptionalExtension, params};

use crate::error::Result;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::open_in_memory;

    #[test]
    fn set_get_and_overwrite() {
        let conn = open_in_memory();
        assert_eq!(get(&conn, "a").unwrap(), None);
        set(&conn, "a", "1").unwrap();
        set(&conn, "a", "2").unwrap();
        assert_eq!(get(&conn, "a").unwrap().as_deref(), Some("2"));
    }
}
