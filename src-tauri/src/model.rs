//! Data sent to and received from the frontend.

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

/// Boards, columns, cards, subtasks and tags carry ULIDs from the vault.
pub type Id = String;
/// Colour presets live in the app's database and keep its row ids.
pub type PresetId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
}

impl ToSql for Priority {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(*self as i64))
    }
}

impl FromSql for Priority {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(Priority::None),
            1 => Ok(Priority::Low),
            2 => Ok(Priority::Medium),
            3 => Ok(Priority::High),
            other => Err(FromSqlError::OutOfRange(other)),
        }
    }
}

/// Palette keys for tag colours; the frontend maps each to a theme token.
pub const TAG_COLORS: &[&str] = &[
    "gray", "red", "orange", "yellow", "green", "teal", "blue", "purple",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: Id,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub id: Id,
    pub name: String,
    pub is_done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSummary {
    pub id: Id,
    pub column_id: Id,
    pub title: String,
    pub priority: Priority,
    pub has_notes: bool,
    pub subtasks_done: u32,
    pub subtasks_total: u32,
    pub tag_ids: Vec<Id>,
    /// A second file for this card exists, e.g. a sync service's conflict copy.
    pub has_conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnView {
    #[serde(flatten)]
    pub column: Column,
    pub cards: Vec<CardSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardView {
    pub board: Board,
    pub columns: Vec<ColumnView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtask {
    pub id: Id,
    pub title: String,
    pub done: bool,
}

/// Timestamps are Unix milliseconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDetail {
    pub id: Id,
    pub column_id: Id,
    pub title: String,
    pub notes: String,
    pub priority: Priority,
    pub created_at: i64,
    pub updated_at: i64,
    pub archived_at: Option<i64>,
    pub subtasks: Vec<Subtask>,
    pub tag_ids: Vec<Id>,
    pub has_conflict: bool,
}

/// Another version of a card, from a conflict copy of its file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardVersion {
    /// Relative to the vault; identifies the version when resolving.
    pub path: String,
    pub title: String,
    pub notes: String,
    pub updated_at: i64,
}

/// A file in the vault that was skipped while loading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultProblem {
    /// `invalid` (unreadable) or `conflict` (second copy of `original`).
    pub kind: &'static str,
    /// Relative to the vault, with `/` as separator.
    pub path: String,
    /// Absolute, for showing it in the file manager.
    pub full_path: String,
    pub original: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Id,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCard {
    pub column_id: Id,
    pub title: String,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tag_ids: Vec<Id>,
}

/// Only the fields that are present are changed.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardPatch {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtaskPatch {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagPatch {
    pub name: Option<String>,
    pub color: Option<String>,
}

/// Names of the colour presets built into the frontend; the first is the default.
pub const BUILTIN_PRESETS: &[&str] = &["hodie", "ocean", "forest"];

/// Accent and base colour (`#rrggbb`) of a preset for each mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetColors {
    pub light_accent: String,
    pub light_base: String,
    pub dark_accent: String,
    pub dark_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPreset {
    pub id: PresetId,
    pub name: String,
    pub colors: PresetColors,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPresetPatch {
    pub name: Option<String>,
    pub colors: Option<PresetColors>,
}
