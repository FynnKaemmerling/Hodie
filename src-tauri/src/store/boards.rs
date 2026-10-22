use rusqlite::{Connection, OptionalExtension, params};

use super::{cards, expect_changed, now_ms, position_keys, require_text};
use crate::error::{AppError, Result};
use crate::model::{Board, BoardView, Column, ColumnView, Id};
use crate::position;

const LIVE_KEYS: &str =
    "SELECT position FROM boards WHERE deleted_at IS NULL AND id <> ?1 ORDER BY position, id";

pub fn list(conn: &Connection) -> Result<Vec<Board>> {
    let mut stmt =
        conn.prepare("SELECT id, name FROM boards WHERE deleted_at IS NULL ORDER BY position, id")?;
    let boards = stmt
        .query_map([], |row| {
            Ok(Board {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(boards)
}

/// Creates a board with the given columns; the last column becomes the done
/// column. Column names come from the frontend because they are translated.
pub fn create(conn: &mut Connection, name: &str, column_names: &[String]) -> Result<Board> {
    let name = require_text(name)?;
    if column_names.is_empty() {
        return Err(AppError::Validation("board.no_columns"));
    }
    let column_names = column_names
        .iter()
        .map(|column| require_text(column))
        .collect::<Result<Vec<_>>>()?;

    let tx = conn.transaction()?;
    let key = position::key_after_last(&position_keys(&tx, LIVE_KEYS, [0])?)?;
    let now = now_ms();
    tx.execute(
        "INSERT INTO boards (name, position, created_at) VALUES (?1, ?2, ?3)",
        params![name, key, now],
    )?;
    let board_id = tx.last_insert_rowid();

    let last = column_names.len() - 1;
    let mut column_key: Option<String> = None;
    for (index, column) in column_names.iter().enumerate() {
        let key = position::key_between(column_key.as_deref(), None)?;
        tx.execute(
            "INSERT INTO board_columns (board_id, name, position, is_done, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![board_id, column, key, index == last, now],
        )?;
        column_key = Some(key);
    }
    tx.commit()?;

    Ok(Board { id: board_id, name })
}

pub fn rename(conn: &Connection, id: Id, name: &str) -> Result<()> {
    let name = require_text(name)?;
    expect_changed(conn.execute(
        "UPDATE boards SET name = ?2 WHERE id = ?1 AND deleted_at IS NULL",
        params![id, name],
    )?)
}

pub fn reorder(conn: &mut Connection, id: Id, index: usize) -> Result<()> {
    let tx = conn.transaction()?;
    let key = position::key_at(&position_keys(&tx, LIVE_KEYS, [id])?, index)?;
    expect_changed(tx.execute(
        "UPDATE boards SET position = ?2 WHERE id = ?1 AND deleted_at IS NULL",
        params![id, key],
    )?)?;
    tx.commit()?;
    Ok(())
}

/// Soft delete; the board can be restored until the app restarts.
pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    expect_changed(conn.execute(
        "UPDATE boards SET deleted_at = ?2 WHERE id = ?1 AND deleted_at IS NULL",
        params![id, now_ms()],
    )?)
}

pub fn restore(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    let wanted: String = tx
        .query_row(
            "SELECT position FROM boards WHERE id = ?1 AND deleted_at IS NOT NULL",
            [id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or(AppError::NotFound)?;
    let key = position::free_key(&position_keys(&tx, LIVE_KEYS, [id])?, &wanted)?;
    tx.execute(
        "UPDATE boards SET deleted_at = NULL, position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

/// The board with its columns and the active (not archived) cards.
pub fn view(conn: &Connection, id: Id) -> Result<BoardView> {
    let board = conn
        .query_row(
            "SELECT id, name FROM boards WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| {
                Ok(Board {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            },
        )
        .optional()?
        .ok_or(AppError::NotFound)?;

    let mut stmt = conn.prepare(
        "SELECT id, name, is_done FROM board_columns
         WHERE board_id = ?1 AND deleted_at IS NULL
         ORDER BY position, id",
    )?;
    let mut columns: Vec<ColumnView> = stmt
        .query_map([id], |row| {
            Ok(ColumnView {
                column: Column {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    is_done: row.get(2)?,
                },
                cards: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;

    for card in cards::active_summaries(conn, id)? {
        if let Some(column) = columns.iter_mut().find(|c| c.column.id == card.column_id) {
            column.cards.push(card);
        }
    }

    Ok(BoardView { board, columns })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::store::open_in_memory;

    pub(crate) fn default_columns() -> Vec<String> {
        vec!["Open".into(), "Doing".into(), "Done".into()]
    }

    fn names(conn: &Connection) -> Vec<String> {
        list(conn).unwrap().into_iter().map(|b| b.name).collect()
    }

    #[test]
    fn create_adds_columns_with_last_as_done() {
        let mut conn = open_in_memory();
        let board = create(&mut conn, " Work ", &default_columns()).unwrap();
        assert_eq!(board.name, "Work");

        let view = view(&conn, board.id).unwrap();
        let columns: Vec<_> = view
            .columns
            .iter()
            .map(|c| (c.column.name.as_str(), c.column.is_done))
            .collect();
        assert_eq!(columns, [("Open", false), ("Doing", false), ("Done", true)]);
    }

    #[test]
    fn create_validates_input() {
        let mut conn = open_in_memory();
        assert!(matches!(
            create(&mut conn, "", &default_columns()),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            create(&mut conn, "Board", &[]),
            Err(AppError::Validation("board.no_columns"))
        ));
        assert!(matches!(
            create(&mut conn, "Board", &["A".into(), " ".into()]),
            Err(AppError::Validation(_))
        ));
        assert!(list(&conn).unwrap().is_empty());
    }

    #[test]
    fn boards_keep_creation_order_and_can_be_reordered() {
        let mut conn = open_in_memory();
        let a = create(&mut conn, "A", &default_columns()).unwrap();
        create(&mut conn, "B", &default_columns()).unwrap();
        let c = create(&mut conn, "C", &default_columns()).unwrap();
        assert_eq!(names(&conn), ["A", "B", "C"]);

        reorder(&mut conn, c.id, 0).unwrap();
        assert_eq!(names(&conn), ["C", "A", "B"]);
        reorder(&mut conn, a.id, 99).unwrap();
        assert_eq!(names(&conn), ["C", "B", "A"]);
        reorder(&mut conn, a.id, 1).unwrap();
        assert_eq!(names(&conn), ["C", "A", "B"]);
    }

    #[test]
    fn rename_changes_name() {
        let mut conn = open_in_memory();
        let board = create(&mut conn, "A", &default_columns()).unwrap();
        rename(&conn, board.id, "Renamed").unwrap();
        assert_eq!(names(&conn), ["Renamed"]);
        assert!(matches!(rename(&conn, 999, "X"), Err(AppError::NotFound)));
    }

    #[test]
    fn delete_hides_board_and_restore_brings_it_back_in_place() {
        let mut conn = open_in_memory();
        create(&mut conn, "A", &default_columns()).unwrap();
        let b = create(&mut conn, "B", &default_columns()).unwrap();
        create(&mut conn, "C", &default_columns()).unwrap();

        delete(&conn, b.id).unwrap();
        assert_eq!(names(&conn), ["A", "C"]);
        assert!(matches!(view(&conn, b.id), Err(AppError::NotFound)));
        assert!(matches!(delete(&conn, b.id), Err(AppError::NotFound)));

        restore(&mut conn, b.id).unwrap();
        assert_eq!(names(&conn), ["A", "B", "C"]);
        assert!(matches!(restore(&mut conn, b.id), Err(AppError::NotFound)));
    }
}
