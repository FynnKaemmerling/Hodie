//! Moodboards in the open vault: folders with a `moodboard.json` that places
//! images from their `images/` folder. They share the sidebar order with the
//! boards, so renaming, moving and deleting go through the board commands.

use std::fs;
use std::path::{Path, PathBuf};

use super::super::format::{ImageEntry, MoodboardFile};
use super::super::image::{self, MAX_BYTES};
use super::{LoadedMoodboard, VaultStore, new_id, now};
use crate::error::{AppError, Result};
use crate::model::{Board, BoardKind, Id, ImagePatch, MoodboardImage, MoodboardView};
use crate::position;
use crate::store::require_text;

/// A new image is shown at most this large (its longer side), so a photo
/// does not cover the whole moodboard.
const NEW_IMAGE_SIZE: f64 = 360.0;
/// Images cannot be made smaller than this (shorter side).
pub const MIN_IMAGE_SIZE: f64 = 16.0;

pub(super) struct TrashedMoodboard {
    moodboard: LoadedMoodboard,
    /// Hidden folder name in the vault.
    trash: String,
}

pub(super) struct TrashedImage {
    moodboard_id: Id,
    /// Layer it had, to put it back there.
    index: usize,
    image: ImageEntry,
}

/// Two decimals are plenty for CSS pixels and keep the file readable.
fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn view_of(entry: &ImageEntry) -> MoodboardImage {
    MoodboardImage {
        id: entry.id.clone(),
        file: entry.file.clone(),
        x: entry.x,
        y: entry.y,
        width: entry.width,
        height: entry.height,
    }
}

fn board_of(file: &MoodboardFile) -> Board {
    Board {
        id: file.id.clone(),
        name: file.name.clone(),
        kind: BoardKind::Moodboard,
    }
}

/// Size for a new image: its own, scaled down to `NEW_IMAGE_SIZE`.
fn initial_size(width: u32, height: u32) -> (f64, f64) {
    let (width, height) = (width as f64, height as f64);
    let scale = (NEW_IMAGE_SIZE / width.max(height)).min(1.0);
    (round(width * scale), round(height * scale))
}

impl VaultStore {
    pub(super) fn is_moodboard(&self, id: &str) -> bool {
        self.moodboard_index(id).is_ok()
    }

    pub(super) fn is_trashed_moodboard(&self, id: &str) -> bool {
        self.trash
            .moodboards
            .iter()
            .any(|trashed| trashed.moodboard.moodboard.id == id)
    }

    fn moodboard_index(&self, id: &str) -> Result<usize> {
        self.moodboards
            .iter()
            .position(|loaded| loaded.moodboard.id == id)
            .ok_or(AppError::NotFound)
    }

    /// Index of the moodboard that holds an image.
    fn image_moodboard(&self, id: &str) -> Result<usize> {
        self.moodboards
            .iter()
            .position(|loaded| loaded.moodboard.images.iter().any(|image| image.id == id))
            .ok_or(AppError::NotFound)
    }

    /// The moodboard's file as it is now. A moodboard that is gone is dropped.
    fn fresh_moodboard(&mut self, m: usize) -> Result<MoodboardFile> {
        let dir = self.moodboards[m].dir.clone();
        match self.vault.read_moodboard(&dir)? {
            Some(file) if file.id == self.moodboards[m].moodboard.id => {
                self.moodboards[m].moodboard = file.clone();
                Ok(file)
            }
            _ => {
                self.moodboards.remove(m);
                Err(AppError::NotFound)
            }
        }
    }

    /// The moodboard holding an image, its fresh file and the image's index.
    fn fresh_image(&mut self, id: &str) -> Result<(usize, MoodboardFile, usize)> {
        let m = self.image_moodboard(id)?;
        let file = self.fresh_moodboard(m)?;
        let i = file
            .images
            .iter()
            .position(|image| image.id == id)
            .ok_or(AppError::NotFound)?;
        Ok((m, file, i))
    }

    fn save_moodboard(&mut self, m: usize, file: MoodboardFile) -> Result<()> {
        self.vault.write_moodboard(&self.moodboards[m].dir, &file)?;
        self.moodboards[m].moodboard = file;
        self.moodboards.sort_by(|a, b| {
            (&a.moodboard.position, &a.moodboard.id).cmp(&(&b.moodboard.position, &b.moodboard.id))
        });
        Ok(())
    }

