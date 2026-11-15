//! Tauri commands: the only interface between frontend and backend.
//! Each command locks the vault or the database and delegates to it.

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, Result};
use crate::library::{Library, VaultInfo};
use crate::model::{
    Board, BoardView, CardDetail, CardPatch, CardSummary, CardVersion, ColorPreset,
    ColorPresetPatch, Column, Id, ImagePatch, MoodboardImage, MoodboardView, NewCard, PresetColors,
    PresetId, Subtask, SubtaskPatch, Tag, TagPatch, VaultProblem,
};
use crate::store::{pomodoro, presets};

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
pub async fn get_vault(library: State<'_, Library>) -> Result<VaultInfo> {
    library.info()
}

/// Opens another folder as the vault; an empty folder becomes a new vault.
#[tauri::command]
pub async fn open_vault(
    app: AppHandle,
    db: State<'_, Db>,
    library: State<'_, Library>,
    path: String,
) -> Result<VaultInfo> {
    let info = db.with(|conn| library.switch(conn, std::path::Path::new(&path)))?;
    library.watch(&app)?;
    Ok(info)
}

/// Tries the configured vault folder again.
#[tauri::command]
pub async fn reopen_vault(
    app: AppHandle,
    db: State<'_, Db>,
    library: State<'_, Library>,
) -> Result<VaultInfo> {
    let info = db.with(|conn| library.reopen(&app, conn))?;
    library.watch(&app)?;
    Ok(info)
}

/// Checks the vault's files for changes from outside, for folders the system
/// cannot watch. Windows call it when they get focus; returns whether
/// anything changed (the windows are then told through the usual event).
#[tauri::command]
pub async fn refresh_vault(app: AppHandle) -> Result<bool> {
    Ok(crate::watch::refresh(&app))
}

#[tauri::command]
pub async fn list_vault_problems(library: State<'_, Library>) -> Result<Vec<VaultProblem>> {
    library.with(|store| Ok(store.list_problems()))
}

#[tauri::command]
pub async fn get_card_conflicts(
    library: State<'_, Library>,
    card_id: Id,
) -> Result<Vec<CardVersion>> {
    library.with(|store| store.card_conflicts(&card_id))
}

/// Keeps the version at `keep` (a conflict copy), or the card as it is if
/// `keep` is null, and removes the other copies.
#[tauri::command]
pub async fn resolve_card_conflict(
    library: State<'_, Library>,
    card_id: Id,
    keep: Option<String>,
) -> Result<()> {
    library.with(|store| store.resolve_card_conflict(&card_id, keep.as_deref()))
}

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
pub async fn create_moodboard(library: State<'_, Library>, name: String) -> Result<Board> {
    library.with(|store| store.create_moodboard(&name))
}

#[tauri::command]
pub async fn get_moodboard(library: State<'_, Library>, board_id: Id) -> Result<MoodboardView> {
    library.with(|store| store.moodboard_view(&board_id))
}

/// Copies an image file the user picked or dropped onto the moodboard.
#[tauri::command]
pub async fn add_moodboard_image(
    library: State<'_, Library>,
    board_id: Id,
    path: String,
    x: f64,
    y: f64,
) -> Result<MoodboardImage> {
    library.with(|store| store.add_image_file(&board_id, std::path::Path::new(&path), x, y))
}

/// Adds a pasted image. The bytes come as the raw request body (no JSON),
/// the moodboard and position as headers.
#[tauri::command]
pub async fn add_moodboard_image_data(
    library: State<'_, Library>,
    request: tauri::ipc::Request<'_>,
) -> Result<MoodboardImage> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(AppError::Validation("image.unsupported"));
    };
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    };
    let board_id = header("x-board-id").ok_or(AppError::NotFound)?;
    let coordinate = |name| {
        header(name)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0.0)
    };
    library.with(|store| store.add_image(board_id, bytes, coordinate("x-x"), coordinate("x-y")))
}

#[tauri::command]
pub async fn update_moodboard_image(
    library: State<'_, Library>,
    image_id: Id,
    patch: ImagePatch,
) -> Result<()> {
    library.with(|store| store.update_image(&image_id, &patch))
}

/// Moves an image to another layer: 0 is the back.
#[tauri::command]
pub async fn reorder_moodboard_image(
    library: State<'_, Library>,
    image_id: Id,
    index: usize,
) -> Result<()> {
    library.with(|store| store.reorder_image(&image_id, index))
}

#[tauri::command]
pub async fn delete_moodboard_image(library: State<'_, Library>, image_id: Id) -> Result<()> {
    library.with(|store| store.delete_image(&image_id))
}

#[tauri::command]
pub async fn restore_moodboard_image(library: State<'_, Library>, image_id: Id) -> Result<()> {
    library.with(|store| store.restore_image(&image_id))
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

#[tauri::command]
pub async fn get_pomodoro_sound(db: State<'_, Db>) -> Result<Option<String>> {
    db.with(|conn| pomodoro::sound(conn))
}

#[tauri::command]
pub async fn set_pomodoro_sound(db: State<'_, Db>, sound: String) -> Result<()> {
    db.with(|conn| pomodoro::set_sound(conn, &sound))
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
