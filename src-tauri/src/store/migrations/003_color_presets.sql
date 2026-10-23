-- Colour presets the user created. The built-in ones live in the frontend.
-- Each holds an accent and a base colour (#rrggbb) for light and dark mode.
CREATE TABLE color_presets (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    light_accent TEXT NOT NULL,
    light_base   TEXT NOT NULL,
    dark_accent  TEXT NOT NULL,
    dark_base    TEXT NOT NULL
);