    pub fn create_moodboard(&mut self, name: &str) -> Result<Board> {
        let file = MoodboardFile {
            id: new_id(),
            name: require_text(name)?,
            position: position::key_after_last(&self.sidebar_keys(""))?,
            created_at: now(),
            images: Vec::new(),
            extra: Default::default(),
        };
        let dir = self.vault.create_moodboard(&file)?;
        let board = board_of(&file);
        self.moodboards.push(LoadedMoodboard {
            dir,
            moodboard: file,
        });
        Ok(board)
    }

    pub fn moodboard_view(&self, id: &str) -> Result<MoodboardView> {
        let file = &self.moodboards[self.moodboard_index(id)?].moodboard;
        Ok(MoodboardView {
            board: board_of(file),
            images: file.images.iter().map(view_of).collect(),
        })
    }

    pub(super) fn rename_moodboard(&mut self, id: &str, name: &str) -> Result<()> {
        let name = require_text(name)?;
        let m = self.moodboard_index(id)?;
        let mut file = self.fresh_moodboard(m)?;
        file.name = name;
        self.save_moodboard(m, file)
    }

    pub(super) fn reorder_moodboard(&mut self, id: &str, index: usize) -> Result<()> {
        let m = self.moodboard_index(id)?;
        let mut file = self.fresh_moodboard(m)?;
        file.position = position::key_at(&self.sidebar_keys(id), index)?;
        self.save_moodboard(m, file)
    }

    pub(super) fn delete_moodboard(&mut self, id: &str) -> Result<()> {
        let m = self.moodboard_index(id)?;
        let trash = self.vault.trash_board(&self.moodboards[m].dir)?;
        let moodboard = self.moodboards.remove(m);
        self.trash
            .moodboards
            .push(TrashedMoodboard { moodboard, trash });
        Ok(())
    }

    pub(super) fn restore_moodboard(&mut self, id: &str) -> Result<()> {
        let t = self
            .trash
            .moodboards
            .iter()
            .position(|trashed| trashed.moodboard.moodboard.id == id)
            .ok_or(AppError::NotFound)?;
        let trashed = &self.trash.moodboards[t];
        let dir = self
            .vault
            .restore_board(&trashed.trash, &trashed.moodboard.moodboard.name)?;
        let TrashedMoodboard { mut moodboard, .. } = self.trash.moodboards.remove(t);
        moodboard.dir = dir;
        let wanted = moodboard.moodboard.position.clone();
        let key = position::free_key(&self.sidebar_keys(id), &wanted)?;
        self.moodboards.push(moodboard);
        let m = self.moodboard_index(id)?;
        let mut file = self.fresh_moodboard(m)?;
        file.position = key;
        self.save_moodboard(m, file)
    }

    /// Stores an image and places it on top of the moodboard, its top left
    /// corner at `x`, `y`.
    pub fn add_image(
        &mut self,
        moodboard_id: &str,
        bytes: &[u8],
        x: f64,
        y: f64,
    ) -> Result<MoodboardImage> {
        if bytes.len() > MAX_BYTES {
            return Err(AppError::Validation("image.too_large"));
        }
        let info = image::inspect(bytes).ok_or(AppError::Validation("image.unsupported"))?;
        let m = self.moodboard_index(moodboard_id)?;
        let mut file = self.fresh_moodboard(m)?;
        let dir = self.moodboards[m].dir.clone();
        let stored = self.vault.write_image(&dir, bytes, info.extension)?;
        let (width, height) = initial_size(info.width, info.height);
        let entry = ImageEntry {
            id: new_id(),
            file: stored.clone(),
            x: round(x.max(0.0)),
            y: round(y.max(0.0)),
            width,
            height,
            extra: Default::default(),
        };
        let image = view_of(&entry);
        file.images.push(entry);
        if let Err(error) = self.save_moodboard(m, file) {
            let _ = self.vault.remove_image(&dir, &stored);
            return Err(error);
        }
        Ok(image)
    }

