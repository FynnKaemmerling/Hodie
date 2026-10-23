//! The files of a vault and how they are read and written, see `VAULT.md`.
//! Parsing returns `None` for anything that is not a valid file; the caller
//! reports it and leaves the file untouched.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::time;
use crate::error::{AppError, Result};
use crate::model::Priority;
use crate::position;

/// Fields an app version does not know (e.g. written by a newer one). They are
/// kept so that rewriting a file never drops them.
pub type JsonExtra = serde_json::Map<String, serde_json::Value>;
pub type YamlExtra = BTreeMap<String, serde_yaml_ng::Value>;

/// Unix milliseconds, stored as ISO 8601 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp(pub i64);

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&time::format(self.0))
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        time::parse(&text)
            .map(Timestamp)
            .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
    }
}

/// `.hodie/vault.json`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultFile {
    pub format_version: u32,
    pub id: String,
    #[serde(flatten)]
    pub extra: JsonExtra,
}

/// `.hodie/tags.json`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TagsFile {
    #[serde(default)]
    pub tags: Vec<TagEntry>,
    #[serde(flatten)]
    pub extra: JsonExtra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagEntry {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub extra: JsonExtra,
}

/// `<board>/board.json`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardFile {
    pub id: String,
    pub name: String,
    pub position: String,
    pub created_at: Timestamp,
    #[serde(default)]
    pub columns: Vec<ColumnEntry>,
    #[serde(flatten)]
    pub extra: JsonExtra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnEntry {
    pub id: String,
    pub name: String,
    pub position: String,
    #[serde(default)]
    pub is_done: bool,
    #[serde(flatten)]
    pub extra: JsonExtra,
}

/// `<board>/cards/<id>.md`: the frontmatter plus the notes below it.
#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub meta: CardMeta,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMeta {
    pub id: String,
    pub column: String,
    pub position: String,
    pub title: String,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtasks: Vec<SubtaskEntry>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<Timestamp>,
    #[serde(flatten)]
    pub extra: YamlExtra,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtaskEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub done: bool,
    #[serde(flatten)]
    pub extra: YamlExtra,
}

fn has_text(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn parse_vault(text: &str) -> Option<VaultFile> {
    let file: VaultFile = serde_json::from_str(strip_bom(text)).ok()?;
    has_text(&file.id).then_some(file)
}

pub fn parse_tags(text: &str) -> Option<TagsFile> {
    let file: TagsFile = serde_json::from_str(strip_bom(text)).ok()?;
    file.tags
        .iter()
        .all(|tag| has_text(&tag.id) && has_text(&tag.name))
        .then_some(file)
}

/// Columns come back sorted, with exactly one done column (see `normalize`).
pub fn parse_board(text: &str) -> Option<BoardFile> {
    let mut file: BoardFile = serde_json::from_str(strip_bom(text)).ok()?;
    let valid = has_text(&file.id)
        && has_text(&file.name)
        && position::is_valid(&file.position)
        && file.columns.iter().all(|column| {
            has_text(&column.id) && has_text(&column.name) && position::is_valid(&column.position)
        });
    if !valid {
        return None;
    }
    file.normalize();
    Some(file)
}

impl BoardFile {
    /// Sorts the columns and repairs the done marker, which a hand merge can
    /// leave missing or doubled: the last marked column wins, else the last one.
    pub fn normalize(&mut self) {
        self.columns
            .sort_by(|a, b| (&a.position, &a.id).cmp(&(&b.position, &b.id)));
        let done = self
            .columns
            .iter()
            .rposition(|column| column.is_done)
            .or(self.columns.len().checked_sub(1));
        for (index, column) in self.columns.iter_mut().enumerate() {
            column.is_done = Some(index) == done;
        }
    }
}

pub fn parse_card(text: &str) -> Option<Card> {
    let text = strip_bom(text).replace("\r\n", "\n");
    let rest = text.strip_prefix("---\n")?;

    let mut offset = 0;
    let mut split = None;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            split = Some((&rest[..offset], &rest[offset + line.len()..]));
            break;
        }
        offset += line.len();
    }
    let (front, body) = split?;

    let meta: CardMeta = serde_yaml_ng::from_str(front).ok()?;
    let valid = has_text(&meta.id)
        && has_text(&meta.column)
        && has_text(&meta.title)
        && position::is_valid(&meta.position)
        && meta.subtasks.iter().all(|subtask| has_text(&subtask.id));
    if !valid {
        return None;
    }
    // `write_card` ends non-empty notes with a newline; editors add one too.
    let notes = body.strip_suffix('\n').unwrap_or(body).to_owned();
    Some(Card { meta, notes })
}

