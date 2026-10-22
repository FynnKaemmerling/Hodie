-- Timestamps are Unix milliseconds. `position` holds fractional-index keys
-- (see src/position.rs). `deleted_at` marks rows kept for undo; they are
-- purged on the next start.

CREATE TABLE boards (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL,
    position   TEXT    NOT NULL,
    created_at INTEGER NOT NULL,
    deleted_at INTEGER
);

CREATE TABLE board_columns (
    id         INTEGER PRIMARY KEY,
    board_id   INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    name       TEXT    NOT NULL,
    position   TEXT    NOT NULL,
    is_done    INTEGER NOT NULL DEFAULT 0 CHECK (is_done IN (0, 1)),
    created_at INTEGER NOT NULL,
    deleted_at INTEGER
);

CREATE INDEX board_columns_by_board ON board_columns (board_id, position);

-- At most one done column per board; the store keeps it at exactly one.
CREATE UNIQUE INDEX board_columns_one_done
    ON board_columns (board_id)
    WHERE is_done = 1 AND deleted_at IS NULL;

CREATE TABLE cards (
    id          INTEGER PRIMARY KEY,
    column_id   INTEGER NOT NULL REFERENCES board_columns (id) ON DELETE CASCADE,
    title       TEXT    NOT NULL,
    notes       TEXT    NOT NULL DEFAULT '',
    priority    INTEGER NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 3),
    position    TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    archived_at INTEGER,
    deleted_at  INTEGER
);

CREATE INDEX cards_by_column ON cards (column_id, position);

CREATE TABLE subtasks (
    id       INTEGER PRIMARY KEY,
    card_id  INTEGER NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
    title    TEXT    NOT NULL,
    done     INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    position TEXT    NOT NULL
);

CREATE INDEX subtasks_by_card ON subtasks (card_id, position);

CREATE TABLE tags (
    id    INTEGER PRIMARY KEY,
    name  TEXT NOT NULL UNIQUE COLLATE NOCASE,
    color TEXT NOT NULL
);

CREATE TABLE card_tags (
    card_id INTEGER NOT NULL REFERENCES cards (id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (card_id, tag_id)
) WITHOUT ROWID;

CREATE INDEX card_tags_by_tag ON card_tags (tag_id);