    /// Like `add_image`, for a file the user picked or dropped.
    pub fn add_image_file(
        &mut self,
        moodboard_id: &str,
        path: &Path,
        x: f64,
        y: f64,
    ) -> Result<MoodboardImage> {
        let unreadable = |_| AppError::Validation("image.unreadable");
        let size = fs::metadata(path).map_err(unreadable)?.len();
        if size > MAX_BYTES as u64 {
            return Err(AppError::Validation("image.too_large"));
        }
        let bytes = fs::read(path).map_err(unreadable)?;
        self.add_image(moodboard_id, &bytes, x, y)
    }

    /// Moves or resizes an image. Images stay inside the moodboard (no
    /// negative coordinates) and never get smaller than `MIN_IMAGE_SIZE`.
    pub fn update_image(&mut self, id: &str, patch: &ImagePatch) -> Result<()> {
        let (m, mut file, i) = self.fresh_image(id)?;
        let image = &mut file.images[i];
        let set = |target: &mut f64, value: Option<f64>, min: f64| {
            if let Some(value) = value.filter(|value| value.is_finite()) {
                *target = round(value.max(min));
            }
        };
        set(&mut image.x, patch.x, 0.0);
        set(&mut image.y, patch.y, 0.0);
        set(&mut image.width, patch.width, MIN_IMAGE_SIZE);
        set(&mut image.height, patch.height, MIN_IMAGE_SIZE);
        self.save_moodboard(m, file)
    }

    /// Moves an image to another layer: 0 is the back, past the end the front.
    pub fn reorder_image(&mut self, id: &str, index: usize) -> Result<()> {
        let (m, mut file, i) = self.fresh_image(id)?;
        let image = file.images.remove(i);
        let index = index.min(file.images.len());
        file.images.insert(index, image);
        self.save_moodboard(m, file)
    }

    /// Removes an image from its moodboard and hides its file, so it can be
    /// restored until the vault is opened next.
    pub fn delete_image(&mut self, id: &str) -> Result<()> {
        let (m, mut file, i) = self.fresh_image(id)?;
        let image = file.images.remove(i);
        let moodboard_id = file.id.clone();
        self.save_moodboard(m, file)?;
        let m = self.moodboard_index(&moodboard_id)?;
        let dir = self.moodboards[m].dir.clone();
        self.vault.trash_image(&dir, &image.file)?;
        self.trash.images.push(TrashedImage {
            moodboard_id,
            index: i,
            image,
        });
        Ok(())
    }

    pub fn restore_image(&mut self, id: &str) -> Result<()> {
        let t = self
            .trash
            .images
            .iter()
            .position(|trashed| trashed.image.id == id)
            .ok_or(AppError::NotFound)?;
        let m = self.moodboard_index(&self.trash.images[t].moodboard_id)?;
        let mut file = self.fresh_moodboard(m)?;
        let TrashedImage { index, image, .. } = self.trash.images.remove(t);
        let dir = self.moodboards[m].dir.clone();
        self.vault.restore_image(&dir, &image.file)?;
        let index = index.min(file.images.len());
        file.images.insert(index, image);
        self.save_moodboard(m, file)
    }

