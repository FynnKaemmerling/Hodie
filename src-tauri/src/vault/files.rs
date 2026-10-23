//! File system helpers: atomic writes, directory listings and folder names
//! that are valid on Windows, macOS and Linux.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

const MAX_DIR_NAME_CHARS: usize = 64;
const FORBIDDEN_CHARS: &str = r#"<>:"/\|?*"#;
const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Writes to a hidden temporary file next to `path` and renames it over the
/// target, so readers (and sync clients) never see a half-written file.
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(AppError::Internal("path without file name"));
    };
    let temp = dir.join(format!(".{}.tmp", name.to_string_lossy()));
    let result = File::create(&temp)
        .and_then(|mut file| {
            file.write_all(content.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&temp, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    Ok(result?)
}

pub fn is_temp_file(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(".tmp")
}

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// Entries of `dir` sorted by name, without hidden ones (leading dot) and
/// names that are not valid UTF-8.
pub fn visible_entries(dir: &Path) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        entries.push(Entry {
            name,
            path: entry.path(),
            is_dir: entry.file_type()?.is_dir(),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Removes leftovers of interrupted `write_atomic` calls in `dir`.
pub fn remove_temp_files(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().to_str().is_some_and(is_temp_file) && entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

/// A folder name for a new board: derived from its name, valid on every
/// system and unique among `taken` ignoring case (macOS and Windows do too).
pub fn board_dir_name(name: &str, taken: &[String]) -> String {
    let replaced: String = name
        .chars()
        .map(|c| {
            if c.is_control() || FORBIDDEN_CHARS.contains(c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    // Leading dots would hide the folder, trailing dots and spaces are
    // dropped by Windows.
    let trim = |text: &str| {
        text.trim_matches(|c: char| c.is_whitespace() || c == '.')
            .to_owned()
    };
    let mut base = trim(
        &trim(&replaced)
            .chars()
            .take(MAX_DIR_NAME_CHARS)
            .collect::<String>(),
    );
    if base.is_empty() {
        base = "Board".into();
    }
    if is_reserved(&base) {
        base.push('-');
    }

    let taken: Vec<String> = taken.iter().map(|name| name.to_lowercase()).collect();
    let mut candidate = base.clone();
    let mut counter = 2;
    while taken.contains(&candidate.to_lowercase()) {
        candidate = format!("{base} {counter}");
        counter += 1;
    }
    candidate
}

/// Windows reserves these names even with an extension (`CON.txt`).
fn is_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).trim_end();
    RESERVED_NAMES
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(stem))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::tests::TestDir;

    #[test]
    fn board_dir_names_are_valid_everywhere() {
        let none: &[String] = &[];
        assert_eq!(board_dir_name("Arbeit", none), "Arbeit");
        assert_eq!(board_dir_name("  Haus & Garten  ", none), "Haus & Garten");
        assert_eq!(board_dir_name("Q1/Q2: Ziele?", none), "Q1-Q2- Ziele-");
        assert_eq!(board_dir_name("Tab\there", none), "Tab-here");
        assert_eq!(board_dir_name("..versteckt..", none), "versteckt");
        assert_eq!(board_dir_name(" . ", none), "Board");
        assert_eq!(board_dir_name("con", none), "con-");
        assert_eq!(board_dir_name("LPT1.notes", none), "LPT1.notes-");
        assert_eq!(board_dir_name("Console", none), "Console");
        assert_eq!(board_dir_name("Grüße 🎉", none), "Grüße 🎉");
    }

    #[test]
    fn board_dir_names_are_shortened_without_trailing_dots() {
        let long = format!("{}. Ende", "ä".repeat(63));
        assert_eq!(board_dir_name(&long, &[]), "ä".repeat(63));
        assert_eq!(board_dir_name(&"x".repeat(100), &[]).chars().count(), 64);
    }

    #[test]
    fn board_dir_names_are_unique_ignoring_case() {
        let taken = vec!["arbeit".to_owned(), "Arbeit 2".to_owned()];
        assert_eq!(board_dir_name("Arbeit", &taken), "Arbeit 3");
        assert_eq!(board_dir_name("Privat", &taken), "Privat");
    }

    #[test]
    fn atomic_write_replaces_the_file_and_leaves_nothing_behind() {
        let dir = TestDir::new();
        let path = dir.path().join("board.json");
        write_atomic(&path, "eins").unwrap();
        write_atomic(&path, "zwei").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "zwei");
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["board.json"]);
    }

    #[test]
    fn listing_skips_hidden_entries_and_removes_temp_files() {
        let dir = TestDir::new();
        fs::write(dir.path().join("b.md"), "").unwrap();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join(".DS_Store"), "").unwrap();
        fs::write(dir.path().join(".a.md.tmp"), "").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();

        remove_temp_files(dir.path()).unwrap();
        assert!(!dir.path().join(".a.md.tmp").exists());
        assert!(dir.path().join(".DS_Store").exists());

        let entries = visible_entries(dir.path()).unwrap();
        let names: Vec<_> = entries
            .iter()
            .map(|e| (e.name.as_str(), e.is_dir))
            .collect();
        assert_eq!(names, [("a.md", false), ("b.md", false), ("sub", true)]);
    }
}
