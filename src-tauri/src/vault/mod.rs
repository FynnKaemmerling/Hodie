//! Vault storage: boards, columns, cards and tags as plain files in a folder
//! the user picks, so it can be shared through any sync service. The format
//! is specified in `VAULT.md`. Knows nothing about Tauri or the UI.

pub mod files;
pub mod format;
pub mod time;

use std::fs;
use std::path::{Path, PathBuf};

use ulid::Ulid;

use crate::error::{AppError, Result};
use files::{remove_temp_files, visible_entries, write_atomic};
use format::{BoardFile, Card, TagsFile, VaultFile};

/// Highest `formatVersion` this app reads and writes.
pub const FORMAT_VERSION: u32 = 1;

const META_DIR: &str = ".hodie";
const VAULT_FILE: &str = "vault.json";
const TAGS_FILE: &str = "tags.json";
const BOARD_FILE: &str = "board.json";
const CARDS_DIR: &str = "cards";

pub fn new_id() -> String {
    Ulid::generate().to_string()
}

#[derive(Debug)]
pub struct Vault {
    root: PathBuf,
    id: String,
}

/// Everything in a vault, read into memory.
#[derive(Debug, Default)]
pub struct VaultData {
    pub tags: TagsFile,
    /// Sorted by position.
    pub boards: Vec<LoadedBoard>,
    pub problems: Vec<Problem>,
}

#[derive(Debug)]
pub struct LoadedBoard {
    /// Folder name inside the vault.
    pub dir: String,
    pub board: BoardFile,
    /// Sorted by position; archived ones included.
    pub cards: Vec<LoadedCard>,
}

#[derive(Debug)]
pub struct LoadedCard {
    /// File name inside `cards/`. Usually `<id>.md`, but a card written by
    /// hand may be named differently.
    pub file: String,
    pub card: Card,
}

/// Files that were skipped while loading. Paths are relative to the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Could not be read or parsed; left untouched.
    Invalid { path: PathBuf },
    /// A second file for the same board, card or tag list, typically a sync
    /// service's conflict copy. `original` is the file that was loaded.
    Conflict { path: PathBuf, original: PathBuf },
}