    /// Where an image of a moodboard lies, for showing it. Only files the
    /// moodboard refers to are served, nothing else in the vault.
    pub fn image_path(&self, moodboard_id: &str, file: &str) -> Result<PathBuf> {
        let loaded = &self.moodboards[self.moodboard_index(moodboard_id)?];
        if !loaded
            .moodboard
            .images
            .iter()
            .any(|image| image.file == file)
        {
            return Err(AppError::NotFound);
        }
        Ok(self.vault.image_path(&loaded.dir, file))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::super::image::tests::png_bytes;
    use super::*;
    use crate::vault::Vault;
    use crate::vault::tests::TestDir;

    fn open(dir: &TestDir) -> VaultStore {
        VaultStore::load(Vault::open(dir.path()).unwrap()).unwrap()
    }

    fn columns() -> Vec<String> {
        vec!["Offen".into(), "Erledigt".into()]
    }

    fn ids(store: &VaultStore, moodboard_id: &str) -> Vec<Id> {
        let view = store.moodboard_view(moodboard_id).unwrap();
        view.images.into_iter().map(|image| image.id).collect()
    }

    fn image_file(dir: &TestDir, folder: &str, image: &MoodboardImage) -> PathBuf {
        dir.path().join(folder).join("images").join(&image.file)
    }

    #[test]
    fn moodboards_share_the_sidebar_order_with_boards() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let work = store.create_board("Arbeit", &columns()).unwrap();
        let ideas = store.create_moodboard(" Ideen ").unwrap();
        let home = store.create_board("Privat", &columns()).unwrap();
        assert_eq!(ideas.name, "Ideen");
        assert_eq!(ideas.kind, BoardKind::Moodboard);
        assert!(dir.path().join("Ideen/moodboard.json").is_file());
        assert!(dir.path().join("Ideen/images").is_dir());

        let order = |store: &VaultStore| -> Vec<(String, BoardKind)> {
            store
                .list_boards()
                .into_iter()
                .map(|board| (board.name, board.kind))
                .collect()
        };
        assert_eq!(
            order(&store),
            [
                ("Arbeit".into(), BoardKind::Kanban),
                ("Ideen".into(), BoardKind::Moodboard),
                ("Privat".into(), BoardKind::Kanban),
            ]
        );

        store.reorder_board(&ideas.id, 0).unwrap();
        store.reorder_board(&home.id, 1).unwrap();
        store.rename_board(&ideas.id, "Inspiration").unwrap();
        let expected = [
            ("Inspiration".to_owned(), BoardKind::Moodboard),
            ("Privat".to_owned(), BoardKind::Kanban),
            ("Arbeit".to_owned(), BoardKind::Kanban),
        ];
        assert_eq!(order(&store), expected);
        assert_eq!(order(&open(&dir)), expected);

        // A moodboard is no board with columns, and the other way round.
        assert_eq!(store.view(&ideas.id).unwrap_err().code(), "not_found");
        assert_eq!(
            store.moodboard_view(&work.id).unwrap_err().code(),
            "not_found"
        );
        assert_eq!(
            store.create_moodboard("  ").unwrap_err().code(),
            "validation.empty"
        );
    }

    #[test]
    fn deleted_moodboards_come_back_with_their_images() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let image = store
            .add_image(&ideas.id, &png_bytes(100, 50), 0.0, 0.0)
            .unwrap();

        store.delete_board(&ideas.id).unwrap();
        assert!(store.list_boards().is_empty());
        assert!(!dir.path().join("Ideen").exists());

