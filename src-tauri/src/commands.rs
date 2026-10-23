//! Tauri commands: the only interface between frontend and backend.
//! Each command locks the vault or the database and delegates to it.

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, Result};
use crate::library::Library;
use crate::model::{
    Board, BoardView, CardDetail, CardPatch, CardSummary, ColorPreset, ColorPresetPatch, Column,
    Id, NewCard, PresetColors, PresetId, Subtask, SubtaskPatch, Tag, TagPatch,
};
use crate::store::presets;

pub struct Db(Mutex<Connection>);

impl Db {
    pub fn new(conn: Connection) -> Self {
        Db(Mutex::new(conn))
    }

    pub(crate) fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut conn = self
            .0
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned"))?;
        f(&mut conn)
    }
}

// Async so that file and database work runs off the main (UI) thread.

#[tauri::command]
pub async fn list_boards(library: State<'_, Library>) -> Result<Vec<Board>> {
    library.with(|store| Ok(store.list_boards()))
}

#[tauri::command]
pub async fn get_board(library: State<'_, Library>, board_id: Id) -> Result<BoardView> {
    library.with(|store| store.view(&board_id))
}

#[tauri::command]
pub async fn create_board(
    library: State<'_, Library>,
    name: String,
    column_names: Vec<String>,
) -> Result<Board> {
    library.with(|store| store.create_board(&name, &column_names))
}

#[tauri::command]
pub async fn rename_board(library: State<'_, Library>, board_id: Id, name: String) -> Result<()> {
    library.with(|store| store.rename_board(&board_id, &name))
}

#[tauri::command]
pub async fn reorder_board(library: State<'_, Library>, board_id: Id, index: usize) -> Result<()> {
    library.with(|store| store.reorder_board(&board_id, index))
}

#[tauri::command]
pub async fn delete_board(library: State<'_, Library>, board_id: Id) -> Result<()> {
    library.with(|store| store.delete_board(&board_id))
}

#[tauri::command]
pub async fn restore_board(library: State<'_, Library>, board_id: Id) -> Result<()> {
    library.with(|store| store.restore_board(&board_id))
}

#[tauri::command]
pub async fn create_column(
    library: State<'_, Library>,
    board_id: Id,
    name: String,
) -> Result<Column> {
    library.with(|store| store.create_column(&board_id, &name))
}

#[tauri::command]
pub async fn rename_column(library: State<'_, Library>, column_id: Id, name: String) -> Result<()> {
    library.with(|store| store.rename_column(&column_id, &name))
}

#[tauri::command]
pub async fn reorder_column(
    library: State<'_, Library>,
    column_id: Id,
    index: usize,
) -> Result<()> {
    library.with(|store| store.reorder_column(&column_id, index))
}

#[tauri::command]
pub async fn set_done_column(library: State<'_, Library>, column_id: Id) -> Result<()> {
    library.with(|store| store.set_done_column(&column_id))
}

#[tauri::command]
pub async fn delete_column(library: State<'_, Library>, column_id: Id) -> Result<()> {
    library.with(|store| store.delete_column(&column_id))
}

#[tauri::command]
pub async fn restore_column(library: State<'_, Library>, column_id: Id) -> Result<()> {
    library.with(|store| store.restore_column(&column_id))
}

#[tauri::command]
pub async fn search_cards(
    library: State<'_, Library>,
    board_id: Id,
    query: String,
) -> Result<Vec<Id>> {
    library.with(|store| store.search_cards(&board_id, &query))
}

#[tauri::command]
pub async fn get_card(library: State<'_, Library>, card_id: Id) -> Result<CardDetail> {
    library.with(|store| store.get_card(&card_id))
}

#[tauri::command]
pub async fn create_card(library: State<'_, Library>, card: NewCard) -> Result<CardSummary> {
    library.with(|store| store.create_card(&card))
}

#[tauri::command]
pub async fn update_card(library: State<'_, Library>, card_id: Id, patch: CardPatch) -> Result<()> {
    library.with(|store| store.update_card(&card_id, &patch))
}

#[tauri::command]
pub async fn move_card(
    library: State<'_, Library>,
    card_id: Id,
    column_id: Id,
    index: usize,
) -> Result<()> {
    library.with(|store| store.move_card(&card_id, &column_id, index))
}

#[tauri::command]
pub async fn delete_card(library: State<'_, Library>, card_id: Id) -> Result<()> {
    library.with(|store| store.delete_card(&card_id))
}

#[tauri::command]
pub async fn restore_card(library: State<'_, Library>, card_id: Id) -> Result<()> {
    library.with(|store| store.restore_card(&card_id))
}