impl Vault {
    /// Opens the vault in `root`, or turns `root` into a new vault if it holds
    /// nothing but hidden files.
    pub fn open(root: &Path) -> Result<Vault> {
        if !root.is_dir() {
            return Err(AppError::Vault("vault.not_found"));
        }
        let meta_dir = root.join(META_DIR);
        let vault_path = meta_dir.join(VAULT_FILE);
        let file = if vault_path.is_file() {
            let file = format::parse_vault(&fs::read_to_string(&vault_path)?)
                .ok_or(AppError::Vault("file.invalid"))?;
            if file.format_version > FORMAT_VERSION {
                return Err(AppError::Vault("vault.too_new"));
            }
            file
        } else if visible_entries(root)?.is_empty() {
            let file = VaultFile {
                format_version: FORMAT_VERSION,
                id: new_id(),
                extra: Default::default(),
            };
            fs::create_dir_all(&meta_dir)?;
            write_atomic(&vault_path, &format::write_json(&file)?)?;
            file
        } else {
            return Err(AppError::Vault("vault.not_a_vault"));
        };

        let vault = Vault {
            root: root.to_owned(),
            id: file.id,
        };
        vault.remove_temp_files()?;
        Ok(vault)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn remove_temp_files(&self) -> Result<()> {
        remove_temp_files(&self.root.join(META_DIR))?;
        for entry in visible_entries(&self.root)? {
            if entry.is_dir && entry.path.join(BOARD_FILE).is_file() {
                remove_temp_files(&entry.path)?;
                let cards = entry.path.join(CARDS_DIR);
                if cards.is_dir() {
                    remove_temp_files(&cards)?;
                }
            }
        }
        Ok(())
    }

    pub fn load(&self) -> Result<VaultData> {
        let mut data = VaultData::default();
        self.load_tags(&mut data)?;

        let mut seen: Vec<(String, PathBuf)> = Vec::new();
        for entry in visible_entries(&self.root)? {
            let board_path = entry.path.join(BOARD_FILE);
            if !entry.is_dir || !board_path.is_file() {
                continue;
            }
            let relative = Path::new(&entry.name).join(BOARD_FILE);
            let Some(board) = read(&board_path, format::parse_board) else {
                data.problems.push(Problem::Invalid { path: relative });
                continue;
            };
            if let Some((_, original)) = seen.iter().find(|(id, _)| *id == board.id) {
                data.problems.push(Problem::Conflict {
                    path: relative,
                    original: original.clone(),
                });
                continue;
            }
            seen.push((board.id.clone(), relative.clone()));

            // Conflict copies of board.json, e.g. "board (conflicted copy).json".
            for other in visible_entries(&entry.path)? {
                if other.is_dir || other.name == BOARD_FILE || !other.name.ends_with(".json") {
                    continue;
                }
                if read(&other.path, format::parse_board).is_some_and(|copy| copy.id == board.id) {
                    data.problems.push(Problem::Conflict {
                        path: Path::new(&entry.name).join(&other.name),
                        original: relative.clone(),
                    });
                }
            }

            let cards = self.load_cards(&entry.name, &mut data.problems)?;
            data.boards.push(LoadedBoard {
                dir: entry.name,
                board,
                cards,
            });
        }
        data.boards.sort_by(|a, b| {
            (&a.board.position, &a.board.id).cmp(&(&b.board.position, &b.board.id))
        });
        Ok(data)
    }

    fn load_tags(&self, data: &mut VaultData) -> Result<()> {
        let meta_dir = self.root.join(META_DIR);
        let relative = Path::new(META_DIR).join(TAGS_FILE);
        let path = meta_dir.join(TAGS_FILE);
        if path.is_file() {
            match read(&path, format::parse_tags) {
                Some(tags) => data.tags = tags,
                None => data.problems.push(Problem::Invalid {
                    path: relative.clone(),
                }),
            }
        }
        for entry in visible_entries(&meta_dir)? {
            let is_copy = !entry.is_dir
                && entry.name != TAGS_FILE
                && entry.name != VAULT_FILE
                && entry.name.ends_with(".json")
                && read(&entry.path, format::parse_tags).is_some();
            if is_copy {
                data.problems.push(Problem::Conflict {
                    path: Path::new(META_DIR).join(&entry.name),
                    original: relative.clone(),
                });
            }
        }
        Ok(())
    }

    fn load_cards(&self, dir: &str, problems: &mut Vec<Problem>) -> Result<Vec<LoadedCard>> {
        let cards_dir = self.root.join(dir).join(CARDS_DIR);
        if !cards_dir.is_dir() {
            return Ok(Vec::new());
        }
        let relative = |file: &str| Path::new(dir).join(CARDS_DIR).join(file);

        let mut cards: Vec<LoadedCard> = Vec::new();
        for entry in visible_entries(&cards_dir)? {
            if entry.is_dir || !entry.name.ends_with(".md") {
                continue;
            }
            match read(&entry.path, format::parse_card) {
                Some(card) => cards.push(LoadedCard {
                    file: entry.name,
                    card,
                }),
                None => problems.push(Problem::Invalid {
                    path: relative(&entry.name),
                }),
            }
        }

        // Several files with the same id: keep `<id>.md`, else the first by name.
        let mut kept: Vec<LoadedCard> = Vec::with_capacity(cards.len());
        let canonical_first = |card: &LoadedCard| card.file != card_file_name(&card.card.meta.id);
        cards.sort_by(|a, b| {
            (&a.card.meta.id, canonical_first(a), &a.file).cmp(&(
                &b.card.meta.id,
                canonical_first(b),
                &b.file,
            ))
        });
        for card in cards {
            match kept.last() {
                Some(last) if last.card.meta.id == card.card.meta.id => {
                    problems.push(Problem::Conflict {
                        path: relative(&card.file),
                        original: relative(&last.file),
                    });
                }
                _ => kept.push(card),
            }
        }
        kept.sort_by(|a, b| {
            let key =
                |card: &LoadedCard| (card.card.meta.position.clone(), card.card.meta.id.clone());
            key(a).cmp(&key(b))
        });
        Ok(kept)
    }

    pub fn write_tags(&self, tags: &TagsFile) -> Result<()> {
        let path = self.root.join(META_DIR).join(TAGS_FILE);
        replace(&path, &format::write_json(tags)?, format::parse_tags)
    }

    /// Creates the folder for a new board and returns its name.
    pub fn create_board(&self, board: &BoardFile) -> Result<String> {
        let taken: Vec<String> = fs::read_dir(&self.root)?
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect();
        let dir = files::board_dir_name(&board.name, &taken);
        fs::create_dir_all(self.root.join(&dir).join(CARDS_DIR))?;
        self.write_board(&dir, board)?;
        Ok(dir)
    }

    pub fn write_board(&self, dir: &str, board: &BoardFile) -> Result<()> {
        let path = self.root.join(dir).join(BOARD_FILE);
        replace(&path, &format::write_json(board)?, format::parse_board)
    }

    /// Removes the board's folder with all its cards.
    pub fn remove_board(&self, dir: &str) -> Result<()> {
        Ok(fs::remove_dir_all(self.root.join(dir))?)
    }

    /// Writes the card to `cards/<id>.md`. If it was loaded from a file with
    /// another name, the caller removes that one with `remove_file`.
    pub fn write_card(&self, dir: &str, card: &Card) -> Result<()> {
        let cards_dir = self.root.join(dir).join(CARDS_DIR);
        fs::create_dir_all(&cards_dir)?;
        let path = cards_dir.join(card_file_name(&card.meta.id));
        replace(&path, &format::write_card(card)?, format::parse_card)
    }

    pub fn remove_card(&self, dir: &str, file: &str) -> Result<()> {
        self.remove_file(&Path::new(dir).join(CARDS_DIR).join(file))
    }

    /// Removes a file given relative to the vault, e.g. a conflict copy.
    pub fn remove_file(&self, relative: &Path) -> Result<()> {
        Ok(fs::remove_file(self.root.join(relative))?)
    }
}

pub fn card_file_name(id: &str) -> String {
    format!("{id}.md")
}

fn read<T>(path: &Path, parse: fn(&str) -> Option<T>) -> Option<T> {
    parse(&fs::read_to_string(path).ok()?)
}

/// Writes `content` to `path`, but refuses to overwrite a file this app
/// cannot read: it may hold someone's data in a form we do not understand.
fn replace<T>(path: &Path, content: &str, parse: fn(&str) -> Option<T>) -> Result<()> {
    if path.exists() && read(path, parse).is_none() {
        return Err(AppError::Vault("file.invalid"));
    }
    write_atomic(path, content)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;
    use crate::model::Priority;
    use format::{CardMeta, ColumnEntry, TagEntry, Timestamp};

    /// A fresh directory under the system temp dir, removed on drop.
    pub struct TestDir(PathBuf);

    impl TestDir {
        pub fn new() -> TestDir {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let path = std::env::temp_dir().join(format!(
                "hodie-vault-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            TestDir(path)
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn board(name: &str, position: &str) -> BoardFile {
        BoardFile {
            id: new_id(),
            name: name.into(),
            position: position.into(),
            created_at: Timestamp(1_792_746_600_000),
            columns: vec![
                ColumnEntry {
                    id: new_id(),
                    name: "Offen".into(),
                    position: "V".into(),
                    is_done: false,
                    extra: Default::default(),
                },
                ColumnEntry {
                    id: new_id(),
                    name: "Erledigt".into(),
                    position: "k".into(),
                    is_done: true,
                    extra: Default::default(),
                },
            ],
            extra: Default::default(),
        }
    }

    fn card(board: &BoardFile, title: &str, position: &str) -> Card {
        Card {
            meta: CardMeta {
                id: new_id(),
                column: board.columns[0].id.clone(),
                position: position.into(),
                title: title.into(),
                priority: Priority::Medium,
                tags: Vec::new(),
                subtasks: Vec::new(),
                created_at: Timestamp(1_792_746_845_123),
                updated_at: Timestamp(1_792_746_845_123),
                archived_at: None,
                extra: Default::default(),
            },
            notes: "Notiz".into(),
        }
    }

    fn titles(board: &LoadedBoard) -> Vec<&str> {
        board
            .cards
            .iter()
            .map(|c| c.card.meta.title.as_str())
            .collect()
    }

    #[test]
    fn open_turns_an_empty_folder_into_a_vault() {
        let dir = TestDir::new();
        fs::write(dir.path().join(".DS_Store"), "").unwrap();
        let vault = Vault::open(dir.path()).unwrap();
        assert!(dir.path().join(".hodie/vault.json").is_file());

        let again = Vault::open(dir.path()).unwrap();
        assert_eq!(again.id(), vault.id());
        let data = again.load().unwrap();
        assert!(data.boards.is_empty() && data.problems.is_empty());
        assert!(data.tags.tags.is_empty());
    }

    #[test]
    fn open_refuses_missing_foreign_and_newer_folders() {
        let dir = TestDir::new();
        let code = |result: Result<Vault>| result.unwrap_err().code();

        assert_eq!(
            code(Vault::open(&dir.path().join("fehlt"))),
            "vault.not_found"
        );

        fs::write(dir.path().join("Steuern.pdf"), "").unwrap();
        assert_eq!(code(Vault::open(dir.path())), "vault.not_a_vault");

        fs::create_dir(dir.path().join(".hodie")).unwrap();
        fs::write(
            dir.path().join(".hodie/vault.json"),
            format!(
                r#"{{ "formatVersion": {}, "id": "v" }}"#,
                FORMAT_VERSION + 1
            ),
        )
        .unwrap();
        assert_eq!(code(Vault::open(dir.path())), "vault.too_new");

        fs::write(dir.path().join(".hodie/vault.json"), "kaputt").unwrap();
        assert_eq!(code(Vault::open(dir.path())), "file.invalid");
    }

    #[test]
    fn boards_cards_and_tags_survive_reopening() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();

        let work = board("Arbeit", "k");
        let home = board("Privat", "V");
        let work_dir = vault.create_board(&work).unwrap();
        let home_dir = vault.create_board(&home).unwrap();
        assert_eq!((work_dir.as_str(), home_dir.as_str()), ("Arbeit", "Privat"));

        let second = card(&work, "Zweite", "k");
        let first = card(&work, "Erste", "V");
        vault.write_card(&work_dir, &second).unwrap();
        vault.write_card(&work_dir, &first).unwrap();
        let tags = TagsFile {
            tags: vec![TagEntry {
                id: new_id(),
                name: "Steuern".into(),
                color: "orange".into(),
                extra: Default::default(),
            }],
            extra: Default::default(),
        };
        vault.write_tags(&tags).unwrap();

        let data = Vault::open(dir.path()).unwrap().load().unwrap();
        assert!(data.problems.is_empty(), "{:?}", data.problems);
        assert_eq!(data.tags, tags);
        let names: Vec<_> = data.boards.iter().map(|b| b.board.name.as_str()).collect();
        assert_eq!(names, ["Privat", "Arbeit"]);
        assert_eq!(data.boards[1].board, work);
        assert_eq!(titles(&data.boards[1]), ["Erste", "Zweite"]);
        assert_eq!(data.boards[1].cards[0].card, first);
        assert_eq!(data.boards[1].cards[0].file, card_file_name(&first.meta.id));
    }

    #[test]
    fn boards_with_the_same_name_get_separate_folders() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        assert_eq!(vault.create_board(&board("Arbeit", "V")).unwrap(), "Arbeit");
        assert_eq!(
            vault.create_board(&board("ARBEIT", "k")).unwrap(),
            "ARBEIT 2"
        );
    }

    #[test]
    fn invalid_files_are_reported_and_never_overwritten() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        let work = board("Arbeit", "V");
        let work_dir = vault.create_board(&work).unwrap();
        let broken = card(&work, "Kaputt", "V");
        let broken_path = dir
            .path()
            .join("Arbeit/cards")
            .join(card_file_name(&broken.meta.id));
        fs::write(&broken_path, "---\ntitle: [\n---\n").unwrap();
        fs::write(dir.path().join(".hodie/tags.json"), "{").unwrap();
        fs::create_dir(dir.path().join("Kein Board")).unwrap();
        fs::write(dir.path().join("Arbeit/cards/notizen.txt"), "").unwrap();

        let data = vault.load().unwrap();
        assert_eq!(
            data.problems,
            [
                Problem::Invalid {
                    path: Path::new(".hodie").join("tags.json")
                },
                Problem::Invalid {
                    path: Path::new("Arbeit/cards").join(card_file_name(&broken.meta.id))
                },
            ]
        );
        assert!(data.boards[0].cards.is_empty());

        assert_eq!(
            vault.write_card(&work_dir, &broken).unwrap_err().code(),
            "file.invalid"
        );
        assert_eq!(
            vault.write_tags(&TagsFile::default()).unwrap_err().code(),
            "file.invalid"
        );
        assert_eq!(
            fs::read_to_string(&broken_path).unwrap(),
            "---\ntitle: [\n---\n"
        );
    }

    #[test]
    fn conflict_copies_are_reported_and_the_original_is_kept() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        let work = board("Arbeit", "V");
        let work_dir = vault.create_board(&work).unwrap();
        let original = card(&work, "Original", "V");
        vault.write_card(&work_dir, &original).unwrap();
        vault.write_tags(&TagsFile::default()).unwrap();

        let cards = dir.path().join("Arbeit/cards");
        let mut copy = original.clone();
        copy.meta.title = "Kopie".into();
        // Sorts before the original's name, so the name decides, not the order.
        fs::write(
            cards.join("0 (Konfliktkopie).md"),
            format::write_card(&copy).unwrap(),
        )
        .unwrap();
        fs::copy(
            dir.path().join("Arbeit/board.json"),
            dir.path().join("Arbeit/board 2.json"),
        )
        .unwrap();
        fs::copy(
            dir.path().join(".hodie/tags.json"),
            dir.path().join(".hodie/tags.sync-conflict-20261023.json"),
        )
        .unwrap();
        // A whole board folder duplicated by a sync client.
        fs::create_dir_all(dir.path().join("Arbeit (1)/cards")).unwrap();
        fs::copy(
            dir.path().join("Arbeit/board.json"),
            dir.path().join("Arbeit (1)/board.json"),
        )
        .unwrap();

        let data = vault.load().unwrap();
        assert_eq!(data.boards.len(), 1);
        assert_eq!(titles(&data.boards[0]), ["Original"]);
        let board_json = Path::new("Arbeit").join("board.json");
        let card_file = Path::new("Arbeit/cards").join(card_file_name(&original.meta.id));
        assert_eq!(
            data.problems,
            [
                Problem::Conflict {
                    path: Path::new(".hodie").join("tags.sync-conflict-20261023.json"),
                    original: Path::new(".hodie").join("tags.json"),
                },
                Problem::Conflict {
                    path: Path::new("Arbeit").join("board 2.json"),
                    original: board_json.clone(),
                },
                Problem::Conflict {
                    path: Path::new("Arbeit/cards").join("0 (Konfliktkopie).md"),
                    original: card_file,
                },
                Problem::Conflict {
                    path: Path::new("Arbeit (1)").join("board.json"),
                    original: board_json,
                },
            ]
        );

        vault
            .remove_file(&Path::new("Arbeit/cards").join("0 (Konfliktkopie).md"))
            .unwrap();
        assert_eq!(vault.load().unwrap().problems.len(), 3);
    }

    #[test]
    fn a_hand_written_card_keeps_its_file_name() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        let work = board("Arbeit", "V");
        let work_dir = vault.create_board(&work).unwrap();
        let mut hand = card(&work, "Von Hand", "V");
        hand.meta.id = "einkauf".into();
        fs::write(
            dir.path().join("Arbeit/cards/Einkaufen.md"),
            format::write_card(&hand).unwrap(),
        )
        .unwrap();

        let data = vault.load().unwrap();
        assert!(data.problems.is_empty());
        assert_eq!(data.boards[0].cards[0].file, "Einkaufen.md");

        vault.write_card(&work_dir, &hand).unwrap();
        vault.remove_card(&work_dir, "Einkaufen.md").unwrap();
        let data = vault.load().unwrap();
        assert_eq!(data.boards[0].cards[0].file, "einkauf.md");
    }

    #[test]
    fn removing_cards_and_boards_deletes_their_files() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        let work = board("Arbeit", "V");
        let work_dir = vault.create_board(&work).unwrap();
        let task = card(&work, "Weg", "V");
        vault.write_card(&work_dir, &task).unwrap();

        vault
            .remove_card(&work_dir, &card_file_name(&task.meta.id))
            .unwrap();
        assert!(vault.load().unwrap().boards[0].cards.is_empty());

        vault.remove_board(&work_dir).unwrap();
        assert!(vault.load().unwrap().boards.is_empty());
        assert!(!dir.path().join("Arbeit").exists());
    }

    #[test]
    fn open_removes_interrupted_writes() {
        let dir = TestDir::new();
        let vault = Vault::open(dir.path()).unwrap();
        vault.create_board(&board("Arbeit", "V")).unwrap();
        let leftovers = [
            dir.path().join(".hodie/.tags.json.tmp"),
            dir.path().join("Arbeit/.board.json.tmp"),
            dir.path().join("Arbeit/cards/.x.md.tmp"),
        ];
        for path in &leftovers {
            fs::write(path, "halb").unwrap();
        }
        Vault::open(dir.path()).unwrap();
        assert!(leftovers.iter().all(|path| !path.exists()));
    }
}
