//! Which sound the Pomodoro timer plays when a phase ends. The sounds
//! themselves live in the frontend; the backend only keeps the chosen name.

use rusqlite::Connection;

use super::{require_text, settings};
use crate::error::Result;

const SOUND_KEY: &str = "pomodoro_sound";

/// The chosen sound, or `None` while the user has not picked one.
pub fn sound(conn: &Connection) -> Result<Option<String>> {
    settings::get(conn, SOUND_KEY)
}

pub fn set_sound(conn: &Connection, sound: &str) -> Result<()> {
    settings::set(conn, SOUND_KEY, &require_text(sound)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::store::open_in_memory;

    #[test]
    fn stores_the_chosen_sound() {
        let conn = open_in_memory();
        assert_eq!(sound(&conn).unwrap(), None);
        set_sound(&conn, " bell ").unwrap();
        assert_eq!(sound(&conn).unwrap().as_deref(), Some("bell"));
    }

    #[test]
    fn rejects_an_empty_name() {
        let conn = open_in_memory();
        assert!(matches!(
            set_sound(&conn, "  "),
            Err(AppError::Validation("validation.empty"))
        ));
    }
}
