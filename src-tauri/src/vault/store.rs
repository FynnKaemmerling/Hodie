//! The open vault in memory. Answers the commands from the loaded files and
//! writes each change straight back, only to the files it touches.
//!
//! Deleted boards, columns and cards are hidden on disk (see `trash_*` in the
//! parent module) and kept here, so they can be restored until the app quits.

use super::format::{
    BoardFile, Card, CardMeta, ColumnEntry, SubtaskEntry, TagEntry, TagsFile, Timestamp,
};
use std::path::{Path, PathBuf};

use super::{CARDS_DIR, LoadedBoard, LoadedCard, Problem, Vault, card_file_name, new_id};
use crate::error::{AppError, Result};
use crate::model::{
    Board, BoardView, CardDetail, CardPatch, CardSummary, CardVersion, Column, ColumnView, Id,
    NewCard, Subtask, SubtaskPatch, TAG_COLORS, Tag, TagPatch, VaultProblem,
};
use crate::position;
use crate::store::{now_ms, require_text};

pub struct VaultStore {
    vault: Vault,
    tags: TagsFile,
    /// Sorted by position.
    boards: Vec<LoadedBoard>,
    /// Files skipped while loading: unreadable ones and conflict copies.
    problems: Vec<Problem>,
    trash: Trash,
}

#[derive(Default)]
struct Trash {
    boards: Vec<TrashedBoard>,
    columns: Vec<TrashedColumn>,
    cards: Vec<TrashedCard>,
}

struct TrashedBoard {
    board: LoadedBoard,
    /// Hidden folder name in the vault.
    trash: String,
}

struct TrashedColumn {
    board_id: Id,
    column: ColumnEntry,
    cards: Vec<LoadedCard>,
}

struct TrashedCard {
    board_id: Id,
    card: LoadedCard,
}

fn now() -> Timestamp {
    Timestamp(now_ms())
}

fn require_color(color: &str) -> Result<&str> {
    if TAG_COLORS.contains(&color) {
        Ok(color)
    } else {
        Err(AppError::Validation("tag.invalid_color"))
    }
}

/// The column a card shows in. A card whose column is gone (deleted by
/// someone else) shows in the first column; its file is fixed on next write.
fn column_of<'a>(board: &'a BoardFile, card: &'a CardMeta) -> &'a str {
    if board.columns.iter().any(|column| column.id == card.column) {
        &card.column
    } else {
        board
            .columns
            .first()
            .map_or(&card.column, |column| &column.id)
    }
}

fn subtask_index(card: &Card, id: &str) -> Result<usize> {
    card.meta
        .subtasks
        .iter()
        .position(|subtask| subtask.id == id)
        .ok_or(AppError::NotFound)
}

fn by_position<'a>(cards: impl Iterator<Item = &'a CardMeta>) -> Vec<&'a CardMeta> {
    let mut cards: Vec<_> = cards.collect();
    cards.sort_by(|a, b| (&a.position, &a.id).cmp(&(&b.position, &b.id)));
    cards
}

/// Position keys of the active cards in a column, except `except`.
fn active_keys(board: &LoadedBoard, column_id: &str, except: &str) -> Vec<String> {
    by_position(
        board
            .cards
            .iter()
            .map(|loaded| &loaded.card.meta)
            .filter(|meta| {
                meta.archived_at.is_none()
                    && meta.id != except
                    && column_of(&board.board, meta) == column_id
            }),
    )
    .into_iter()
    .map(|meta| meta.position.clone())
    .collect()
}

impl VaultStore {
    pub fn load(vault: Vault) -> Result<VaultStore> {
        let data = vault.load()?;
        Ok(VaultStore {
            vault,
            tags: data.tags,
            boards: data.boards,
            problems: data.problems,
            trash: Trash::default(),
        })
    }

    /// Reads the files again after they changed on disk. Returns whether
    /// anything differs from what was loaded, so a change this store wrote
    /// itself does not count.
    pub fn refresh(&mut self) -> Result<bool> {
        let data = self.vault.load()?;
        let changed =
            data.tags != self.tags || data.boards != self.boards || data.problems != self.problems;
        self.tags = data.tags;
        self.boards = data.boards;
        self.problems = data.problems;
        Ok(changed)
    }

    pub fn vault(&self) -> &Vault {
        &self.vault
    }

    fn board_index(&self, id: &str) -> Result<usize> {
        self.boards
            .iter()
            .position(|board| board.board.id == id)
            .ok_or(AppError::NotFound)
    }

    /// Board and column index of a column.
    fn column_location(&self, id: &str) -> Result<(usize, usize)> {
        self.boards
            .iter()
            .enumerate()
            .find_map(|(b, board)| {
                let c = board
                    .board
                    .columns
                    .iter()
                    .position(|column| column.id == id)?;
                Some((b, c))
            })
            .ok_or(AppError::NotFound)
    }

    /// Board and card index of a card.
    fn card_location(&self, id: &str) -> Result<(usize, usize)> {
        self.boards
            .iter()
            .enumerate()
            .find_map(|(b, board)| {
                let k = board
                    .cards
                    .iter()
                    .position(|card| card.card.meta.id == id)?;
                Some((b, k))
            })
            .ok_or(AppError::NotFound)
    }

    /// Board, card and subtask index of a subtask.
    fn subtask_location(&self, id: &str) -> Result<(usize, usize, usize)> {
        self.boards
            .iter()
            .enumerate()
            .find_map(|(b, board)| {
                board.cards.iter().enumerate().find_map(|(k, card)| {
                    let s = card
                        .card
                        .meta
                        .subtasks
                        .iter()
                        .position(|subtask| subtask.id == id)?;
                    Some((b, k, s))
                })
            })
            .ok_or(AppError::NotFound)
    }

    fn card(&self, (b, k): (usize, usize)) -> &Card {
        &self.boards[b].cards[k].card
    }

    // Changes build on the files as they are on disk now, not on what was
    // loaded: someone else may have changed them meanwhile, and a change to
    // one field must not undo theirs to another.

    /// The board's file as it is now. A board that is gone is dropped.
    fn fresh_board(&mut self, b: usize) -> Result<BoardFile> {
        let dir = self.boards[b].dir.clone();
        match self.vault.read_board(&dir)? {
            Some(file) if file.id == self.boards[b].board.id => {
                self.boards[b].board = file.clone();
                Ok(file)
            }
            _ => {
                self.boards.remove(b);
                Err(AppError::NotFound)
            }
        }
    }

    /// Board index, its fresh file and the column's index in that file.
    fn fresh_column(&mut self, id: &str) -> Result<(usize, BoardFile, usize)> {
        let (b, _) = self.column_location(id)?;
        let file = self.fresh_board(b)?;
        let c = file
            .columns
            .iter()
            .position(|column| column.id == id)
            .ok_or(AppError::NotFound)?;
        Ok((b, file, c))
    }

