-- Sifr vault schema
-- Applied once on vault creation via embedded migration

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version     INTEGER NOT NULL,
    applied_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Categories (Logins, Cards, Notes, etc.)
CREATE TABLE IF NOT EXISTS categories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    icon        TEXT,
    color       TEXT
);

-- Core password entries
CREATE TABLE IF NOT EXISTS entries (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT    NOT NULL,
    username     TEXT,
    password     TEXT,           -- stored encrypted by SQLCipher
    url          TEXT,
    notes        TEXT,
    totp_secret  TEXT,           -- base32 TOTP secret, encrypted
    category_id  INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    favorite     INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at   INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE
);

-- Entry ↔ Tag many-to-many
CREATE TABLE IF NOT EXISTS entry_tags (
    entry_id  INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id    INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);

-- Audit log (local, never synced)
CREATE TABLE IF NOT EXISTS audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action     TEXT    NOT NULL,  -- 'create' | 'read' | 'update' | 'delete' | 'unlock' | 'lock'
    entry_id   INTEGER REFERENCES entries(id) ON DELETE SET NULL,
    detail     TEXT,
    at         INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Seed default categories
INSERT OR IGNORE INTO categories (name, icon) VALUES
    ('Login',      '🔑'),
    ('Card',       '💳'),
    ('Secure Note','📝'),
    ('Identity',   '🪪');

INSERT INTO schema_version (version) VALUES (1);
