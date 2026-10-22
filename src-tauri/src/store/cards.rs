use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};

use super::{columns, expect_changed, now_ms, position_keys, require_text, subtasks, tags};
use crate::error::{AppError, Result};
use crate::model::{CardDetail, CardPatch, CardSummary, Id, NewCard};
use crate::position;

/// Active cards are neither deleted nor archived; only they have a place in
/// a column's order.
const ACTIVE_KEYS: &str = "SELECT position FROM cards
     WHERE column_id = ?1 AND deleted_at IS NULL AND archived_at IS NULL AND id <> ?2
     ORDER BY position, id";

struct CardState {
    column_id: Id,
    archived: bool,
}

/// A card that is not deleted and sits in a live column of a live board.
fn live_card(conn: &Connection, id: Id) -> Result<CardState> {
    conn.query_row(
        "SELECT c.column_id, c.archived_at IS NOT NULL
         FROM cards c
         JOIN board_columns col ON col.id = c.column_id
         JOIN boards b ON b.id = col.board_id
         WHERE c.id = ?1 AND c.deleted_at IS NULL
           AND col.deleted_at IS NULL AND b.deleted_at IS NULL",
        [id],
        |row| {
            Ok(CardState {
                column_id: row.get(0)?,
                archived: row.get(1)?,
            })
        },
    )
    .optional()?
    .ok_or(AppError::NotFound)
}

pub(crate) fn require_live(conn: &Connection, id: Id) -> Result<()> {
    live_card(conn, id).map(|_| ())
}

fn touch(conn: &Connection, id: Id) -> Result<()> {
    conn.execute(
        "UPDATE cards SET updated_at = ?2 WHERE id = ?1",
        params![id, now_ms()],
    )?;
    Ok(())
}