    /// The card as it is now. A card whose file is gone is dropped.
    fn fresh_card(&mut self, (b, k): (usize, usize)) -> Result<Card> {
        let dir = self.boards[b].dir.clone();
        let file = self.boards[b].cards[k].file.clone();
        match self.vault.read_card(&dir, &file)? {
            Some(card) if card.meta.id == self.boards[b].cards[k].card.meta.id => {
                self.boards[b].cards[k].card = card.clone();
                Ok(card)
            }
            _ => {
                self.boards[b].cards.remove(k);
                Err(AppError::NotFound)
            }
        }
    }

    /// Like `fresh_card`, for loops over several cards: None if it is gone.
    fn fresh_card_by_id(&mut self, id: &str) -> Result<Option<Card>> {
        let Ok(at) = self.card_location(id) else {
            return Ok(None);
        };
        match self.fresh_card(at) {
            Ok(card) => Ok(Some(card)),
            Err(AppError::NotFound) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn fresh_tags(&mut self) -> Result<TagsFile> {
        self.tags = self.vault.read_tags()?;
        Ok(self.tags.clone())
    }

    fn sort_boards(&mut self) {
        self.boards.sort_by(|a, b| {
            (&a.board.position, &a.board.id).cmp(&(&b.board.position, &b.board.id))
        });
    }

    fn save_board(&mut self, b: usize, mut file: BoardFile) -> Result<()> {
        file.normalize();
        self.vault.write_board(&self.boards[b].dir, &file)?;
        self.boards[b].board = file;
        self.sort_boards();
        Ok(())
    }

    /// Writes the card and replaces the one in memory. A card whose column is
    /// gone is moved to the column it shows in.
    fn save_card(&mut self, (b, k): (usize, usize), mut card: Card) -> Result<()> {
        card.meta.column = column_of(&self.boards[b].board, &card.meta).to_owned();
        let dir = self.boards[b].dir.clone();
        self.vault.write_card(&dir, &card)?;
        let loaded = &mut self.boards[b].cards[k];
        let canonical = card_file_name(&card.meta.id);
        if loaded.file != canonical {
            self.vault.remove_card(&dir, &loaded.file)?;
            loaded.file = canonical;
        }
        loaded.card = card;
        Ok(())
    }

    fn tag(&self, id: &str) -> Option<&TagEntry> {
        self.tags.tags.iter().find(|tag| tag.id == id)
    }

    /// Ids of the tags that exist, sorted by name. Unknown ids (e.g. a tag list
    /// not yet synced) are left out here but stay in the file.
    fn known_tags(&self, ids: &[String]) -> Vec<Id> {
        let mut tags: Vec<&TagEntry> = self
            .tags
            .tags
            .iter()
            .filter(|tag| ids.contains(&tag.id))
            .collect();
        tags.sort_by_key(|tag| tag.name.to_lowercase());
        tags.into_iter().map(|tag| tag.id.clone()).collect()
    }

    /// Checks `tag_ids` and returns them, keeping the card's unknown ones.
    fn merge_tags(&self, kept_from: &[String], tag_ids: &[Id]) -> Result<Vec<String>> {
        if tag_ids.iter().any(|id| self.tag(id).is_none()) {
            return Err(AppError::NotFound);
        }
        let mut tags: Vec<String> = kept_from
            .iter()
            .filter(|id| self.tag(id).is_none())
            .cloned()
            .collect();
        for id in tag_ids {
            if !tags.contains(id) {
                tags.push(id.clone());
            }
        }
        Ok(tags)
    }

    fn summary(&self, board: &LoadedBoard, card: &Card) -> CardSummary {
        let meta = &card.meta;
        CardSummary {
            id: meta.id.clone(),
            column_id: column_of(&board.board, meta).to_owned(),
            title: meta.title.clone(),
            priority: meta.priority,
            has_notes: !card.notes.is_empty(),
            subtasks_done: meta.subtasks.iter().filter(|subtask| subtask.done).count() as u32,
            subtasks_total: meta.subtasks.len() as u32,
            tag_ids: self.known_tags(&meta.tags),
            has_conflict: !self.conflict_copies(board, &meta.id).is_empty(),
        }
    }

    /// Paths (relative to the vault) of conflict copies of a card's file.
    fn conflict_copies(&self, board: &LoadedBoard, id: &str) -> Vec<PathBuf> {
        let Some(loaded) = board.cards.iter().find(|card| card.card.meta.id == id) else {
            return Vec::new();
        };
        let file = Path::new(&board.dir).join(CARDS_DIR).join(&loaded.file);
        self.problems
            .iter()
            .filter_map(|problem| match problem {
                Problem::Conflict { path, original } if *original == file => Some(path.clone()),
                _ => None,
            })
            .collect()
    }

    /// Files that were skipped while loading, for the settings.
    pub fn list_problems(&self) -> Vec<VaultProblem> {
        let text = |path: &Path| {
            path.components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        };
        let view = |kind, path: &Path, original: Option<&PathBuf>| VaultProblem {
            kind,
            path: text(path),
            full_path: self.vault.root().join(path).to_string_lossy().into_owned(),
            original: original.map(|original| text(original)),
        };
        self.problems
            .iter()
            .map(|problem| match problem {
                Problem::Invalid { path } => view("invalid", path, None),
                Problem::Conflict { path, original } => view("conflict", path, Some(original)),
            })
            .collect()
    }

    /// The other versions of a card, from conflict copies of its file.
    pub fn card_conflicts(&self, id: &str) -> Result<Vec<CardVersion>> {
        let (b, _) = self.card_location(id)?;
        let mut versions = Vec::new();
        for path in self.conflict_copies(&self.boards[b], id) {
            // A copy that vanished or broke meanwhile is simply not offered.
            if let Ok(Some(copy)) = self.vault.read_card_at(&path) {
                versions.push(CardVersion {
                    path: path.to_string_lossy().into_owned(),
                    title: copy.meta.title,
                    notes: copy.notes,
                    updated_at: copy.meta.updated_at.0,
                });
            }
        }
        Ok(versions)
    }

    /// Keeps one version of a card and removes the other files. `keep` is the
    /// path of a conflict copy to take over, or None to keep the card as is.
    pub fn resolve_card_conflict(&mut self, id: &str, keep: Option<&str>) -> Result<()> {
        let (b, _) = self.card_location(id)?;
        let copies = self.conflict_copies(&self.boards[b], id);
        if let Some(keep) = keep {
            let path = copies
                .iter()
                .find(|copy| copy.to_string_lossy() == keep)
                .ok_or(AppError::NotFound)?;
            let mut chosen = self.vault.read_card_at(path)?.ok_or(AppError::NotFound)?;
            chosen.meta.id = id.to_owned();
            self.save_card(self.card_location(id)?, chosen)?;
        }
        for copy in &copies {
            self.vault.trash_file(copy)?;
            self.problems.retain(
                |problem| !matches!(problem, Problem::Conflict { path, .. } if path == copy),
            );
        }
        Ok(())
    }

    // Boards

    pub fn list_boards(&self) -> Vec<Board> {
        self.boards
            .iter()
            .map(|board| Board {
                id: board.board.id.clone(),
                name: board.board.name.clone(),
            })
            .collect()
    }

    /// The board with its columns and the active (not archived) cards.
    pub fn view(&self, id: &str) -> Result<BoardView> {
        let board = &self.boards[self.board_index(id)?];
        let mut columns: Vec<ColumnView> = board
            .board
            .columns
            .iter()
            .map(|column| ColumnView {
                column: Column {
                    id: column.id.clone(),
                    name: column.name.clone(),
                    is_done: column.is_done,
                },
                cards: Vec::new(),
            })
            .collect();
        let active = by_position(
            board
                .cards
                .iter()
                .map(|loaded| &loaded.card.meta)
                .filter(|meta| meta.archived_at.is_none()),
        );
        for meta in active {
            let column_id = column_of(&board.board, meta);
            if let Some(column) = columns.iter_mut().find(|c| c.column.id == column_id) {
                let loaded = board.cards.iter().find(|c| c.card.meta.id == meta.id);
                if let Some(loaded) = loaded {
                    column.cards.push(self.summary(board, &loaded.card));
                }
            }
        }
        Ok(BoardView {
            board: Board {
                id: board.board.id.clone(),
                name: board.board.name.clone(),
            },
            columns,
        })
    }

    /// Creates a board with the given columns; the last column becomes the done
    /// column. Column names come from the frontend because they are translated.
    pub fn create_board(&mut self, name: &str, column_names: &[String]) -> Result<Board> {
        let name = require_text(name)?;
        if column_names.is_empty() {
            return Err(AppError::Validation("board.no_columns"));
        }
        let mut columns = Vec::with_capacity(column_names.len());
        let mut previous: Option<String> = None;
        for (index, column_name) in column_names.iter().enumerate() {
            let key = position::key_between(previous.as_deref(), None)?;
            columns.push(ColumnEntry {
                id: new_id(),
                name: require_text(column_name)?,
                position: key.clone(),
                is_done: index == column_names.len() - 1,
                extra: Default::default(),
            });
            previous = Some(key);
        }
        let keys: Vec<String> = self
            .boards
            .iter()
            .map(|b| b.board.position.clone())
            .collect();
        let file = BoardFile {
            id: new_id(),
            name: name.clone(),
            position: position::key_after_last(&keys)?,
            created_at: now(),
            columns,
            extra: Default::default(),
        };
        let dir = self.vault.create_board(&file)?;
        let id = file.id.clone();
        self.boards.push(LoadedBoard {
            dir,
            board: file,
            cards: Vec::new(),
        });
        self.sort_boards();
        Ok(Board { id, name })
    }

    pub fn rename_board(&mut self, id: &str, name: &str) -> Result<()> {
        let name = require_text(name)?;
        let b = self.board_index(id)?;
        let mut file = self.fresh_board(b)?;
        file.name = name;
        self.save_board(b, file)
    }

    fn other_board_keys(&self, id: &str) -> Vec<String> {
        self.boards
            .iter()
            .filter(|board| board.board.id != id)
            .map(|board| board.board.position.clone())
            .collect()
    }

    pub fn reorder_board(&mut self, id: &str, index: usize) -> Result<()> {
        let b = self.board_index(id)?;
        let mut file = self.fresh_board(b)?;
        file.position = position::key_at(&self.other_board_keys(id), index)?;
        self.save_board(b, file)
    }

    pub fn delete_board(&mut self, id: &str) -> Result<()> {
        let b = self.board_index(id)?;
        let trash = self.vault.trash_board(&self.boards[b].dir)?;
        let board = self.boards.remove(b);
        self.trash.boards.push(TrashedBoard { board, trash });
        Ok(())
    }

    pub fn restore_board(&mut self, id: &str) -> Result<()> {
        let t = self
            .trash
            .boards
            .iter()
            .position(|trashed| trashed.board.board.id == id)
            .ok_or(AppError::NotFound)?;
        let trashed = &self.trash.boards[t];
        let dir = self
            .vault
            .restore_board(&trashed.trash, &trashed.board.board.name)?;
        let TrashedBoard { mut board, .. } = self.trash.boards.remove(t);
        board.dir = dir;
        let wanted = board.board.position.clone();
        let key = position::free_key(&self.other_board_keys(id), &wanted)?;
        self.boards.push(board);
        self.sort_boards();
        if key != wanted {
            let b = self.board_index(id)?;
            let mut file = self.fresh_board(b)?;
            file.position = key;
            self.save_board(b, file)?;
        }
        Ok(())
    }

    // Columns

    /// Appends a new column at the end of the board.
    pub fn create_column(&mut self, board_id: &str, name: &str) -> Result<Column> {
        let name = require_text(name)?;
        let b = self.board_index(board_id)?;
        let mut file = self.fresh_board(b)?;
        let keys: Vec<String> = file.columns.iter().map(|c| c.position.clone()).collect();
        let column = ColumnEntry {
            id: new_id(),
            name: name.clone(),
            position: position::key_after_last(&keys)?,
            is_done: false,
            extra: Default::default(),
        };
        let id = column.id.clone();
        file.columns.push(column);
        self.save_board(b, file)?;
        Ok(Column {
            id,
            name,
            is_done: false,
        })
    }

    pub fn rename_column(&mut self, id: &str, name: &str) -> Result<()> {
        let name = require_text(name)?;
        let (b, mut file, c) = self.fresh_column(id)?;
        file.columns[c].name = name;
        self.save_board(b, file)
    }

    pub fn reorder_column(&mut self, id: &str, index: usize) -> Result<()> {
        let (b, mut file, c) = self.fresh_column(id)?;
        let keys: Vec<String> = file
            .columns
            .iter()
            .filter(|column| column.id != id)
            .map(|column| column.position.clone())
            .collect();
        file.columns[c].position = position::key_at(&keys, index)?;
        self.save_board(b, file)
    }

    /// Makes this column the board's done column.
    pub fn set_done_column(&mut self, id: &str) -> Result<()> {
        let (b, mut file, c) = self.fresh_column(id)?;
        for (index, column) in file.columns.iter_mut().enumerate() {
            column.is_done = index == c;
        }
        self.save_board(b, file)
    }

    /// Deletes the column together with its cards. The done column cannot be
    /// deleted; another column has to become the done column first.
    pub fn delete_column(&mut self, id: &str) -> Result<()> {
        let (b, mut file, c) = self.fresh_column(id)?;
        if file.columns[c].is_done {
            return Err(AppError::Conflict("column.is_done"));
        }
        let dir = self.boards[b].dir.clone();
        // Trash the cards first: if that fails half-way, the column is still
        // there and the remaining cards show in it.
        let mut cards = Vec::new();
        while let Some(k) = self.boards[b]
            .cards
            .iter()
            .position(|card| card.card.meta.column == id)
        {
            self.vault.trash_card(&dir, &self.boards[b].cards[k].file)?;
            cards.push(self.boards[b].cards.remove(k));
        }
        let column = file.columns.remove(c);
        let board_id = file.id.clone();
        let saved = self.save_board(b, file);
        self.trash.columns.push(TrashedColumn {
            board_id,
            column,
            cards,
        });
        saved
    }

    pub fn restore_column(&mut self, id: &str) -> Result<()> {
        let t = self
            .trash
            .columns
            .iter()
            .position(|trashed| trashed.column.id == id)
            .ok_or(AppError::NotFound)?;
        let board_id = self.trash.columns[t].board_id.clone();
        let b = self.board_index(&board_id)?;
        let TrashedColumn {
            mut column, cards, ..
        } = self.trash.columns.remove(t);

        let mut file = self.fresh_board(b)?;
        let keys: Vec<String> = file.columns.iter().map(|c| c.position.clone()).collect();
        column.position = position::free_key(&keys, &column.position)?;
        column.is_done = false;
        file.columns.push(column);
        self.save_board(b, file)?;
        let b = self.board_index(&board_id)?;

        let dir = self.boards[b].dir.clone();
        for card in cards {
            self.vault.restore_card(&dir, &card.file)?;
            self.boards[b].cards.push(card);
        }
        Ok(())
    }

    // Cards

    /// Ids of active cards on a board whose title or notes contain `query`,
    /// ignoring case.
    pub fn search_cards(&self, board_id: &str, query: &str) -> Result<Vec<Id>> {
        let board = &self.boards[self.board_index(board_id)?];
        let needle = query.trim().to_lowercase();
        let matches = |meta: &CardMeta| {
            meta.archived_at.is_none()
                && board.cards.iter().any(|loaded| {
                    loaded.card.meta.id == meta.id
                        && (meta.title.to_lowercase().contains(&needle)
                            || loaded.card.notes.to_lowercase().contains(&needle))
                })
        };
        Ok(
            by_position(board.cards.iter().map(|loaded| &loaded.card.meta))
                .into_iter()
                .filter(|meta| matches(meta))
                .map(|meta| meta.id.clone())
                .collect(),
        )
    }

    pub fn get_card(&self, id: &str) -> Result<CardDetail> {
        let (b, k) = self.card_location(id)?;
        let card = self.card((b, k));
        let meta = &card.meta;
        Ok(CardDetail {
            id: meta.id.clone(),
            column_id: column_of(&self.boards[b].board, meta).to_owned(),
            title: meta.title.clone(),
            notes: card.notes.clone(),
            priority: meta.priority,
            created_at: meta.created_at.0,
            updated_at: meta.updated_at.0,
            archived_at: meta.archived_at.map(|at| at.0),
            subtasks: meta
                .subtasks
                .iter()
                .map(|subtask| Subtask {
                    id: subtask.id.clone(),
                    title: subtask.title.clone(),
                    done: subtask.done,
                })
                .collect(),
            tag_ids: self.known_tags(&meta.tags),
            has_conflict: !self.conflict_copies(&self.boards[b], &meta.id).is_empty(),
        })
    }

    /// Appends a new card at the end of the column.
    pub fn create_card(&mut self, new: &NewCard) -> Result<CardSummary> {
        let title = require_text(&new.title)?;
        let (b, _) = self.column_location(&new.column_id)?;
        let tags = self.merge_tags(&[], &new.tag_ids)?;
        let stamp = now();
        let card = Card {
            meta: CardMeta {
                id: new_id(),
                column: new.column_id.clone(),
                position: position::key_after_last(&active_keys(
                    &self.boards[b],
                    &new.column_id,
                    "",
                ))?,
                title,
                priority: new.priority,
                tags,
                subtasks: Vec::new(),
                created_at: stamp,
                updated_at: stamp,
                archived_at: None,
                extra: Default::default(),
            },
            notes: String::new(),
        };
        self.vault.write_card(&self.boards[b].dir, &card)?;
        let summary = self.summary(&self.boards[b], &card);
        self.boards[b].cards.push(LoadedCard {
            file: card_file_name(&card.meta.id),
            card,
        });
        Ok(summary)
    }

    pub fn update_card(&mut self, id: &str, patch: &CardPatch) -> Result<()> {
        let title = patch.title.as_deref().map(require_text).transpose()?;
        let at = self.card_location(id)?;
        let mut card = self.fresh_card(at)?;
        if let Some(title) = title {
            card.meta.title = title;
        }
        if let Some(notes) = &patch.notes {
            card.notes.clone_from(notes);
        }
        if let Some(priority) = patch.priority {
            card.meta.priority = priority;
        }
        card.meta.updated_at = now();
        self.save_card(at, card)
    }

    /// Moves an active card to `index` within `column_id` of the same board.
    pub fn move_card(&mut self, id: &str, column_id: &str, index: usize) -> Result<()> {
        let (b, k) = self.card_location(id)?;
        let (column_board, _) = self.column_location(column_id)?;
        if column_board != b {
            return Err(AppError::NotFound);
        }
        let mut card = self.fresh_card((b, k))?;
        if card.meta.archived_at.is_some() {
            return Err(AppError::Conflict("card.archived"));
        }
        card.meta.position = position::key_at(&active_keys(&self.boards[b], column_id, id), index)?;
        card.meta.column = column_id.to_owned();
        card.meta.updated_at = now();
        self.save_card((b, k), card)
    }

    pub fn delete_card(&mut self, id: &str) -> Result<()> {
        let (b, k) = self.card_location(id)?;
        self.vault
            .trash_card(&self.boards[b].dir, &self.boards[b].cards[k].file)?;
        let card = self.boards[b].cards.remove(k);
        let board_id = self.boards[b].board.id.clone();
        self.trash.cards.push(TrashedCard { board_id, card });
        Ok(())
    }

    pub fn restore_card(&mut self, id: &str) -> Result<()> {
        let t = self
            .trash
            .cards
            .iter()
            .position(|trashed| trashed.card.card.meta.id == id)
            .ok_or(AppError::NotFound)?;
        let b = self.board_index(&self.trash.cards[t].board_id)?;
        self.vault
            .restore_card(&self.boards[b].dir, &self.trash.cards[t].card.file)?;
        let TrashedCard { card, .. } = self.trash.cards.remove(t);
        self.boards[b].cards.push(card);

        let k = self.boards[b].cards.len() - 1;
        let meta = &self.boards[b].cards[k].card.meta;
        // Archived cards have no place in the column order, so keep their key.
        if meta.archived_at.is_none() {
            let column = column_of(&self.boards[b].board, meta).to_owned();
            let key =
                position::free_key(&active_keys(&self.boards[b], &column, id), &meta.position)?;
            if key != meta.position {
                let mut card = self.boards[b].cards[k].card.clone();
                card.meta.position = key;
                self.save_card((b, k), card)?;
            }
        }
        Ok(())
    }

    pub fn archive_card(&mut self, id: &str) -> Result<()> {
        let at = self.card_location(id)?;
        let mut card = self.fresh_card(at)?;
        if card.meta.archived_at.is_some() {
            return Err(AppError::Conflict("card.archived"));
        }
        card.meta.archived_at = Some(now());
        self.save_card(at, card)
    }

    /// Puts an archived card back into its column, at its old place if free.
    pub fn unarchive_card(&mut self, id: &str) -> Result<()> {
        let (b, k) = self.card_location(id)?;
        let mut card = self.fresh_card((b, k))?;
        if card.meta.archived_at.is_none() {
            return Err(AppError::Conflict("card.not_archived"));
        }
        let column = column_of(&self.boards[b].board, &card.meta).to_owned();
        card.meta.position = position::free_key(
            &active_keys(&self.boards[b], &column, id),
            &card.meta.position,
        )?;
        card.meta.archived_at = None;
        self.save_card((b, k), card)
    }

    /// Archives every active card in the board's done column and returns their
    /// ids, so the frontend can offer undo.
    pub fn archive_done_cards(&mut self, board_id: &str) -> Result<Vec<Id>> {
        let b = self.board_index(board_id)?;
        let board = &self.boards[b];
        let Some(done) = board.board.columns.iter().find(|column| column.is_done) else {
            return Ok(Vec::new());
        };
        let ids: Vec<Id> = by_position(
            board
                .cards
                .iter()
                .map(|loaded| &loaded.card.meta)
                .filter(|meta| {
                    meta.archived_at.is_none() && column_of(&board.board, meta) == done.id
                }),
        )
        .into_iter()
        .map(|meta| meta.id.clone())
        .collect();
        let stamp = now();
        for id in &ids {
            let Some(mut card) = self.fresh_card_by_id(id)? else {
                continue;
            };
            card.meta.archived_at = Some(stamp);
            self.save_card(self.card_location(id)?, card)?;
        }
        Ok(ids)
    }

    /// Archived cards of a board, most recently archived first.
    pub fn list_archived_cards(&self, board_id: &str) -> Result<Vec<CardSummary>> {
        let board = &self.boards[self.board_index(board_id)?];
        let mut archived: Vec<&Card> = board
            .cards
            .iter()
            .map(|loaded| &loaded.card)
            .filter(|card| card.meta.archived_at.is_some())
            .collect();
        archived.sort_by(|a, b| {
            (b.meta.archived_at.map(|at| at.0), &b.meta.id)
                .cmp(&(a.meta.archived_at.map(|at| at.0), &a.meta.id))
        });
        Ok(archived
            .into_iter()
            .map(|card| self.summary(board, card))
            .collect())
    }

    pub fn set_card_tags(&mut self, id: &str, tag_ids: &[Id]) -> Result<Vec<Id>> {
        let at = self.card_location(id)?;
        let mut card = self.fresh_card(at)?;
        card.meta.tags = self.merge_tags(&card.meta.tags, tag_ids)?;
        card.meta.updated_at = now();
        let known = self.known_tags(&card.meta.tags);
        self.save_card(at, card)?;
        Ok(known)
    }

    // Subtasks

    /// Appends a new subtask at the end of the card's checklist.
    pub fn create_subtask(&mut self, card_id: &str, title: &str) -> Result<Subtask> {
        let title = require_text(title)?;
        let at = self.card_location(card_id)?;
        let mut card = self.fresh_card(at)?;
        let subtask = SubtaskEntry {
            id: new_id(),
            title: title.clone(),
            done: false,
            extra: Default::default(),
        };
        let id = subtask.id.clone();
        card.meta.subtasks.push(subtask);
        self.save_card(at, card)?;
        Ok(Subtask {
            id,
            title,
            done: false,
        })
    }

    pub fn update_subtask(&mut self, id: &str, patch: &SubtaskPatch) -> Result<()> {
        let title = patch.title.as_deref().map(require_text).transpose()?;
        let (b, k, _) = self.subtask_location(id)?;
        let mut card = self.fresh_card((b, k))?;
        let s = subtask_index(&card, id)?;
        let subtask = &mut card.meta.subtasks[s];
        if let Some(title) = title {
            subtask.title = title;
        }
        if let Some(done) = patch.done {
            subtask.done = done;
        }
        self.save_card((b, k), card)
    }

    pub fn reorder_subtask(&mut self, id: &str, index: usize) -> Result<()> {
        let (b, k, _) = self.subtask_location(id)?;
        let mut card = self.fresh_card((b, k))?;
        let s = subtask_index(&card, id)?;
        let subtask = card.meta.subtasks.remove(s);
        let index = index.min(card.meta.subtasks.len());
        card.meta.subtasks.insert(index, subtask);
        self.save_card((b, k), card)
    }

    /// Deleted for good: a subtask is small enough to be re-created for undo.
    pub fn delete_subtask(&mut self, id: &str) -> Result<()> {
        let (b, k, _) = self.subtask_location(id)?;
        let mut card = self.fresh_card((b, k))?;
        let s = subtask_index(&card, id)?;
        card.meta.subtasks.remove(s);
        self.save_card((b, k), card)
    }

    // Tags

    pub fn list_tags(&self) -> Vec<Tag> {
        let mut tags: Vec<Tag> = self
            .tags
            .tags
            .iter()
            .map(|tag| Tag {
                id: tag.id.clone(),
                name: tag.name.clone(),
                // A colour from a newer version falls back to the neutral one.
                color: if TAG_COLORS.contains(&tag.color.as_str()) {
                    tag.color.clone()
                } else {
                    TAG_COLORS[0].to_owned()
                },
            })
            .collect();
        tags.sort_by_key(|tag| tag.name.to_lowercase());
        tags
    }

    /// Fails with `tag.duplicate_name` if another tag has the name, ignoring case.
    fn require_free_name(&self, name: &str, except: &str) -> Result<()> {
        let lower = name.to_lowercase();
        if self
            .tags
            .tags
            .iter()
            .any(|tag| tag.id != except && tag.name.to_lowercase() == lower)
        {
            return Err(AppError::Conflict("tag.duplicate_name"));
        }
        Ok(())
    }

    fn save_tags(&mut self, tags: TagsFile) -> Result<()> {
        self.vault.write_tags(&tags)?;
        self.tags = tags;
        Ok(())
    }

    pub fn create_tag(&mut self, name: &str, color: &str) -> Result<Tag> {
        let name = require_text(name)?;
        let color = require_color(color)?.to_owned();
        let mut tags = self.fresh_tags()?;
        self.require_free_name(&name, "")?;
        let tag = Tag {
            id: new_id(),
            name,
            color,
        };
        tags.tags.push(TagEntry {
            id: tag.id.clone(),
            name: tag.name.clone(),
            color: tag.color.clone(),
            extra: Default::default(),
        });
        self.save_tags(tags)?;
        Ok(tag)
    }

    pub fn update_tag(&mut self, id: &str, patch: &TagPatch) -> Result<()> {
        let name = patch.name.as_deref().map(require_text).transpose()?;
        let color = patch.color.as_deref().map(require_color).transpose()?;
        let mut tags = self.fresh_tags()?;
        let tag = tags
            .tags
            .iter_mut()
            .find(|tag| tag.id == id)
            .ok_or(AppError::NotFound)?;
        if let Some(name) = name {
            self.require_free_name(&name, id)?;
            tag.name = name;
        }
        if let Some(color) = color {
            color.clone_into(&mut tag.color);
        }
        self.save_tags(tags)
    }

    fn cards_with_tag(&self, id: &str) -> Vec<(usize, usize)> {
        self.boards
            .iter()
            .enumerate()
            .flat_map(|(b, board)| {
                board
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, card)| card.card.meta.tags.iter().any(|tag| tag == id))
                    .map(move |(k, _)| (b, k))
            })
            .collect()
    }

