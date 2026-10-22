use rusqlite::{Connection, ErrorCode, params};

use super::{expect_changed, require_text};
use crate::error::{AppError, Result};
use crate::model::{Id, TAG_COLORS, Tag, TagPatch};

fn require_color(color: &str) -> Result<&str> {
    if TAG_COLORS.contains(&color) {
        Ok(color)
    } else {
        Err(AppError::Validation("tag.invalid_color"))
    }
}

/// Maps the unique-name violation to a code the frontend can show.
fn duplicate_name(error: rusqlite::Error) -> AppError {
    match error.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => AppError::Conflict("tag.duplicate_name"),
        _ => AppError::Database(error),
    }
}

pub fn list(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name COLLATE NOCASE")?;
    let tags = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(tags)
}

/// Tag names are unique regardless of case.
pub fn create(conn: &Connection, name: &str, color: &str) -> Result<Tag> {
    let name = require_text(name)?;
    let color = require_color(color)?;
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )
    .map_err(duplicate_name)?;
    Ok(Tag {
        id: conn.last_insert_rowid(),
        name,
        color: color.to_owned(),
    })
}

pub fn update(conn: &mut Connection, id: Id, patch: &TagPatch) -> Result<()> {
    let name = patch.name.as_deref().map(require_text).transpose()?;
    let color = patch.color.as_deref().map(require_color).transpose()?;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM tags WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound);
    }
    if let Some(name) = name {
        tx.execute("UPDATE tags SET name = ?2 WHERE id = ?1", params![id, name])
            .map_err(duplicate_name)?;
    }
    if let Some(color) = color {
        tx.execute(
            "UPDATE tags SET color = ?2 WHERE id = ?1",
            params![id, color],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Removes the tag from every card.
pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    expect_changed(conn.execute("DELETE FROM tags WHERE id = ?1", [id])?)
}

/// Tag ids of a card, sorted by tag name.
pub(crate) fn card_tag_ids(conn: &Connection, card_id: Id) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare(
        "SELECT t.id FROM card_tags ct JOIN tags t ON t.id = ct.tag_id
         WHERE ct.card_id = ?1 ORDER BY t.name COLLATE NOCASE",
    )?;
    let ids = stmt
        .query_map([card_id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

/// Replaces a card's tags. Run inside the caller's transaction.
pub(crate) fn replace_card_tags(conn: &Connection, card_id: Id, tag_ids: &[Id]) -> Result<Vec<Id>> {
    conn.execute("DELETE FROM card_tags WHERE card_id = ?1", [card_id])?;
    for tag_id in tag_ids {
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM tags WHERE id = ?1)",
            [tag_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(AppError::NotFound);
        }
        conn.execute(
            "INSERT OR IGNORE INTO card_tags (card_id, tag_id) VALUES (?1, ?2)",
            params![card_id, tag_id],
        )?;
    }
    card_tag_ids(conn, card_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NewCard;
    use crate::store::cards::tests::{add, fixture};
    use crate::store::{boards, cards, open_in_memory};

    #[test]
    fn create_list_update_delete() {
        let mut conn = open_in_memory();
        let work = create(&conn, " work ", "blue").unwrap();
        create(&conn, "Home", "green").unwrap();
        let names: Vec<_> = list(&conn).unwrap().into_iter().map(|t| t.name).collect();
        assert_eq!(names, ["Home", "work"]);

        update(
            &mut conn,
            work.id,
            &TagPatch {
                name: Some("Arbeit".into()),
                color: Some("red".into()),
            },
        )
        .unwrap();
        assert_eq!(
            list(&conn).unwrap()[0],
            Tag {
                id: work.id,
                name: "Arbeit".into(),
                color: "red".into()
            }
        );

        delete(&conn, work.id).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 1);
        assert!(matches!(delete(&conn, work.id), Err(AppError::NotFound)));
    }

    #[test]
    fn names_are_unique_ignoring_case() {
        let mut conn = open_in_memory();
        create(&conn, "Work", "blue").unwrap();
        let home = create(&conn, "Home", "blue").unwrap();
        assert!(matches!(
            create(&conn, "WORK", "red"),
            Err(AppError::Conflict("tag.duplicate_name"))
        ));
        assert!(matches!(
            update(
                &mut conn,
                home.id,
                &TagPatch {
                    name: Some("work".into()),
                    ..Default::default()
                }
            ),
            Err(AppError::Conflict("tag.duplicate_name"))
        ));
    }

    #[test]
    fn rejects_unknown_color() {
        let conn = open_in_memory();
        assert!(matches!(
            create(&conn, "Tag", "#ff0000"),
            Err(AppError::Validation("tag.invalid_color"))
        ));
    }

    #[test]
    fn cards_get_and_lose_tags() {
        let mut f = fixture();
        let urgent = create(&f.conn, "Urgent", "red").unwrap();
        let home = create(&f.conn, "Home", "green").unwrap();

        let card = cards::create(
            &mut f.conn,
            &NewCard {
                column_id: f.columns[0],
                title: "Card".into(),
                tag_ids: vec![urgent.id, home.id, urgent.id],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(card.tag_ids, [home.id, urgent.id]);

        let other = add(&mut f.conn, f.columns[0], "Other");
        assert_eq!(
            cards::set_tags(&mut f.conn, other, &[urgent.id]).unwrap(),
            [urgent.id]
        );
        assert!(matches!(
            cards::set_tags(&mut f.conn, other, &[999]),
            Err(AppError::NotFound)
        ));
        // The failed call must not have removed the existing tag.
        assert_eq!(cards::get(&f.conn, other).unwrap().tag_ids, [urgent.id]);

        delete(&f.conn, urgent.id).unwrap();
        let view = boards::view(&f.conn, f.board_id).unwrap();
        let tag_ids: Vec<_> = view.columns[0]
            .cards
            .iter()
            .map(|c| c.tag_ids.clone())
            .collect();
        assert_eq!(tag_ids, [vec![home.id], vec![]]);
    }
}
