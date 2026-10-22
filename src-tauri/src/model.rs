//! Data sent to and received from the frontend.

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

pub type Id = i64;

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