    /// How many cards (active or archived) carry the tag, to warn before deleting.
    pub fn count_tag_cards(&self, id: &str) -> Result<u32> {
        self.tag(id).ok_or(AppError::NotFound)?;
        Ok(self.cards_with_tag(id).len() as u32)
    }

    /// Removes the tag from every card, then from the tag list.
    pub fn delete_tag(&mut self, id: &str) -> Result<()> {
        self.fresh_tags()?;
        self.tag(id).ok_or(AppError::NotFound)?;
        let card_ids: Vec<Id> = self
            .cards_with_tag(id)
            .into_iter()
            .map(|at| self.card(at).meta.id.clone())
            .collect();
        for card_id in card_ids {
            let Some(mut card) = self.fresh_card_by_id(&card_id)? else {
                continue;
            };
            card.meta.tags.retain(|tag| tag != id);
            self.save_card(self.card_location(&card_id)?, card)?;
        }
        let mut tags = self.fresh_tags()?;
        tags.tags.retain(|tag| tag.id != id);
        self.save_tags(tags)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::model::Priority;
    use crate::vault::tests::TestDir;

    fn open(dir: &TestDir) -> VaultStore {
        VaultStore::load(Vault::open(dir.path()).unwrap()).unwrap()
    }

    /// Reads the files again like a restart would, but keeps the trash.
    fn reload(dir: &TestDir) -> VaultStore {
        VaultStore::load(Vault {
            root: dir.path().to_owned(),
        })
        .unwrap()
    }

    fn columns() -> Vec<String> {
        vec!["Offen".into(), "In Arbeit".into(), "Erledigt".into()]
    }

    fn new_card(column_id: &str, title: &str) -> NewCard {
        NewCard {
            column_id: column_id.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    /// Titles of the active cards per column.
    fn titles(store: &VaultStore, board_id: &str) -> Vec<Vec<String>> {
        store
            .view(board_id)
            .unwrap()
            .columns
            .into_iter()
            .map(|column| column.cards.into_iter().map(|card| card.title).collect())
            .collect()
    }

    fn column_ids(store: &VaultStore, board_id: &str) -> Vec<Id> {
        store
            .view(board_id)
            .unwrap()
            .columns
            .into_iter()
            .map(|column| column.column.id)
            .collect()
    }

    fn code<T: std::fmt::Debug>(result: Result<T>) -> &'static str {
        result.unwrap_err().code()
    }

    #[test]
    fn boards_are_created_ordered_and_survive_reloading() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let work = store.create_board(" Arbeit ", &columns()).unwrap();
        let home = store.create_board("Privat", &columns()).unwrap();
        assert_eq!(work.name, "Arbeit");

        let view = store.view(&work.id).unwrap();
        let done: Vec<bool> = view.columns.iter().map(|c| c.column.is_done).collect();
        assert_eq!(done, [false, false, true]);

        store.reorder_board(&home.id, 0).unwrap();
        store.rename_board(&work.id, "Job").unwrap();
        let names = |store: &VaultStore| -> Vec<String> {
            store.list_boards().into_iter().map(|b| b.name).collect()
        };
        assert_eq!(names(&store), ["Privat", "Job"]);
        assert_eq!(names(&reload(&dir)), ["Privat", "Job"]);

        assert_eq!(
            code(store.create_board("  ", &columns())),
            "validation.empty"
        );
        assert_eq!(code(store.create_board("Leer", &[])), "board.no_columns");
        assert_eq!(code(store.rename_board("fehlt", "X")), "not_found");
    }

    #[test]
    fn deleted_boards_come_back_with_their_cards() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let work = store.create_board("Arbeit", &columns()).unwrap();
        let home = store.create_board("Privat", &columns()).unwrap();
        let column = column_ids(&store, &work.id)[0].clone();
        store.create_card(&new_card(&column, "Bericht")).unwrap();

        store.delete_board(&work.id).unwrap();
        assert_eq!(store.list_boards().len(), 1);
        assert_eq!(reload(&dir).list_boards().len(), 1);
        assert_eq!(code(store.view(&work.id)), "not_found");

        store.restore_board(&work.id).unwrap();
        let ids: Vec<Id> = store.list_boards().into_iter().map(|b| b.id).collect();
        assert_eq!(ids, [work.id.clone(), home.id]);
        assert_eq!(titles(&reload(&dir), &work.id)[0], ["Bericht"]);
        assert_eq!(code(store.restore_board(&work.id)), "not_found");
    }