        store.restore_board(&ideas.id).unwrap();
        assert_eq!(store.list_boards()[0].id, ideas.id);
        assert!(image_file(&dir, "Ideen", &image).is_file());
        assert_eq!(ids(&store, &ideas.id), [image.id]);
    }

    #[test]
    fn images_are_stored_sized_and_placed_on_top() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();

        let big = store
            .add_image(&ideas.id, &png_bytes(1200, 800), 40.0, -5.0)
            .unwrap();
        assert_eq!((big.x, big.y), (40.0, 0.0));
        assert_eq!((big.width, big.height), (360.0, 240.0));
        assert!(big.file.ends_with(".png"));
        assert_eq!(
            fs::read(image_file(&dir, "Ideen", &big)).unwrap(),
            png_bytes(1200, 800)
        );

        let small = store
            .add_image(&ideas.id, &png_bytes(20, 30), 0.0, 0.0)
            .unwrap();
        assert_eq!((small.width, small.height), (20.0, 30.0));
        assert_eq!(ids(&store, &ideas.id), [big.id.clone(), small.id.clone()]);
        assert_eq!(
            open(&dir).moodboard_view(&ideas.id).unwrap().images.len(),
            2
        );

        let code = |result: Result<MoodboardImage>| result.unwrap_err().code();
        assert_eq!(
            code(store.add_image(&ideas.id, b"%PDF-1.7", 0.0, 0.0)),
            "image.unsupported"
        );
        assert_eq!(
            code(store.add_image("fehlt", &png_bytes(10, 10), 0.0, 0.0)),
            "not_found"
        );
        assert_eq!(
            code(store.add_image_file(&ideas.id, &dir.path().join("fehlt.png"), 0.0, 0.0)),
            "image.unreadable"
        );
        let on_disk = dir.path().join("Foto.png");
        fs::write(&on_disk, png_bytes(90, 90)).unwrap();
        let picked = store.add_image_file(&ideas.id, &on_disk, 1.0, 2.0).unwrap();
        assert_eq!((picked.width, picked.height), (90.0, 90.0));
        // Rejected files leave nothing behind.
        let stored = fs::read_dir(dir.path().join("Ideen/images"))
            .unwrap()
            .count();
        assert_eq!(stored, 3);
    }

    #[test]
    fn images_are_moved_resized_and_layered() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let add = |store: &mut VaultStore| {
            store
                .add_image(&ideas.id, &png_bytes(100, 100), 0.0, 0.0)
                .unwrap()
                .id
        };
        let (a, b, c) = (add(&mut store), add(&mut store), add(&mut store));

        let patch = ImagePatch {
            x: Some(120.456),
            y: Some(-30.0),
            width: Some(4.0),
            height: None,
        };
        store.update_image(&a, &patch).unwrap();
        let moved = &open(&dir).moodboard_view(&ideas.id).unwrap().images[0];
        assert_eq!(
            (moved.x, moved.y, moved.width, moved.height),
            (120.46, 0.0, MIN_IMAGE_SIZE, 100.0)
        );

        store.reorder_image(&a, 99).unwrap();
        assert_eq!(ids(&store, &ideas.id), [b.clone(), c.clone(), a.clone()]);
        store.reorder_image(&c, 0).unwrap();
        assert_eq!(ids(&store, &ideas.id), [c.clone(), b.clone(), a.clone()]);
        assert_eq!(ids(&open(&dir), &ideas.id), [c, b, a]);
        assert_eq!(
            store.update_image("fehlt", &patch).unwrap_err().code(),
            "not_found"
        );
    }

    #[test]
    fn deleted_images_return_to_their_layer() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let mut add = || {
            store
                .add_image(&ideas.id, &png_bytes(100, 100), 0.0, 0.0)
                .unwrap()
        };
        let (a, b, c) = (add(), add(), add());

        store.delete_image(&b.id).unwrap();
        assert_eq!(ids(&store, &ideas.id), [a.id.clone(), c.id.clone()]);
        assert!(!image_file(&dir, "Ideen", &b).exists());
        assert!(store.image_path(&ideas.id, &b.file).is_err());

        store.restore_image(&b.id).unwrap();
        assert_eq!(ids(&store, &ideas.id), [a.id, b.id.clone(), c.id]);
        assert!(image_file(&dir, "Ideen", &b).is_file());

        // Reopening the vault removes the hidden file for good.
        store.delete_image(&b.id).unwrap();
        open(&dir);
        let names: Vec<_> = fs::read_dir(dir.path().join("Ideen/images"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names.len(), 2, "{names:?}");
    }

    #[test]
    fn only_images_of_the_moodboard_are_served() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let image = store
            .add_image(&ideas.id, &png_bytes(10, 10), 0.0, 0.0)
            .unwrap();
        assert_eq!(
            store.image_path(&ideas.id, &image.file).unwrap(),
            image_file(&dir, "Ideen", &image)
        );
        assert!(store.image_path(&ideas.id, "moodboard.json").is_err());
        assert!(store.image_path(&ideas.id, "../moodboard.json").is_err());
        assert!(store.image_path("fehlt", &image.file).is_err());
    }

    #[test]
    fn a_missing_image_file_does_not_block_deleting_its_place() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let image = store
            .add_image(&ideas.id, &png_bytes(10, 10), 0.0, 0.0)
            .unwrap();
        fs::remove_file(image_file(&dir, "Ideen", &image)).unwrap();
        store.delete_image(&image.id).unwrap();
        store.restore_image(&image.id).unwrap();
        assert_eq!(ids(&store, &ideas.id), [image.id]);
    }

    #[test]
    fn changes_build_on_the_file_as_others_left_it() {
        let dir = TestDir::new();
        let mut store = open(&dir);
        let ideas = store.create_moodboard("Ideen").unwrap();
        let a = store
            .add_image(&ideas.id, &png_bytes(10, 10), 0.0, 0.0)
            .unwrap();

        // Someone else renames the moodboard meanwhile.
        let path = dir.path().join("Ideen/moodboard.json");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.replace("\"Ideen\"", "\"Andere\"")).unwrap();

        let patch = ImagePatch {
            x: Some(50.0),
            ..Default::default()
        };
        store.update_image(&a.id, &patch).unwrap();
        let view = open(&dir).moodboard_view(&ideas.id).unwrap();
        assert_eq!(view.board.name, "Andere");
        assert_eq!(view.images[0].x, 50.0);
    }
}
