use rusqlite::{Connection, OptionalExtension, params};

use super::{cards, position_keys, require_text};
use crate::error::{AppError, Result};
use crate::model::{Id, Subtask, SubtaskPatch};
use crate::position;

const KEYS: &str =
    "SELECT position FROM subtasks WHERE card_id = ?1 AND id <> ?2 ORDER BY position, id";

/// Card of a subtask whose card is live.
fn live_card_id(conn: &Connection, id: Id) -> Result<Id> {
    let card_id: Id = conn
        .query_row("SELECT card_id FROM subtasks WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()?
        .ok_or(AppError::NotFound)?;
    cards::require_live(conn, card_id)?;
    Ok(card_id)
}

pub fn list(conn: &Connection, card_id: Id) -> Result<Vec<Subtask>> {
    let mut stmt = conn
        .prepare("SELECT id, title, done FROM subtasks WHERE card_id = ?1 ORDER BY position, id")?;
    let subtasks = stmt
        .query_map([card_id], |row| {
            Ok(Subtask {
                id: row.get(0)?,
                title: row.get(1)?,
                done: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(subtasks)
}

/// Appends a new subtask at the end of the card's checklist.
pub fn create(conn: &mut Connection, card_id: Id, title: &str) -> Result<Subtask> {
    let title = require_text(title)?;
    let tx = conn.transaction()?;
    cards::require_live(&tx, card_id)?;
    let key = position::key_after_last(&position_keys(&tx, KEYS, [card_id, 0])?)?;
    tx.execute(
        "INSERT INTO subtasks (card_id, title, position) VALUES (?1, ?2, ?3)",
        params![card_id, title, key],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(Subtask {
        id,
        title,
        done: false,
    })
}

pub fn update(conn: &Connection, id: Id, patch: &SubtaskPatch) -> Result<()> {
    let title = patch.title.as_deref().map(require_text).transpose()?;
    live_card_id(conn, id)?;
    if let Some(title) = title {
        conn.execute(
            "UPDATE subtasks SET title = ?2 WHERE id = ?1",
            params![id, title],
        )?;
    }
    if let Some(done) = patch.done {
        conn.execute(
            "UPDATE subtasks SET done = ?2 WHERE id = ?1",
            params![id, done],
        )?;
    }
    Ok(())
}

pub fn reorder(conn: &mut Connection, id: Id, index: usize) -> Result<()> {
    let tx = conn.transaction()?;
    let card_id = live_card_id(&tx, id)?;
    let key = position::key_at(&position_keys(&tx, KEYS, [card_id, id])?, index)?;
    tx.execute(
        "UPDATE subtasks SET position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

/// Hard delete: a subtask is small enough to be re-created for undo.
pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    live_card_id(conn, id)?;
    conn.execute("DELETE FROM subtasks WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::cards::tests::{add, fixture};

    fn titles(conn: &Connection, card_id: Id) -> Vec<String> {
        list(conn, card_id)
            .unwrap()
            .into_iter()
            .map(|s| s.title)
            .collect()
    }

    #[test]
    fn create_update_reorder_delete() {
        let mut f = fixture();
        let card = add(&mut f.conn, f.columns[0], "Card");
        let a = create(&mut f.conn, card, "A").unwrap();
        create(&mut f.conn, card, "B").unwrap();
        let c = create(&mut f.conn, card, " C ").unwrap();
        assert_eq!(titles(&f.conn, card), ["A", "B", "C"]);

        reorder(&mut f.conn, c.id, 0).unwrap();
        assert_eq!(titles(&f.conn, card), ["C", "A", "B"]);

        update(
            &f.conn,
            a.id,
            &SubtaskPatch {
                title: Some("A2".into()),
                done: Some(true),
            },
        )
        .unwrap();
        let updated = list(&f.conn, card).unwrap();
        assert_eq!(updated[1].title, "A2");
        assert!(updated[1].done);

        delete(&f.conn, a.id).unwrap();
        assert_eq!(titles(&f.conn, card), ["C", "B"]);
        assert!(matches!(delete(&f.conn, a.id), Err(AppError::NotFound)));
    }

    #[test]
    fn requires_live_card() {
        let mut f = fixture();
        let card = add(&mut f.conn, f.columns[0], "Card");
        let subtask = create(&mut f.conn, card, "A").unwrap();
        cards::delete(&f.conn, card).unwrap();
        assert!(matches!(
            create(&mut f.conn, card, "B"),
            Err(AppError::NotFound)
        ));
        assert!(matches!(
            update(&f.conn, subtask.id, &SubtaskPatch::default()),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn rejects_empty_title() {
        let mut f = fixture();
        let card = add(&mut f.conn, f.columns[0], "Card");
        assert!(matches!(
            create(&mut f.conn, card, "  "),
            Err(AppError::Validation(_))
        ));
    }
}