/// Summaries of cards on a board in live columns, filtered by `condition`.
fn summaries(
    conn: &Connection,
    board_id: Id,
    condition: &str,
    order: &str,
) -> Result<Vec<CardSummary>> {
    let sql = format!(
        "SELECT c.id, c.column_id, c.title, c.priority, c.notes <> '',
                (SELECT COUNT(*) FROM subtasks s WHERE s.card_id = c.id AND s.done = 1),
                (SELECT COUNT(*) FROM subtasks s WHERE s.card_id = c.id)
         FROM cards c
         JOIN board_columns col ON col.id = c.column_id
         WHERE col.board_id = ?1 AND col.deleted_at IS NULL AND c.deleted_at IS NULL
           AND {condition}
         ORDER BY {order}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut cards: Vec<CardSummary> = stmt
        .query_map([board_id], |row| {
            Ok(CardSummary {
                id: row.get(0)?,
                column_id: row.get(1)?,
                title: row.get(2)?,
                priority: row.get(3)?,
                has_notes: row.get(4)?,
                subtasks_done: row.get(5)?,
                subtasks_total: row.get(6)?,
                tag_ids: Vec::new(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;

    let mut tags_by_card = board_tag_ids(conn, board_id)?;
    for card in &mut cards {
        card.tag_ids = tags_by_card.remove(&card.id).unwrap_or_default();
    }
    Ok(cards)
}

fn board_tag_ids(conn: &Connection, board_id: Id) -> Result<HashMap<Id, Vec<Id>>> {
    let mut stmt = conn.prepare(
        "SELECT ct.card_id, ct.tag_id
         FROM card_tags ct
         JOIN tags t ON t.id = ct.tag_id
         JOIN cards c ON c.id = ct.card_id
         JOIN board_columns col ON col.id = c.column_id
         WHERE col.board_id = ?1
         ORDER BY t.name COLLATE NOCASE",
    )?;
    let mut map: HashMap<Id, Vec<Id>> = HashMap::new();
    for row in stmt.query_map([board_id], |row| Ok((row.get(0)?, row.get(1)?)))? {
        let (card_id, tag_id) = row?;
        map.entry(card_id).or_default().push(tag_id);
    }
    Ok(map)
}

pub(crate) fn active_summaries(conn: &Connection, board_id: Id) -> Result<Vec<CardSummary>> {
    summaries(conn, board_id, "c.archived_at IS NULL", "c.position, c.id")
}

/// Archived cards of a board, most recently archived first.
pub fn archived(conn: &Connection, board_id: Id) -> Result<Vec<CardSummary>> {
    summaries(
        conn,
        board_id,
        "c.archived_at IS NOT NULL",
        "c.archived_at DESC, c.id DESC",
    )
}

/// Ids of active cards on a board whose title or notes contain `query`,
/// ignoring case. Compared in Rust because SQLite's LIKE only folds ASCII.
pub fn search(conn: &Connection, board_id: Id, query: &str) -> Result<Vec<Id>> {
    let needle = query.trim().to_lowercase();
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.notes FROM cards c
         JOIN board_columns col ON col.id = c.column_id
         WHERE col.board_id = ?1 AND col.deleted_at IS NULL
           AND c.deleted_at IS NULL AND c.archived_at IS NULL
         ORDER BY c.position, c.id",
    )?;
    let mut ids = Vec::new();
    for row in stmt.query_map([board_id], |row| {
        Ok((
            row.get::<_, Id>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })? {
        let (id, title, notes) = row?;
        if title.to_lowercase().contains(&needle) || notes.to_lowercase().contains(&needle) {
            ids.push(id);
        }
    }
    Ok(ids)
}

pub fn get(conn: &Connection, id: Id) -> Result<CardDetail> {
    live_card(conn, id)?;
    let mut card = conn.query_row(
        "SELECT id, column_id, title, notes, priority, created_at, updated_at, archived_at
         FROM cards WHERE id = ?1",
        [id],
        |row| {
            Ok(CardDetail {
                id: row.get(0)?,
                column_id: row.get(1)?,
                title: row.get(2)?,
                notes: row.get(3)?,
                priority: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                archived_at: row.get(7)?,
                subtasks: Vec::new(),
                tag_ids: Vec::new(),
            })
        },
    )?;
    card.subtasks = subtasks::list(conn, id)?;
    card.tag_ids = tags::card_tag_ids(conn, id)?;
    Ok(card)
}

/// Appends a new card at the end of the column.
pub fn create(conn: &mut Connection, card: &NewCard) -> Result<CardSummary> {
    let title = require_text(&card.title)?;
    let tx = conn.transaction()?;
    columns::live_board_id(&tx, card.column_id)?;
    let key = position::key_after_last(&position_keys(&tx, ACTIVE_KEYS, [card.column_id, 0])?)?;
    let now = now_ms();
    tx.execute(
        "INSERT INTO cards (column_id, title, priority, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![card.column_id, title, card.priority, key, now],
    )?;
    let id = tx.last_insert_rowid();
    let tag_ids = tags::replace_card_tags(&tx, id, &card.tag_ids)?;
    tx.commit()?;

    Ok(CardSummary {
        id,
        column_id: card.column_id,
        title,
        priority: card.priority,
        has_notes: false,
        subtasks_done: 0,
        subtasks_total: 0,
        tag_ids,
    })
}

pub fn update(conn: &mut Connection, id: Id, patch: &CardPatch) -> Result<()> {
    let title = patch.title.as_deref().map(require_text).transpose()?;
    let tx = conn.transaction()?;
    live_card(&tx, id)?;
    if let Some(title) = title {
        tx.execute(
            "UPDATE cards SET title = ?2 WHERE id = ?1",
            params![id, title],
        )?;
    }
    if let Some(notes) = &patch.notes {
        tx.execute(
            "UPDATE cards SET notes = ?2 WHERE id = ?1",
            params![id, notes],
        )?;
    }
    if let Some(priority) = patch.priority {
        tx.execute(
            "UPDATE cards SET priority = ?2 WHERE id = ?1",
            params![id, priority],
        )?;
    }
    touch(&tx, id)?;
    tx.commit()?;
    Ok(())
}

/// Moves an active card to `index` within `column_id` (same or other column).
pub fn move_to(conn: &mut Connection, id: Id, column_id: Id, index: usize) -> Result<()> {
    let tx = conn.transaction()?;
    if live_card(&tx, id)?.archived {
        return Err(AppError::Conflict("card.archived"));
    }
    columns::live_board_id(&tx, column_id)?;
    let key = position::key_at(&position_keys(&tx, ACTIVE_KEYS, [column_id, id])?, index)?;
    tx.execute(
        "UPDATE cards SET column_id = ?2, position = ?3 WHERE id = ?1",
        params![id, column_id, key],
    )?;
    touch(&tx, id)?;
    tx.commit()?;
    Ok(())
}

/// Soft delete; the card can be restored until the app restarts.
pub fn delete(conn: &Connection, id: Id) -> Result<()> {
    live_card(conn, id)?;
    expect_changed(conn.execute(
        "UPDATE cards SET deleted_at = ?2 WHERE id = ?1",
        params![id, now_ms()],
    )?)
}

pub fn restore(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    let (column_id, wanted, archived): (Id, String, bool) = tx
        .query_row(
            "SELECT column_id, position, archived_at IS NOT NULL FROM cards
             WHERE id = ?1 AND deleted_at IS NOT NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or(AppError::NotFound)?;
    columns::live_board_id(&tx, column_id)?;
    // Archived cards have no place in the column order, so keep their key.
    let key = if archived {
        wanted
    } else {
        position::free_key(&position_keys(&tx, ACTIVE_KEYS, [column_id, id])?, &wanted)?
    };
    tx.execute(
        "UPDATE cards SET deleted_at = NULL, position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn archive(conn: &Connection, id: Id) -> Result<()> {
    if live_card(conn, id)?.archived {
        return Err(AppError::Conflict("card.archived"));
    }
    conn.execute(
        "UPDATE cards SET archived_at = ?2 WHERE id = ?1",
        params![id, now_ms()],
    )?;
    Ok(())
}

/// Puts an archived card back into its column, at its old place if free.
pub fn unarchive(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    let state = live_card(&tx, id)?;
    if !state.archived {
        return Err(AppError::Conflict("card.not_archived"));
    }
    let wanted: String = tx.query_row("SELECT position FROM cards WHERE id = ?1", [id], |row| {
        row.get(0)
    })?;
    let key = position::free_key(
        &position_keys(&tx, ACTIVE_KEYS, [state.column_id, id])?,
        &wanted,
    )?;
    tx.execute(
        "UPDATE cards SET archived_at = NULL, position = ?2 WHERE id = ?1",
        params![id, key],
    )?;
    tx.commit()?;
    Ok(())
}

/// Archives every active card in the board's done column and returns their
/// ids, so the frontend can offer undo.
pub fn archive_done(conn: &mut Connection, board_id: Id) -> Result<Vec<Id>> {
    let tx = conn.transaction()?;
    let ids: Vec<Id> = {
        let mut stmt = tx.prepare(
            "SELECT c.id FROM cards c
             JOIN board_columns col ON col.id = c.column_id
             JOIN boards b ON b.id = col.board_id
             WHERE col.board_id = ?1 AND col.is_done = 1 AND col.deleted_at IS NULL
               AND b.deleted_at IS NULL
               AND c.deleted_at IS NULL AND c.archived_at IS NULL
             ORDER BY c.position, c.id",
        )?;
        stmt.query_map([board_id], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let now = now_ms();
    for id in &ids {
        tx.execute(
            "UPDATE cards SET archived_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
    }
    tx.commit()?;
    Ok(ids)
}

pub fn set_tags(conn: &mut Connection, id: Id, tag_ids: &[Id]) -> Result<Vec<Id>> {
    let tx = conn.transaction()?;
    live_card(&tx, id)?;
    let tag_ids = tags::replace_card_tags(&tx, id, tag_ids)?;
    touch(&tx, id)?;
    tx.commit()?;
    Ok(tag_ids)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::{BoardView, Priority};
    use crate::store::boards::{self, tests::default_columns};
    use crate::store::open_in_memory;

    pub(crate) struct Fixture {
        pub conn: Connection,
        pub board_id: Id,
        pub columns: Vec<Id>,
    }

    pub(crate) fn fixture() -> Fixture {
        let mut conn = open_in_memory();
        let board = boards::create(&mut conn, "Board", &default_columns()).unwrap();
        let columns = boards::view(&conn, board.id)
            .unwrap()
            .columns
            .iter()
            .map(|c| c.column.id)
            .collect();
        Fixture {
            conn,
            board_id: board.id,
            columns,
        }
    }

    pub(crate) fn add(conn: &mut Connection, column_id: Id, title: &str) -> Id {
        create(
            conn,
            &NewCard {
                column_id,
                title: title.into(),
                ..Default::default()
            },
        )
        .unwrap()
        .id
    }

    fn titles(view: &BoardView) -> Vec<Vec<String>> {
        view.columns
            .iter()
            .map(|c| c.cards.iter().map(|card| card.title.clone()).collect())
            .collect()
    }

    fn board_titles(f: &Fixture) -> Vec<Vec<String>> {
        titles(&boards::view(&f.conn, f.board_id).unwrap())
    }

    #[test]
    fn create_appends_to_column() {
        let mut f = fixture();
        add(&mut f.conn, f.columns[0], "First");
        let second = create(
            &mut f.conn,
            &NewCard {
                column_id: f.columns[0],
                title: "  Second ".into(),
                priority: Priority::High,
                tag_ids: vec![],
            },
        )
        .unwrap();
        assert_eq!(second.title, "Second");
        assert_eq!(second.priority, Priority::High);
        assert_eq!(board_titles(&f), [vec!["First", "Second"], vec![], vec![]]);
    }

    #[test]
    fn create_validates_title_and_column() {
        let mut f = fixture();
        let bad_title = NewCard {
            column_id: f.columns[0],
            title: " ".into(),
            ..Default::default()
        };
        assert!(matches!(
            create(&mut f.conn, &bad_title),
            Err(AppError::Validation(_))
        ));
        let bad_column = NewCard {
            column_id: 999,
            title: "X".into(),
            ..Default::default()
        };
        assert!(matches!(
            create(&mut f.conn, &bad_column),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn move_within_and_across_columns() {
        let mut f = fixture();
        let a = add(&mut f.conn, f.columns[0], "A");
        add(&mut f.conn, f.columns[0], "B");
        let c = add(&mut f.conn, f.columns[0], "C");

        move_to(&mut f.conn, c, f.columns[0], 0).unwrap();
        assert_eq!(board_titles(&f)[0], ["C", "A", "B"]);

        move_to(&mut f.conn, a, f.columns[1], 0).unwrap();
        assert_eq!(board_titles(&f), [vec!["C", "B"], vec!["A"], vec![]]);

        move_to(&mut f.conn, c, f.columns[1], 99).unwrap();
        assert_eq!(board_titles(&f), [vec!["B"], vec!["A", "C"], vec![]]);
    }

    #[test]
    fn update_changes_only_given_fields() {
        let mut f = fixture();
        let id = add(&mut f.conn, f.columns[0], "Title");
        update(
            &mut f.conn,
            id,
            &CardPatch {
                notes: Some("Some notes".into()),
                priority: Some(Priority::Medium),
                ..Default::default()
            },
        )
        .unwrap();
        let card = get(&f.conn, id).unwrap();
        assert_eq!(card.title, "Title");
        assert_eq!(card.notes, "Some notes");
        assert_eq!(card.priority, Priority::Medium);
        assert!(card.updated_at >= card.created_at);

        let summary = &boards::view(&f.conn, f.board_id).unwrap().columns[0].cards[0];
        assert!(summary.has_notes);

        assert!(matches!(
            update(
                &mut f.conn,
                id,
                &CardPatch {
                    title: Some("".into()),
                    ..Default::default()
                }
            ),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn delete_and_restore_in_place() {
        let mut f = fixture();
        add(&mut f.conn, f.columns[0], "A");
        let b = add(&mut f.conn, f.columns[0], "B");
        add(&mut f.conn, f.columns[0], "C");

        delete(&f.conn, b).unwrap();
        assert_eq!(board_titles(&f)[0], ["A", "C"]);
        assert!(matches!(get(&f.conn, b), Err(AppError::NotFound)));

        restore(&mut f.conn, b).unwrap();
        assert_eq!(board_titles(&f)[0], ["A", "B", "C"]);
    }

    #[test]
    fn restore_after_a_card_took_the_same_position() {
        let mut f = fixture();
        let a = add(&mut f.conn, f.columns[0], "A");
        delete(&f.conn, a).unwrap();
        // The new card gets the same key "V" the deleted one had.
        add(&mut f.conn, f.columns[0], "New");
        restore(&mut f.conn, a).unwrap();
        assert_eq!(board_titles(&f)[0], ["New", "A"]);
    }

    #[test]
    fn cards_of_deleted_column_are_hidden_and_come_back() {
        let mut f = fixture();
        let card = add(&mut f.conn, f.columns[1], "Card");
        columns::delete(&f.conn, f.columns[1]).unwrap();
        assert!(matches!(get(&f.conn, card), Err(AppError::NotFound)));
        columns::restore(&mut f.conn, f.columns[1]).unwrap();
        assert_eq!(get(&f.conn, card).unwrap().title, "Card");
    }

    #[test]
    fn archive_and_unarchive() {
        let mut f = fixture();
        add(&mut f.conn, f.columns[0], "A");
        let b = add(&mut f.conn, f.columns[0], "B");
        add(&mut f.conn, f.columns[0], "C");

        archive(&f.conn, b).unwrap();
        assert_eq!(board_titles(&f)[0], ["A", "C"]);
        let archived_titles: Vec<_> = archived(&f.conn, f.board_id)
            .unwrap()
            .into_iter()
            .map(|c| c.title)
            .collect();
        assert_eq!(archived_titles, ["B"]);
        assert!(get(&f.conn, b).unwrap().archived_at.is_some());
        assert!(matches!(
            move_to(&mut f.conn, b, f.columns[1], 0),
            Err(AppError::Conflict("card.archived"))
        ));
        assert!(matches!(
            archive(&f.conn, b),
            Err(AppError::Conflict("card.archived"))
        ));

        unarchive(&mut f.conn, b).unwrap();
        assert_eq!(board_titles(&f)[0], ["A", "B", "C"]);
        assert!(archived(&f.conn, f.board_id).unwrap().is_empty());
        assert!(matches!(
            unarchive(&mut f.conn, b),
            Err(AppError::Conflict("card.not_archived"))
        ));
    }

    #[test]
    fn archive_done_archives_only_the_done_column() {
        let mut f = fixture();
        add(&mut f.conn, f.columns[0], "Open");
        let d1 = add(&mut f.conn, f.columns[2], "Done 1");
        let d2 = add(&mut f.conn, f.columns[2], "Done 2");

        let ids = archive_done(&mut f.conn, f.board_id).unwrap();
        assert_eq!(ids, [d1, d2]);
        assert_eq!(board_titles(&f), [vec!["Open"], vec![], vec![]]);
        assert_eq!(archived(&f.conn, f.board_id).unwrap().len(), 2);
    }

    #[test]
    fn search_matches_title_and_notes_ignoring_case() {
        let mut f = fixture();
        let a = add(&mut f.conn, f.columns[0], "Über das Angebot sprechen");
        let b = add(&mut f.conn, f.columns[1], "Call");
        let archived = add(&mut f.conn, f.columns[0], "Angebot alt");
        update(
            &mut f.conn,
            b,
            &CardPatch {
                notes: Some("Thema: ANGEBOT".into()),
                ..Default::default()
            },
        )
        .unwrap();
        archive(&f.conn, archived).unwrap();

        assert_eq!(search(&f.conn, f.board_id, "angebot").unwrap(), [a, b]);
        assert_eq!(search(&f.conn, f.board_id, " über ").unwrap(), [a]);
        assert!(search(&f.conn, f.board_id, "nichts").unwrap().is_empty());
    }

    #[test]
    fn summary_counts_subtasks() {
        let mut f = fixture();
        let id = add(&mut f.conn, f.columns[0], "Card");
        let first = subtasks::create(&mut f.conn, id, "One").unwrap();
        subtasks::create(&mut f.conn, id, "Two").unwrap();
        subtasks::update(
            &f.conn,
            first.id,
            &crate::model::SubtaskPatch {
                done: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let summary = &boards::view(&f.conn, f.board_id).unwrap().columns[0].cards[0];
        assert_eq!((summary.subtasks_done, summary.subtasks_total), (1, 2));
    }
}
