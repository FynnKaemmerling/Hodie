-- App settings as key/value pairs, e.g. the quick entry shortcut.
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) WITHOUT ROWID;
