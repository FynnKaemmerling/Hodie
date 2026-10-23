//! Writes boards from elsewhere (the SQLite database of earlier versions) into
//! a vault, next to whatever it already holds.

use std::collections::HashMap;
use std::fs;

use super::Vault;
use super::format::{BoardFile, Card, TagEntry};
use crate::error::Result;

#[derive(Debug, Default)]
pub struct Import {
    pub tags: Vec<TagEntry>,
    pub boards: Vec<(BoardFile, Vec<Card>)>,
}

/// Tags whose name already exists in the vault (ignoring case) are merged
/// into that tag. If anything fails, the boards written so far are removed
/// again, so a retry does not create them twice.
pub fn import(vault: &Vault, import: Import) -> Result<()> {
    let mut tags = vault.load()?.tags;
    let mut tag_ids: HashMap<String, String> = HashMap::new();
    for tag in import.tags {
        let name = tag.name.to_lowercase();
        match tags.tags.iter().find(|t| t.name.to_lowercase() == name) {
            Some(existing) => tag_ids.insert(tag.id, existing.id.clone()),
            None => {
                tag_ids.insert(tag.id.clone(), tag.id.clone());
                tags.tags.push(tag);
                None
            }
        };
    }

    let mut created = Vec::new();
    let mut write = || {
        for (board, cards) in &import.boards {
            let dir = vault.create_board(board)?;
            created.push(dir.clone());
            for card in cards {
                let mut card = card.clone();
                for tag in &mut card.meta.tags {
                    if let Some(id) = tag_ids.get(tag) {
                        tag.clone_from(id);
                    }
                }
                vault.write_card(&dir, &card)?;
            }
        }
        vault.write_tags(&tags)
    };
    let result = write();
    if result.is_err() {
        for dir in &created {
            let _ = fs::remove_dir_all(vault.root().join(dir));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Priority;
    use crate::vault::format::{CardMeta, ColumnEntry, TagsFile, Timestamp};
    use crate::vault::new_id;
    use crate::vault::tests::TestDir;

    fn tag(name: &str) -> TagEntry {
        TagEntry {
            id: new_id(),
            name: name.into(),
            color: "red".into(),
            extra: Default::default(),
        }
    }

    fn board_with_card(name: &str, tag_ids: Vec<String>) -> (BoardFile, Vec<Card>) {
        let column = ColumnEntry {
            id: new_id(),
            name: "Offen".into(),
            position: "V".into(),
            is_done: true,
            extra: Default::default(),
        };
        let card = Card {
            meta: CardMeta {
                id: new_id(),
                column: column.id.clone(),
                position: "V".into(),
                title: format!("Karte in {name}"),
                priority: Priority::None,
                tags: tag_ids,
                subtasks: Vec::new(),
                created_at: Timestamp(0),
                updated_at: Timestamp(0),
                archived_at: None,
                extra: Default::default(),
            },
            notes: String::new(),
        };
        let board = BoardFile {
            id: new_id(),
            name: name.into(),
            position: "V".into(),
            created_at: Timestamp(0),
            columns: vec![column],
            extra: Default::default(),
        };
        (board, vec![card])
    }

    #[test]
    fn imports_boards_and_merges_tags_by_name() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        let existing = tag("Steuern");
        vault
            .write_tags(&TagsFile {
                tags: vec![existing.clone()],
                extra: Default::default(),
            })
            .unwrap();

        let same = tag("STEUERN");
        let new = tag("Haus");
        let import_data = Import {
            tags: vec![same.clone(), new.clone()],
            boards: vec![board_with_card("Alt", vec![same.id, new.id.clone()])],
        };
        import(&vault, import_data).unwrap();

        let data = vault.load().unwrap();
        let names: Vec<&str> = data.tags.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Steuern", "Haus"]);
        assert_eq!(data.boards[0].board.name, "Alt");
        assert_eq!(
            data.boards[0].cards[0].card.meta.tags,
            [existing.id, new.id]
        );
    }

    #[test]
    fn a_failed_import_leaves_no_boards_behind() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        fs::write(dir.path().join(".hodie/tags.json"), "kaputt").unwrap();
        let import_data = Import {
            tags: vec![tag("T")],
            boards: vec![board_with_card("Alt", Vec::new())],
        };
        assert_eq!(
            import(&vault, import_data).unwrap_err().code(),
            "file.invalid"
        );
        assert!(vault.load().unwrap().boards.is_empty());
    }
}