pub fn write_card(card: &Card) -> Result<String> {
    let front = serde_yaml_ng::to_string(&card.meta)
        .map_err(|_| AppError::Internal("cannot serialise card"))?;
    let newline = if card.notes.is_empty() { "" } else { "\n" };
    Ok(format!("---\n{front}---\n{}{newline}", card.notes))
}

pub fn write_json<T: Serialize>(value: &T) -> Result<String> {
    let mut text =
        serde_json::to_string_pretty(value).map_err(|_| AppError::Internal("cannot serialise"))?;
    text.push('\n');
    Ok(text)
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(title: &str, notes: &str) -> Card {
        Card {
            meta: CardMeta {
                id: "01JB7Q4ZK3M8X2V5T9R6N0PWYH".into(),
                column: "01JB7Q4B00000000000000000A".into(),
                position: "V".into(),
                title: title.into(),
                priority: Priority::High,
                tags: vec!["01JB7Q3A00000000000000000A".into()],
                subtasks: vec![SubtaskEntry {
                    id: "01JB7Q5A00000000000000000A".into(),
                    title: "Belege sammeln".into(),
                    done: true,
                    extra: YamlExtra::new(),
                }],
                created_at: Timestamp(1_792_746_845_123),
                updated_at: Timestamp(1_792_754_560_010),
                archived_at: None,
                extra: YamlExtra::new(),
            },
            notes: notes.into(),
        }
    }

    #[test]
    fn card_survives_a_round_trip() {
        for notes in [
            "",
            "eine Zeile",
            "zwei\n\nAbsätze\n",
            "\n",
            "---\nkein Frontmatter",
        ] {
            let original = card("Steuererklärung 2025", notes);
            let text = write_card(&original).unwrap();
            assert_eq!(parse_card(&text), Some(original), "{notes:?}");
        }
    }

    #[test]
    fn titles_that_look_like_other_yaml_types_stay_text() {
        for title in [
            "no", "true", "1", "1.5", "null", "~", "- a", "a: b", "#tag", "'x'", "\"y\"",
        ] {
            let original = card(title, "");
            assert_eq!(
                parse_card(&write_card(&original).unwrap()),
                Some(original),
                "{title}"
            );
        }
    }

    #[test]
    fn writes_readable_frontmatter() {
        let text = write_card(&card("Titel", "Notiz")).unwrap();
        assert!(
            text.starts_with("---\nid: 01JB7Q4ZK3M8X2V5T9R6N0PWYH\n"),
            "{text}"
        );
        assert!(text.contains("priority: high\n"), "{text}");
        assert!(
            text.contains("createdAt: 2026-10-23T09:14:05.123Z\n"),
            "{text}"
        );
        assert!(!text.contains("archivedAt"), "{text}");
        assert!(text.ends_with("---\nNotiz\n"), "{text}");
    }

    #[test]
    fn reads_hand_written_card_with_defaults() {
        let text = "\u{feff}---\r\nid: abc\r\ncolumn: col\r\nposition: V\r\ntitle: Einkaufen\r\n\
                    createdAt: 2026-10-23T09:14:05Z\r\nupdatedAt: 2026-10-23T09:14:05Z\r\n---\r\n\
                    Milch\r\n";
        let card = parse_card(text).unwrap();
        assert_eq!(card.meta.priority, Priority::None);
        assert!(card.meta.tags.is_empty() && card.meta.subtasks.is_empty());
        assert_eq!(card.meta.archived_at, None);
        assert_eq!(card.notes, "Milch");
    }

    #[test]
    fn keeps_unknown_fields() {
        let text = "---\nid: abc\ncolumn: col\nposition: V\ntitle: T\n\
                    createdAt: 2026-10-23T09:14:05.000Z\nupdatedAt: 2026-10-23T09:14:05.000Z\n\
                    dueDate: 2026-11-01\nsubtasks:\n  - id: s1\n    title: A\n    note: x\n---\n";
        let card = parse_card(text).unwrap();
        assert!(card.meta.extra.contains_key("dueDate"));
        assert!(card.meta.subtasks[0].extra.contains_key("note"));
        let again = parse_card(&write_card(&card).unwrap()).unwrap();
        assert_eq!(again, card);
    }

    #[test]
    fn rejects_invalid_cards() {
        let valid = "id: a\ncolumn: c\nposition: V\ntitle: T\n\
                     createdAt: 2026-10-23T09:14:05Z\nupdatedAt: 2026-10-23T09:14:05Z\n";
        assert!(parse_card(&format!("---\n{valid}---\n")).is_some());
        // Plain YAML numbers are read as text, as people may write them by hand.
        let numeric = valid.replace("position: V", "position: 1");
        assert_eq!(
            parse_card(&format!("---\n{numeric}---\n"))
                .unwrap()
                .meta
                .position,
            "1"
        );
        for text in [
            String::new(),
            "Nur Notizen".into(),
            format!("---\n{valid}"),
            format!("---\n{}---\n", valid.replace("title: T", "title: '  '")),
            format!("---\n{}---\n", valid.replace("position: V", "position: V0")),
            format!(
                "---\n{}---\n",
                valid.replace("09:14:05Z\nupdated", "gestern\nupdated")
            ),
            format!("---\n{valid}priority: urgent\n---\n"),
            format!("---\n{valid}: kaputt\n---\n"),
        ] {
            assert_eq!(parse_card(&text), None, "{text}");
        }
    }

    fn column(id: &str, position: &str, is_done: bool) -> ColumnEntry {
        ColumnEntry {
            id: id.into(),
            name: id.to_uppercase(),
            position: position.into(),
            is_done,
            extra: JsonExtra::new(),
        }
    }

    #[test]
    fn board_survives_a_round_trip_with_unknown_fields() {
        let text = r#"{
            "id": "b", "name": "Arbeit", "position": "V", "createdAt": "2026-10-23T09:10:00.000Z",
            "color": "teal",
            "columns": [
                { "id": "a", "name": "Offen", "position": "V", "wipLimit": 3 },
                { "id": "c", "name": "Erledigt", "position": "k", "isDone": true }
            ]
        }"#;
        let board = parse_board(text).unwrap();
        assert_eq!(board.extra["color"], "teal");
        assert_eq!(board.columns[0].extra["wipLimit"], 3);
        assert_eq!(parse_board(&write_json(&board).unwrap()), Some(board));
    }

    #[test]
    fn board_columns_are_sorted_with_one_done_column() {
        let mut board = BoardFile {
            id: "b".into(),
            name: "B".into(),
            position: "V".into(),
            created_at: Timestamp(0),
            columns: vec![column("c", "k", false), column("a", "V", false)],
            extra: JsonExtra::new(),
        };
        board.normalize();
        let flags: Vec<_> = board
            .columns
            .iter()
            .map(|c| (c.id.as_str(), c.is_done))
            .collect();
        assert_eq!(flags, [("a", false), ("c", true)]);

        board.columns = vec![
            column("a", "V", true),
            column("b", "d", true),
            column("c", "k", false),
        ];
        board.normalize();
        let flags: Vec<_> = board
            .columns
            .iter()
            .map(|c| (c.id.as_str(), c.is_done))
            .collect();
        assert_eq!(flags, [("a", false), ("b", true), ("c", false)]);
    }

    #[test]
    fn rejects_invalid_boards_and_tags() {
        assert!(parse_board("{}").is_none());
        assert!(
            parse_board(
                r#"{ "id": "b", "name": "", "position": "V", "createdAt": "2026-10-23T09:10:00Z" }"#
            )
            .is_none()
        );
        assert!(
            parse_tags(r#"{ "tags": [{ "id": "t", "name": " ", "color": "red" }] }"#).is_none()
        );
        assert_eq!(parse_tags("{}"), Some(TagsFile::default()));
        assert!(parse_vault(r#"{ "formatVersion": 1, "id": "" }"#).is_none());
    }
}
