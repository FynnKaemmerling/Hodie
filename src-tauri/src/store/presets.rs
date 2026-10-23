//! Colour presets created by the user, and which preset is active.

use rusqlite::{Connection, Row, params};

use super::{expect_changed, require_text, settings};
use crate::error::{AppError, Result};
use crate::model::{BUILTIN_PRESETS, ColorPreset, ColorPresetPatch, Id, PresetColors};

const CHOICE_KEY: &str = "color_preset";
const CUSTOM_PREFIX: &str = "custom:";

/// Accepts `#rrggbb` and returns it in lower case.
fn require_hex(color: &str) -> Result<String> {
    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if valid {
        Ok(color.to_ascii_lowercase())
    } else {
        Err(AppError::Validation("preset.invalid_color"))
    }
}

fn require_colors(colors: &PresetColors) -> Result<PresetColors> {
    Ok(PresetColors {
        light_accent: require_hex(&colors.light_accent)?,
        light_base: require_hex(&colors.light_base)?,
        dark_accent: require_hex(&colors.dark_accent)?,
        dark_base: require_hex(&colors.dark_base)?,
    })
}

fn from_row(row: &Row) -> rusqlite::Result<ColorPreset> {
    Ok(ColorPreset {
        id: row.get(0)?,
        name: row.get(1)?,
        colors: PresetColors {
            light_accent: row.get(2)?,
            light_base: row.get(3)?,
            dark_accent: row.get(4)?,
            dark_base: row.get(5)?,
        },
    })
}

pub fn list(conn: &Connection) -> Result<Vec<ColorPreset>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, light_accent, light_base, dark_accent, dark_base
         FROM color_presets ORDER BY id",
    )?;
    let presets = stmt
        .query_map([], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(presets)
}

pub fn create(conn: &Connection, name: &str, colors: &PresetColors) -> Result<ColorPreset> {
    let name = require_text(name)?;
    let colors = require_colors(colors)?;
    conn.execute(
        "INSERT INTO color_presets (name, light_accent, light_base, dark_accent, dark_base)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            name,
            colors.light_accent,
            colors.light_base,
            colors.dark_accent,
            colors.dark_base
        ],
    )?;
    Ok(ColorPreset {
        id: conn.last_insert_rowid(),
        name,
        colors,
    })
}

pub fn update(conn: &mut Connection, id: Id, patch: &ColorPresetPatch) -> Result<()> {
    let name = patch.name.as_deref().map(require_text).transpose()?;
    let colors = patch.colors.as_ref().map(require_colors).transpose()?;
    let tx = conn.transaction()?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM color_presets WHERE id = ?1)",
        [id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound);
    }
    if let Some(name) = name {
        tx.execute(
            "UPDATE color_presets SET name = ?2 WHERE id = ?1",
            params![id, name],
        )?;
    }
    if let Some(colors) = colors {
        tx.execute(
            "UPDATE color_presets
             SET light_accent = ?2, light_base = ?3, dark_accent = ?4, dark_base = ?5
             WHERE id = ?1",
            params![
                id,
                colors.light_accent,
                colors.light_base,
                colors.dark_accent,
                colors.dark_base
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Deleting the active preset switches back to the default one.
pub fn delete(conn: &mut Connection, id: Id) -> Result<()> {
    let tx = conn.transaction()?;
    expect_changed(tx.execute("DELETE FROM color_presets WHERE id = ?1", [id])?)?;
    if settings::get(&tx, CHOICE_KEY)?.as_deref() == Some(&format!("{CUSTOM_PREFIX}{id}")) {
        settings::set(&tx, CHOICE_KEY, BUILTIN_PRESETS[0])?;
    }
    tx.commit()?;
    Ok(())
}

/// The active preset: a built-in name like "ocean" or "custom:<id>".
pub fn choice(conn: &Connection) -> Result<String> {
    Ok(settings::get(conn, CHOICE_KEY)?.unwrap_or_else(|| BUILTIN_PRESETS[0].to_owned()))
}

pub fn set_choice(conn: &Connection, choice: &str) -> Result<()> {
    if !BUILTIN_PRESETS.contains(&choice) {
        let id: Id = choice
            .strip_prefix(CUSTOM_PREFIX)
            .and_then(|id| id.parse().ok())
            .ok_or(AppError::NotFound)?;
        let exists: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM color_presets WHERE id = ?1)",
            [id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(AppError::NotFound);
        }
    }
    settings::set(conn, CHOICE_KEY, choice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::open_in_memory;

    fn colors() -> PresetColors {
        PresetColors {
            light_accent: "#1C6E8C".into(),
            light_base: "#f2f6f8".into(),
            dark_accent: "#6cc4e0".into(),
            dark_base: "#171c20".into(),
        }
    }

    #[test]
    fn create_list_update_delete() {
        let mut conn = open_in_memory();
        let preset = create(&conn, " Sea ", &colors()).unwrap();
        assert_eq!(preset.name, "Sea");
        assert_eq!(preset.colors.light_accent, "#1c6e8c");

        update(
            &mut conn,
            preset.id,
            &ColorPresetPatch {
                name: Some("Ocean".into()),
                colors: Some(PresetColors {
                    dark_base: "#000000".into(),
                    ..colors()
                }),
            },
        )
        .unwrap();
        let listed = &list(&conn).unwrap()[0];
        assert_eq!(listed.name, "Ocean");
        assert_eq!(listed.colors.dark_base, "#000000");

        delete(&mut conn, preset.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
        assert!(matches!(
            delete(&mut conn, preset.id),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn rejects_invalid_input() {
        let mut conn = open_in_memory();
        for bad in ["red", "#12345", "#1234567", "#gggggg", "123456#"] {
            let result = create(
                &conn,
                "Bad",
                &PresetColors {
                    light_accent: bad.into(),
                    ..colors()
                },
            );
            assert!(matches!(
                result,
                Err(AppError::Validation("preset.invalid_color"))
            ));
        }
        assert!(matches!(
            create(&conn, "  ", &colors()),
            Err(AppError::Validation("validation.empty"))
        ));
        assert!(matches!(
            update(&mut conn, 999, &ColorPresetPatch::default()),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn choice_defaults_validates_and_falls_back() {
        let mut conn = open_in_memory();
        assert_eq!(choice(&conn).unwrap(), "hodie");

        set_choice(&conn, "forest").unwrap();
        assert_eq!(choice(&conn).unwrap(), "forest");
        assert!(matches!(set_choice(&conn, "neon"), Err(AppError::NotFound)));
        assert!(matches!(
            set_choice(&conn, "custom:999"),
            Err(AppError::NotFound)
        ));

        let preset = create(&conn, "Mine", &colors()).unwrap();
        let mine = format!("custom:{}", preset.id);
        set_choice(&conn, &mine).unwrap();
        assert_eq!(choice(&conn).unwrap(), mine);

        delete(&mut conn, preset.id).unwrap();
        assert_eq!(choice(&conn).unwrap(), "hodie");
    }
}
