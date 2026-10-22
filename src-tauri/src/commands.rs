//! Tauri commands: the only interface between frontend and backend.
//! Each command locks the database and delegates to the storage layer.

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::error::{AppError, Result};
use crate::model::{
    Board, BoardView, CardDetail, CardPatch, CardSummary, Column, Id, NewCard, Subtask,
    SubtaskPatch, Tag, TagPatch,
};
use crate::store::{boards, cards, columns, subtasks, tags};

pub struct Db(Mutex<Connection>);

impl Db {
    pub fn new(conn: Connection) -> Self {
        Db(Mutex::new(conn))
    }

    fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut conn = self
            .0
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned"))?;
        f(&mut conn)
    }
}

// Async so that database work runs off the main (UI) thread.

#[tauri::command]
pub async fn list_boards(db: State<'_, Db>) -> Result<Vec<Board>> {
    db.with(|conn| boards::list(conn))
}

#[tauri::command]
pub async fn get_board(db: State<'_, Db>, board_id: Id) -> Result<BoardView> {
    db.with(|conn| boards::view(conn, board_id))
}

#[tauri::command]
pub async fn create_board(
    db: State<'_, Db>,
    name: String,
    column_names: Vec<String>,
) -> Result<Board> {
    db.with(|conn| boards::create(conn, &name, &column_names))
}

#[tauri::command]
pub async fn rename_board(db: State<'_, Db>, board_id: Id, name: String) -> Result<()> {
    db.with(|conn| boards::rename(conn, board_id, &name))
}

#[tauri::command]
pub async fn reorder_board(db: State<'_, Db>, board_id: Id, index: usize) -> Result<()> {
    db.with(|conn| boards::reorder(conn, board_id, index))
}

#[tauri::command]
pub async fn delete_board(db: State<'_, Db>, board_id: Id) -> Result<()> {
    db.with(|conn| boards::delete(conn, board_id))
}

#[tauri::command]
pub async fn restore_board(db: State<'_, Db>, board_id: Id) -> Result<()> {
    db.with(|conn| boards::restore(conn, board_id))
}

#[tauri::command]
pub async fn create_column(db: State<'_, Db>, board_id: Id, name: String) -> Result<Column> {
    db.with(|conn| columns::create(conn, board_id, &name))
}

#[tauri::command]
pub async fn rename_column(db: State<'_, Db>, column_id: Id, name: String) -> Result<()> {
    db.with(|conn| columns::rename(conn, column_id, &name))
}

#[tauri::command]
pub async fn reorder_column(db: State<'_, Db>, column_id: Id, index: usize) -> Result<()> {
    db.with(|conn| columns::reorder(conn, column_id, index))
}

#[tauri::command]
pub async fn set_done_column(db: State<'_, Db>, column_id: Id) -> Result<()> {
    db.with(|conn| columns::set_done(conn, column_id))
}

#[tauri::command]
pub async fn delete_column(db: State<'_, Db>, column_id: Id) -> Result<()> {
    db.with(|conn| columns::delete(conn, column_id))
}

#[tauri::command]
pub async fn restore_column(db: State<'_, Db>, column_id: Id) -> Result<()> {
    db.with(|conn| columns::restore(conn, column_id))
}

#[tauri::command]
pub async fn get_card(db: State<'_, Db>, card_id: Id) -> Result<CardDetail> {
    db.with(|conn| cards::get(conn, card_id))
}

#[tauri::command]
pub async fn create_card(db: State<'_, Db>, card: NewCard) -> Result<CardSummary> {
    db.with(|conn| cards::create(conn, &card))
}

#[tauri::command]
pub async fn update_card(db: State<'_, Db>, card_id: Id, patch: CardPatch) -> Result<()> {
    db.with(|conn| cards::update(conn, card_id, &patch))
}

#[tauri::command]
pub async fn move_card(db: State<'_, Db>, card_id: Id, column_id: Id, index: usize) -> Result<()> {
    db.with(|conn| cards::move_to(conn, card_id, column_id, index))
}

#[tauri::command]
pub async fn delete_card(db: State<'_, Db>, card_id: Id) -> Result<()> {
    db.with(|conn| cards::delete(conn, card_id))
}

#[tauri::command]
pub async fn restore_card(db: State<'_, Db>, card_id: Id) -> Result<()> {
    db.with(|conn| cards::restore(conn, card_id))
}

#[tauri::command]
pub async fn archive_card(db: State<'_, Db>, card_id: Id) -> Result<()> {
    db.with(|conn| cards::archive(conn, card_id))
}

#[tauri::command]
pub async fn unarchive_card(db: State<'_, Db>, card_id: Id) -> Result<()> {
    db.with(|conn| cards::unarchive(conn, card_id))
}

#[tauri::command]
pub async fn archive_done_cards(db: State<'_, Db>, board_id: Id) -> Result<Vec<Id>> {
    db.with(|conn| cards::archive_done(conn, board_id))
}

#[tauri::command]
pub async fn list_archived_cards(db: State<'_, Db>, board_id: Id) -> Result<Vec<CardSummary>> {
    db.with(|conn| cards::archived(conn, board_id))
}

#[tauri::command]
pub async fn set_card_tags(db: State<'_, Db>, card_id: Id, tag_ids: Vec<Id>) -> Result<Vec<Id>> {
    db.with(|conn| cards::set_tags(conn, card_id, &tag_ids))
}

#[tauri::command]
pub async fn create_subtask(db: State<'_, Db>, card_id: Id, title: String) -> Result<Subtask> {
    db.with(|conn| subtasks::create(conn, card_id, &title))
}

#[tauri::command]
pub async fn update_subtask(db: State<'_, Db>, subtask_id: Id, patch: SubtaskPatch) -> Result<()> {
    db.with(|conn| subtasks::update(conn, subtask_id, &patch))
}

#[tauri::command]
pub async fn reorder_subtask(db: State<'_, Db>, subtask_id: Id, index: usize) -> Result<()> {
    db.with(|conn| subtasks::reorder(conn, subtask_id, index))
}

#[tauri::command]
pub async fn delete_subtask(db: State<'_, Db>, subtask_id: Id) -> Result<()> {
    db.with(|conn| subtasks::delete(conn, subtask_id))
}

#[tauri::command]
pub async fn list_tags(db: State<'_, Db>) -> Result<Vec<Tag>> {
    db.with(|conn| tags::list(conn))
}

#[tauri::command]
pub async fn create_tag(db: State<'_, Db>, name: String, color: String) -> Result<Tag> {
    db.with(|conn| tags::create(conn, &name, &color))
}

#[tauri::command]
pub async fn update_tag(db: State<'_, Db>, tag_id: Id, patch: TagPatch) -> Result<()> {
    db.with(|conn| tags::update(conn, tag_id, &patch))
}

#[tauri::command]
pub async fn delete_tag(db: State<'_, Db>, tag_id: Id) -> Result<()> {
    db.with(|conn| tags::delete(conn, tag_id))
}