    #[test]
    fn columns_are_added_moved_and_marked_done() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let review = store.create_column(&board.id, "Review").unwrap();
        store.reorder_column(&review.id, 1).unwrap();
        store.rename_column(&review.id, "Prüfen").unwrap();
        store.set_done_column(&review.id).unwrap();

        let reloaded = open(&dir);
        let view = reloaded.view(&board.id).unwrap();
        let columns: Vec<(&str, bool)> = view
            .columns
            .iter()
            .map(|c| (c.column.name.as_str(), c.column.is_done))
            .collect();
        assert_eq!(
            columns,
            [
                ("Offen", false),
                ("Prüfen", true),
                ("In Arbeit", false),
                ("Erledigt", false)
            ]
        );
    }

    #[test]
    fn deleted_columns_take_their_cards_and_bring_them_back() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let ids = column_ids(&store, &board.id);
        store.create_card(&new_card(&ids[1], "Läuft")).unwrap();
        store.create_card(&new_card(&ids[0], "Offen")).unwrap();

        assert_eq!(code(store.delete_column(&ids[2])), "column.is_done");
        store.delete_column(&ids[1]).unwrap();
        assert_eq!(titles(&reload(&dir), &board.id), [vec!["Offen"], vec![]]);

        store.restore_column(&ids[1]).unwrap();
        assert_eq!(
            titles(&reload(&dir), &board.id),
            [vec!["Offen"], vec!["Läuft"], vec![]]
        );
    }

    #[test]
    fn cards_are_created_edited_and_moved() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let ids = column_ids(&store, &board.id);
        let a = store.create_card(&new_card(&ids[0], "A")).unwrap();
        let b = store.create_card(&new_card(&ids[0], "B")).unwrap();
        store.create_card(&new_card(&ids[0], "C")).unwrap();
        assert_eq!(a.column_id, ids[0]);

        store.move_card(&b.id, &ids[0], 0).unwrap();
        store.move_card(&a.id, &ids[1], 0).unwrap();
        store
            .update_card(
                &a.id,
                &CardPatch {
                    title: Some("A2".into()),
                    notes: Some("Zeile 1\nZeile 2".into()),
                    priority: Some(Priority::High),
                },
            )
            .unwrap();

        let store = open(&dir);
        assert_eq!(
            titles(&store, &board.id),
            [vec!["B", "C"], vec!["A2"], vec![]]
        );
        let detail = store.get_card(&a.id).unwrap();
        assert_eq!(detail.notes, "Zeile 1\nZeile 2");
        assert_eq!(detail.priority, Priority::High);
        assert!(detail.updated_at >= detail.created_at);
    }

    #[test]
    fn cards_cannot_move_to_other_boards_or_while_archived() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let one = store.create_board("1", &columns()).unwrap();
        let two = store.create_board("2", &columns()).unwrap();
        let card = store
            .create_card(&new_card(&column_ids(&store, &one.id)[0], "K"))
            .unwrap();
        let other = column_ids(&store, &two.id)[0].clone();
        assert_eq!(code(store.move_card(&card.id, &other, 0)), "not_found");

        store.archive_card(&card.id).unwrap();
        let own = column_ids(&store, &one.id)[1].clone();
        assert_eq!(code(store.move_card(&card.id, &own, 0)), "card.archived");
        assert_eq!(code(store.archive_card(&card.id)), "card.archived");
    }

    #[test]
    fn deleted_cards_return_to_their_place() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        store.create_card(&new_card(&column, "A")).unwrap();
        let b = store.create_card(&new_card(&column, "B")).unwrap();
        store.create_card(&new_card(&column, "C")).unwrap();

        store.delete_card(&b.id).unwrap();
        assert_eq!(titles(&reload(&dir), &board.id)[0], ["A", "C"]);
        assert_eq!(code(store.get_card(&b.id)), "not_found");
        store.restore_card(&b.id).unwrap();
        assert_eq!(titles(&reload(&dir), &board.id)[0], ["A", "B", "C"]);
    }

    #[test]
    fn archive_keeps_cards_out_of_the_board() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let ids = column_ids(&store, &board.id);
        let a = store.create_card(&new_card(&ids[0], "A")).unwrap();
        store.create_card(&new_card(&ids[0], "B")).unwrap();
        let done1 = store.create_card(&new_card(&ids[2], "Fertig 1")).unwrap();
        let done2 = store.create_card(&new_card(&ids[2], "Fertig 2")).unwrap();

        store.archive_card(&a.id).unwrap();
        assert_eq!(
            store.archive_done_cards(&board.id).unwrap(),
            [done1.id.clone(), done2.id.clone()]
        );
        assert_eq!(titles(&store, &board.id), [vec!["B"], vec![], vec![]]);

        let archived: Vec<String> = open(&dir)
            .list_archived_cards(&board.id)
            .unwrap()
            .into_iter()
            .map(|card| card.title)
            .collect();
        assert_eq!(archived.len(), 3);
        assert!(archived.contains(&"A".to_owned()));

        store.unarchive_card(&a.id).unwrap();
        assert_eq!(titles(&reload(&dir), &board.id)[0], ["A", "B"]);
        assert_eq!(code(store.unarchive_card(&a.id)), "card.not_archived");
        assert!(store.search_cards(&board.id, "fertig").unwrap().is_empty());
    }

    #[test]
    fn search_looks_at_titles_and_notes_ignoring_case() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let a = store
            .create_card(&new_card(&column, "Größe messen"))
            .unwrap();
        let b = store.create_card(&new_card(&column, "Einkaufen")).unwrap();
        store.create_card(&new_card(&column, "Anderes")).unwrap();
        store
            .update_card(
                &b.id,
                &CardPatch {
                    notes: Some("GRÖSSE beachten, größer".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            store.search_cards(&board.id, " GRÖ ").unwrap(),
            [a.id, b.id]
        );
    }

    #[test]
    fn subtasks_are_kept_in_order() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let card = store
            .create_card(&new_card(&column_ids(&store, &board.id)[0], "K"))
            .unwrap();
        let one = store.create_subtask(&card.id, "Eins").unwrap();
        let two = store.create_subtask(&card.id, "Zwei").unwrap();
        let three = store.create_subtask(&card.id, "Drei").unwrap();
        store.reorder_subtask(&three.id, 0).unwrap();
        store
            .update_subtask(
                &one.id,
                &SubtaskPatch {
                    title: Some("Eins!".into()),
                    done: Some(true),
                },
            )
            .unwrap();
        store.delete_subtask(&two.id).unwrap();

        let store = open(&dir);
        let subtasks: Vec<(String, bool)> = store
            .get_card(&card.id)
            .unwrap()
            .subtasks
            .into_iter()
            .map(|s| (s.title, s.done))
            .collect();
        assert_eq!(subtasks, [("Drei".into(), false), ("Eins!".into(), true)]);
        let summary = &store.view(&board.id).unwrap().columns[0].cards[0];
        assert_eq!((summary.subtasks_done, summary.subtasks_total), (1, 2));
        assert_eq!(code(reload(&dir).reorder_subtask("fehlt", 0)), "not_found");
    }

    #[test]
    fn tags_are_named_uniquely_and_sorted() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let zebra = store.create_tag("Zebra", "red").unwrap();
        store.create_tag("apfel", "green").unwrap();
        assert_eq!(
            code(store.create_tag("ZEBRA", "blue")),
            "tag.duplicate_name"
        );
        assert_eq!(code(store.create_tag("Neu", "pink")), "tag.invalid_color");

        store
            .update_tag(
                &zebra.id,
                &TagPatch {
                    name: Some("Banane".into()),
                    color: Some("yellow".into()),
                },
            )
            .unwrap();
        assert_eq!(
            code(store.update_tag(
                &zebra.id,
                &TagPatch {
                    name: Some("APFEL".into()),
                    ..Default::default()
                }
            )),
            "tag.duplicate_name"
        );
        let tags: Vec<(String, String)> = open(&dir)
            .list_tags()
            .into_iter()
            .map(|t| (t.name, t.color))
            .collect();
        assert_eq!(
            tags,
            [
                ("apfel".into(), "green".into()),
                ("Banane".into(), "yellow".into())
            ]
        );
    }

    #[test]
    fn deleting_a_tag_removes_it_from_cards() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let tag = store.create_tag("Steuern", "orange").unwrap();
        let other = store.create_tag("Arbeit", "blue").unwrap();
        let card = store
            .create_card(&NewCard {
                tag_ids: vec![tag.id.clone(), other.id.clone()],
                ..new_card(&column, "K")
            })
            .unwrap();
        assert_eq!(card.tag_ids, [other.id.clone(), tag.id.clone()]);
        store.archive_card(&card.id).unwrap();
        assert_eq!(store.count_tag_cards(&tag.id).unwrap(), 1);

        store.delete_tag(&tag.id).unwrap();
        let store = open(&dir);
        assert_eq!(store.get_card(&card.id).unwrap().tag_ids, [other.id]);
        assert_eq!(code(store.count_tag_cards(&tag.id)), "not_found");
    }

    #[test]
    fn unknown_tags_and_columns_on_cards_are_tolerated() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let ids = column_ids(&store, &board.id);
        let tag = store.create_tag("Bekannt", "red").unwrap();
        let card = store.create_card(&new_card(&ids[1], "K")).unwrap();

        // Someone else's change: a tag not yet in tags.json, a deleted column.
        let path = dir.path().join("B/cards").join(card_file_name(&card.id));
        let text = fs::read_to_string(&path).unwrap();
        let text = text
            .replace(&ids[1], "weg")
            .replace("title: K\n", "title: K\ntags:\n- fremd\n");
        fs::write(&path, text).unwrap();

        let mut store = open(&dir);
        assert_eq!(titles(&store, &board.id)[0], ["K"]);
        let detail = store.get_card(&card.id).unwrap();
        assert_eq!(detail.column_id, ids[0]);
        assert!(detail.tag_ids.is_empty());

        assert_eq!(
            store
                .set_card_tags(&card.id, std::slice::from_ref(&tag.id))
                .unwrap(),
            [tag.id.as_str()]
        );
        assert_eq!(
            code(store.set_card_tags(&card.id, &["nix".into()])),
            "not_found"
        );
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("- fremd\n"), "{text}");
        assert!(text.contains(&format!("column: {}\n", ids[0])), "{text}");
    }

    fn card_path(dir: &TestDir, board_dir: &str, id: &str) -> std::path::PathBuf {
        dir.path()
            .join(board_dir)
            .join("cards")
            .join(card_file_name(id))
    }

    #[test]
    fn refresh_notices_outside_changes_but_not_its_own() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let card = store.create_card(&new_card(&column, "Alt")).unwrap();
        assert!(!store.refresh().unwrap());

        let path = card_path(&dir, "B", &card.id);
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.replace("title: Alt", "title: Neu")).unwrap();
        assert!(store.refresh().unwrap());
        assert_eq!(titles(&store, &board.id)[0], ["Neu"]);
        assert!(!store.refresh().unwrap());
    }

    #[test]
    fn changes_build_on_what_others_wrote_meanwhile() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let card = store.create_card(&new_card(&column, "Alt")).unwrap();
        let subtask = store.create_subtask(&card.id, "Schritt").unwrap();

        // Someone renames the card; this store has not noticed yet.
        let path = card_path(&dir, "B", &card.id);
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.replace("title: Alt", "title: Von Anna")).unwrap();
        store
            .update_card(
                &card.id,
                &CardPatch {
                    notes: Some("Meine Notiz".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        store
            .update_subtask(
                &subtask.id,
                &SubtaskPatch {
                    done: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();

        let detail = reload(&dir).get_card(&card.id).unwrap();
        assert_eq!(detail.title, "Von Anna");
        assert_eq!(detail.notes, "Meine Notiz");
        assert!(detail.subtasks[0].done);

        // Someone renames a column while this store renames the board.
        let board_json = dir.path().join("B/board.json");
        let text = fs::read_to_string(&board_json).unwrap();
        fs::write(&board_json, text.replace("\"Offen\"", "\"Backlog\"")).unwrap();
        store.rename_board(&board.id, "Projekt").unwrap();
        let view = reload(&dir).view(&board.id).unwrap();
        assert_eq!(view.board.name, "Projekt");
        assert_eq!(view.columns[0].column.name, "Backlog");
    }

    #[test]
    fn a_card_deleted_elsewhere_is_gone_here_too() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let card = store.create_card(&new_card(&column, "K")).unwrap();
        let subtask = store.create_subtask(&card.id, "S").unwrap();

        let path = card_path(&dir, "B", &card.id);
        let text = fs::read_to_string(&path).unwrap();
        let without_subtask: String = text
            .lines()
            .filter(|line| {
                !line.contains("subtasks:") && !line.starts_with("- ") && !line.starts_with("  ")
            })
            .map(|line| format!("{line}\n"))
            .collect();
        fs::write(&path, without_subtask).unwrap();
        assert_eq!(code(store.delete_subtask(&subtask.id)), "not_found");

        fs::remove_file(&path).unwrap();
        assert_eq!(
            code(store.update_card(&card.id, &CardPatch::default())),
            "not_found"
        );
        assert!(titles(&store, &board.id)[0].is_empty());
    }

    #[test]
    fn conflict_copies_are_shown_and_resolved() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let board = store.create_board("B", &columns()).unwrap();
        let column = column_ids(&store, &board.id)[0].clone();
        let card = store.create_card(&new_card(&column, "Meins")).unwrap();
        let path = card_path(&dir, "B", &card.id);
        let theirs = fs::read_to_string(&path)
            .unwrap()
            .replace("title: Meins", "title: Deins");
        let copy = dir.path().join("B/cards/Kopie.md");
        fs::write(&copy, &theirs).unwrap();
        assert!(store.refresh().unwrap());

        assert!(store.view(&board.id).unwrap().columns[0].cards[0].has_conflict);
        assert!(store.get_card(&card.id).unwrap().has_conflict);
        let versions = store.card_conflicts(&card.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].title, "Deins");
        let problems = store.list_problems();
        assert_eq!(problems[0].kind, "conflict");
        assert_eq!(problems[0].path, "B/cards/Kopie.md");

        store
            .resolve_card_conflict(&card.id, Some(&versions[0].path))
            .unwrap();
        assert!(!copy.exists());
        let detail = store.get_card(&card.id).unwrap();
        assert_eq!(detail.title, "Deins");
        assert!(!detail.has_conflict);
        assert!(!store.refresh().unwrap());

        fs::write(&copy, theirs.replace("Deins", "Noch eins")).unwrap();
        store.refresh().unwrap();
        store.resolve_card_conflict(&card.id, None).unwrap();
        assert_eq!(store.get_card(&card.id).unwrap().title, "Deins");
        assert!(store.list_problems().is_empty());
    }
}
