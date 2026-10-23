//! Boards, cards and tags as earlier versions kept them in the database.
//! Read once to move them into the vault; the tables stay as a backup.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::error::Result;
use crate::model::Priority;
use crate::vault::format::{
    BoardFile, Card, CardMeta, ColumnEntry, SubtaskEntry, TagEntry, Timestamp,
};
use crate::vault::import::Import;
use crate::vault::new_id;

pub fn has_boards(conn: &Connection) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM boards WHERE deleted_at IS NULL)",
        [],
        |row| row.get(0),
    )?)
}

/// Everything not deleted, with fresh ids. Position keys are kept.
pub fn export(conn: &Connection) -> Result<Import> {
    let mut ids: HashMap<(&str, i64), String> = HashMap::new();
    let mut id = |table: &'static str, old: i64| -> String {
        ids.entry((table, old)).or_insert_with(new_id).clone()
    };

    let mut tags = Vec::new();
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY id")?;
    for row in stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))? {
        let (old, name, color): (i64, String, String) = row?;
        tags.push(TagEntry {
            id: id("tags", old),
            name,
            color,
            extra: Default::default(),
        });
    }

    let mut boards: Vec<(i64, BoardFile, Vec<Card>)> = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, name, position, created_at FROM boards WHERE deleted_at IS NULL ORDER BY id",
    )?;
    for row in stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })? {
        let (old, name, position, created_at): (i64, String, String, i64) = row?;
        let board = BoardFile {
            id: id("boards", old),
            name,
            position,
            created_at: Timestamp(created_at),
            columns: Vec::new(),
            extra: Default::default(),
        };
        boards.push((old, board, Vec::new()));
    }

    // Board index of each live column.
    let mut column_boards: HashMap<i64, usize> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT id, board_id, name, position, is_done FROM board_columns
         WHERE deleted_at IS NULL ORDER BY id",
    )?;
    for row in stmt.query_map([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })? {
        let (old, board_old, name, position, is_done): (i64, i64, String, String, bool) = row?;
        let Some(b) = boards.iter().position(|(id, ..)| *id == board_old) else {
            continue;
        };
        column_boards.insert(old, b);
        boards[b].1.columns.push(ColumnEntry {
            id: id("columns", old),
            name,
            position,
            is_done,
            extra: Default::default(),
        });
    }

    let mut subtasks: HashMap<i64, Vec<SubtaskEntry>> = HashMap::new();
    let mut stmt =
        conn.prepare("SELECT id, card_id, title, done FROM subtasks ORDER BY position, id")?;
    for row in stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })? {
        let (old, card_old, title, done): (i64, i64, String, bool) = row?;
        subtasks.entry(card_old).or_default().push(SubtaskEntry {
            id: id("subtasks", old),
            title,
            done,
            extra: Default::default(),
        });
    }

    let mut card_tags: HashMap<i64, Vec<String>> = HashMap::new();
    let mut stmt = conn.prepare("SELECT card_id, tag_id FROM card_tags")?;
    for row in stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))? {
        let (card_old, tag_old): (i64, i64) = row?;
        card_tags
            .entry(card_old)
            .or_default()
            .push(id("tags", tag_old));
    }

    let mut stmt = conn.prepare(
        "SELECT id, column_id, title, notes, priority, position, created_at, updated_at, archived_at
         FROM cards WHERE deleted_at IS NULL ORDER BY id",
    )?;
    for row in stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Priority>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, Option<i64>>(8)?,
        ))
    })? {
        let (old, column_old, title, notes, priority, position, created, updated, archived) = row?;
        let Some(&b) = column_boards.get(&column_old) else {
            continue;
        };
        boards[b].2.push(Card {
            meta: CardMeta {
                id: id("cards", old),
                column: id("columns", column_old),
                position,
                title,
                priority,
                tags: card_tags.remove(&old).unwrap_or_default(),
                subtasks: subtasks.remove(&old).unwrap_or_default(),
                created_at: Timestamp(created),
                updated_at: Timestamp(updated),
                archived_at: archived.map(Timestamp),
                extra: Default::default(),
            },
            notes,
        });
    }

    Ok(Import {
        tags,
        boards: boards
            .into_iter()
            .map(|(_, mut board, cards)| {
                board.normalize();
                (board, cards)
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::open_in_memory;

    #[test]
    fn exports_live_rows_with_new_ids() {
        let conn = open_in_memory();
        assert!(!has_boards(&conn).unwrap());
        conn.execute_batch(
            "INSERT INTO tags (id, name, color) VALUES (1, 'Steuern', 'orange');
             INSERT INTO boards (id, name, position, created_at) VALUES (1, 'Arbeit', 'V', 5);
             INSERT INTO boards (id, name, position, created_at, deleted_at)
                 VALUES (2, 'Weg', 'k', 5, 9);
             INSERT INTO board_columns (id, board_id, name, position, is_done, created_at)
                 VALUES (1, 1, 'Erledigt', 'k', 1, 5), (2, 1, 'Offen', 'V', 0, 5);
             INSERT INTO cards (id, column_id, title, notes, priority, position, created_at,
                                updated_at, archived_at)
                 VALUES (1, 2, 'Bericht', 'Notiz', 3, 'V', 6, 7, NULL),
                        (2, 1, 'Alt', '', 0, 'V', 6, 7, 8);
             INSERT INTO subtasks (id, card_id, title, done, position)
                 VALUES (1, 1, 'Zwei', 0, 'k'), (2, 1, 'Eins', 1, 'V');
             INSERT INTO card_tags (card_id, tag_id) VALUES (1, 1);",
        )
        .unwrap();
        assert!(has_boards(&conn).unwrap());

        let import = export(&conn).unwrap();
        assert_eq!(import.tags.len(), 1);
        assert_eq!(import.boards.len(), 1);
        let (board, cards) = &import.boards[0];
        assert_eq!(board.name, "Arbeit");
        let columns: Vec<(&str, bool)> = board
            .columns
            .iter()
            .map(|c| (c.name.as_str(), c.is_done))
            .collect();
        assert_eq!(columns, [("Offen", false), ("Erledigt", true)]);

        let report = &cards[0];
        assert_eq!(report.meta.column, board.columns[0].id);
        assert_eq!(report.meta.priority, Priority::High);
        assert_eq!(report.meta.tags, [import.tags[0].id.clone()]);
        let subtasks: Vec<&str> = report
            .meta
            .subtasks
            .iter()
            .map(|s| s.title.as_str())
            .collect();
        assert_eq!(subtasks, ["Eins", "Zwei"]);
        assert_eq!(report.notes, "Notiz");
        assert_eq!(cards[1].meta.archived_at, Some(Timestamp(8)));
    }
}