#[tauri::command]
pub async fn archive_card(library: State<'_, Library>, card_id: Id) -> Result<()> {
    library.with(|store| store.archive_card(&card_id))
}

#[tauri::command]
pub async fn unarchive_card(library: State<'_, Library>, card_id: Id) -> Result<()> {
    library.with(|store| store.unarchive_card(&card_id))
}

#[tauri::command]
pub async fn archive_done_cards(library: State<'_, Library>, board_id: Id) -> Result<Vec<Id>> {
    library.with(|store| store.archive_done_cards(&board_id))
}

#[tauri::command]
pub async fn list_archived_cards(
    library: State<'_, Library>,
    board_id: Id,
) -> Result<Vec<CardSummary>> {
    library.with(|store| store.list_archived_cards(&board_id))
}

#[tauri::command]
pub async fn set_card_tags(
    library: State<'_, Library>,
    card_id: Id,
    tag_ids: Vec<Id>,
) -> Result<Vec<Id>> {
    library.with(|store| store.set_card_tags(&card_id, &tag_ids))
}

#[tauri::command]
pub async fn create_subtask(
    library: State<'_, Library>,
    card_id: Id,
    title: String,
) -> Result<Subtask> {
    library.with(|store| store.create_subtask(&card_id, &title))
}

#[tauri::command]
pub async fn update_subtask(
    library: State<'_, Library>,
    subtask_id: Id,
    patch: SubtaskPatch,
) -> Result<()> {
    library.with(|store| store.update_subtask(&subtask_id, &patch))
}

#[tauri::command]
pub async fn reorder_subtask(
    library: State<'_, Library>,
    subtask_id: Id,
    index: usize,
) -> Result<()> {
    library.with(|store| store.reorder_subtask(&subtask_id, index))
}

#[tauri::command]
pub async fn delete_subtask(library: State<'_, Library>, subtask_id: Id) -> Result<()> {
    library.with(|store| store.delete_subtask(&subtask_id))
}

#[tauri::command]
pub async fn list_tags(library: State<'_, Library>) -> Result<Vec<Tag>> {
    library.with(|store| Ok(store.list_tags()))
}

#[tauri::command]
pub async fn create_tag(library: State<'_, Library>, name: String, color: String) -> Result<Tag> {
    library.with(|store| store.create_tag(&name, &color))
}

#[tauri::command]
pub async fn update_tag(library: State<'_, Library>, tag_id: Id, patch: TagPatch) -> Result<()> {
    library.with(|store| store.update_tag(&tag_id, &patch))
}

#[tauri::command]
pub async fn count_tag_cards(library: State<'_, Library>, tag_id: Id) -> Result<u32> {
    library.with(|store| store.count_tag_cards(&tag_id))
}

#[tauri::command]
pub async fn delete_tag(library: State<'_, Library>, tag_id: Id) -> Result<()> {
    library.with(|store| store.delete_tag(&tag_id))
}

#[tauri::command]
pub async fn list_color_presets(db: State<'_, Db>) -> Result<Vec<ColorPreset>> {
    db.with(|conn| presets::list(conn))
}

#[tauri::command]
pub async fn create_color_preset(
    db: State<'_, Db>,
    name: String,
    colors: PresetColors,
) -> Result<ColorPreset> {
    db.with(|conn| presets::create(conn, &name, &colors))
}

#[tauri::command]
pub async fn update_color_preset(
    db: State<'_, Db>,
    preset_id: PresetId,
    patch: ColorPresetPatch,
) -> Result<()> {
    db.with(|conn| presets::update(conn, preset_id, &patch))
}

#[tauri::command]
pub async fn delete_color_preset(db: State<'_, Db>, preset_id: PresetId) -> Result<()> {
    db.with(|conn| presets::delete(conn, preset_id))
}

#[tauri::command]
pub async fn get_color_preset_choice(db: State<'_, Db>) -> Result<String> {
    db.with(|conn| presets::choice(conn))
}

#[tauri::command]
pub async fn set_color_preset_choice(db: State<'_, Db>, choice: String) -> Result<()> {
    db.with(|conn| presets::set_choice(conn, &choice))
}

/// Bytes the app's data takes on disk: the vault and the app's own database.
#[tauri::command]
pub async fn get_storage_usage(app: AppHandle, library: State<'_, Library>) -> Result<u64> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Internal("no app data directory"))?;
    let vault = library
        .with(|store| Ok(store.vault().disk_usage()))
        .unwrap_or(0);
    Ok(crate::store::disk_usage(&dir) + vault)
}
