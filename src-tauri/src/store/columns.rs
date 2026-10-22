use rusqlite::{Connection, OptionalExtension, params};

use super::{expect_changed, now_ms, position_keys, require_text};
use crate::error::{AppError, Result};
use crate::model::{Column, Id};
use crate::position;

const LIVE_KEYS: &str = "SELECT position FROM board_columns
     WHERE board_id = ?1 AND deleted_at IS NULL AND id <> ?2
     ORDER BY position, id";

/// Board of a column that is not deleted and whose board is not deleted.
pub(crate) fn live_board_id(conn: &Connection, column_id: Id) -> Result<Id> {
    conn.query_row(
        "SELECT c.board_id FROM board_columns c JOIN boards b ON b.id = c.board_id
         WHERE c.id = ?1 AND c.deleted_at IS NULL AND b.deleted_at IS NULL",
        [column_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

/// Appends a new column at the end of the board.
pub fn create(conn: &mut Connection, board_id: Id, name: &str) -> Result<Column> {
    let name = require_text(name)?;
    let tx = conn.transaction()?;
    let board_live: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM boards WHERE id = ?1 AND deleted_at IS NULL)",
        [board_id],
        |row| row.get(0),
    )?;
    if !board_live {
        return Err(AppError::NotFound);
    }
    let key = position::key_after_last(&position_keys(&tx, LIVE_KEYS, [board_id, 0])?)?;
    tx.execute(
        "INSERT INTO board_columns (board_id, name, position, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![board_id, name, key, now_ms()],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(Column {
        id,
        name,
        is_done: false,
    })
}

pub fn rename(conn: &Connection, id: Id, name: &str) -> Result<()> {
    let name = require_text(name)?;
    live_board_id(conn, id)?;
    expect_changed(conn.execute(
        "UPDATE board_columns SET name = ?2 WHERE id = ?1",
        params![id, name],
    )?)
}

pub fn reorder(conn: &mut Connection, id: Id, index: usize) -> Result<()> {
    let tx = conn.transaction()?;
    let board_id = live_board_id(&tx, id)?;
    let key = position::key_at(&position_keys(&tx, LIVE_KEYS, [board_id, id])?, index)?;
    tx.execute(
        "UPDATE board_columns SET position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

/// Makes this column the board's done column.
pub fn set_done(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    let board_id = live_board_id(&tx, id)?;
    tx.execute(
        "UPDATE board_columns SET is_done = 0 WHERE board_id = ?1 AND is_done = 1",
        [board_id],
    )?;
    tx.execute("UPDATE board_columns SET is_done = 1 WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

/// Soft delete together with its cards. The done column cannot be deleted;
/// another column has to become the done column first.
pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    live_board_id(conn, id)?;
    let is_done: bool = conn.query_row(
        "SELECT is_done FROM board_columns WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    if is_done {
        return Err(AppError::Conflict("column.is_done"));
    }
    expect_changed(conn.execute(
        "UPDATE board_columns SET deleted_at = ?2 WHERE id = ?1",
        params![id, now_ms()],
    )?)
}

pub fn restore(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    let (board_id, wanted): (Id, String) = tx
        .query_row(
            "SELECT c.board_id, c.position FROM board_columns c JOIN boards b ON b.id = c.board_id
             WHERE c.id = ?1 AND c.deleted_at IS NOT NULL AND b.deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or(AppError::NotFound)?;
    let key = position::free_key(&position_keys(&tx, LIVE_KEYS, [board_id, id])?, &wanted)?;
    tx.execute(
        "UPDATE board_columns SET deleted_at = NULL, position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::boards::{self, tests::default_columns};
    use crate::store::open_in_memory;

    fn columns(conn: &Connection, board_id: Id) -> Vec<(String, bool)> {
        boards::view(conn, board_id)
            .unwrap()
            .columns
            .into_iter()
            .map(|c| (c.column.name, c.column.is_done))
            .collect()
    }

    fn setup() -> (Connection, Id, Vec<Id>) {
        let mut conn = open_in_memory();
        let board = boards::create(&mut conn, "Board", &default_columns()).unwrap();
        let ids = boards::view(&conn, board.id)
            .unwrap()
            .columns
            .iter()
            .map(|c| c.column.id)
            .collect();
        (conn, board.id, ids)
    }

    #[test]
    fn create_appends_and_reorder_moves() {
        let (mut conn, board_id, ids) = setup();
        let review = create(&mut conn, board_id, "Review").unwrap();
        assert!(!review.is_done);

        reorder(&mut conn, review.id, 2).unwrap();
        let names: Vec<_> = columns(&conn, board_id).into_iter().map(|c| c.0).collect();
        assert_eq!(names, ["Open", "Doing", "Review", "Done"]);

        reorder(&mut conn, ids[0], 99).unwrap();
        let names: Vec<_> = columns(&conn, board_id).into_iter().map(|c| c.0).collect();
        assert_eq!(names, ["Doing", "Review", "Done", "Open"]);
    }

    #[test]
    fn create_requires_live_board() {
        let (mut conn, board_id, _) = setup();
        boards::delete(&conn, board_id).unwrap();
        assert!(matches!(
            create(&mut conn, board_id, "X"),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn set_done_moves_the_done_flag() {
        let (mut conn, board_id, ids) = setup();
        set_done(&mut conn, ids[1]).unwrap();
        assert_eq!(
            columns(&conn, board_id),
            [
                ("Open".into(), false),
                ("Doing".into(), true),
                ("Done".into(), false)
            ]
        );
    }

    #[test]
    fn done_column_cannot_be_deleted() {
        let (conn, _, ids) = setup();
        assert!(matches!(
            delete(&conn, ids[2]),
            Err(AppError::Conflict("column.is_done"))
        ));
    }

    #[test]
    fn delete_and_restore() {
        let (mut conn, board_id, ids) = setup();
        rename(&conn, ids[1], "In progress").unwrap();
        delete(&conn, ids[1]).unwrap();
        assert_eq!(columns(&conn, board_id).len(), 2);
        assert!(matches!(
            rename(&conn, ids[1], "X"),
            Err(AppError::NotFound)
        ));

        restore(&mut conn, ids[1]).unwrap();
        let names: Vec<_> = columns(&conn, board_id).into_iter().map(|c| c.0).collect();
        assert_eq!(names, ["Open", "In progress", "Done"]);
    }
}
