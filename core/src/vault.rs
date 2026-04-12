use rusqlite::Connection;
use thiserror::Error;

use crate::crypto;
use crate::db;
use crate::models::{Entry, EntryUpdate, NewEntry};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault file not found or salt file missing: {0}")]
    SaltFileMissing(String),

    #[error("Corrupt salt file")]
    SaltFileCorrupt,

    #[error("Wrong master password or corrupt vault")]
    WrongPassword,

    #[error("Entry not found: {0}")]
    EntryNotFound(i64),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database layer error: {0}")]
    Db(#[from] db::DbError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// High-level handle to an open, authenticated vault.
#[derive(Debug)]
pub struct Vault {
    conn: Connection,
}

impl Vault {
    /// Creates a new vault file at `path` protected by `master_password`.
    /// Writes the 16-byte salt to `<path>.salt`.
    pub fn create(path: &str, master_password: &str) -> Result<Self, VaultError> {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::open(path, &key)?;
        db::init_schema(&conn)?;

        // Write salt to companion file — Argon2 salt is not secret
        let salt_path = format!("{}.salt", path);
        std::fs::write(&salt_path, salt)?;

        // Restrict permissions to owner-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&salt_path, perms)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(Self { conn })
    }

    /// Opens an existing vault file.
    /// Reads the salt from `<path>.salt`, re-derives the key, and verifies with a probe query.
    pub fn open(path: &str, master_password: &str) -> Result<Self, VaultError> {
        let salt_path = format!("{}.salt", path);
        let salt_bytes =
            std::fs::read(&salt_path).map_err(|_| VaultError::SaltFileMissing(salt_path))?;
        let salt: [u8; 16] = salt_bytes
            .try_into()
            .map_err(|_| VaultError::SaltFileCorrupt)?;

        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::open(path, &key)?;

        // Probe to verify the key is correct
        conn.query_row("SELECT COUNT(*) FROM entries", [], |_| Ok(()))
            .map_err(|_| VaultError::WrongPassword)?;

        Ok(Self { conn })
    }

    // -------------------------------------------------------------------------
    // Entry CRUD
    // -------------------------------------------------------------------------

    /// Inserts a new entry and returns it with its generated id and timestamps.
    pub fn add_entry(&self, new: &NewEntry) -> Result<Entry, VaultError> {
        self.conn.execute(
            "INSERT INTO entries (title, username, password, url, notes, totp_secret, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                new.title,
                new.username,
                new.password,
                new.url,
                new.notes,
                new.totp_secret,
                new.category_id
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        let entry = self.get_entry(id)?;
        log_action(&self.conn, "create", Some(id), Some(&new.title))?;
        Ok(entry)
    }

    /// Fetches a single entry by id.
    pub fn get_entry(&self, id: i64) -> Result<Entry, VaultError> {
        let entry = self
            .conn
            .query_row(
                "SELECT id, title, username, password, url, notes, totp_secret,
                        category_id, favorite, created_at, updated_at
                 FROM entries WHERE id = ?1",
                rusqlite::params![id],
                row_to_entry,
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => VaultError::EntryNotFound(id),
                other => VaultError::Database(other),
            })?;
        log_action(&self.conn, "read", Some(id), None)?;
        Ok(entry)
    }

    /// Returns all entries ordered by title.
    pub fn list_entries(&self) -> Result<Vec<Entry>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, username, password, url, notes, totp_secret,
                    category_id, favorite, created_at, updated_at
             FROM entries ORDER BY title ASC",
        )?;
        let entries: Result<Vec<Entry>, rusqlite::Error> =
            stmt.query_map([], row_to_entry)?.collect();
        Ok(entries?)
    }

    /// Applies the non-None fields of `updates` to the entry with the given id.
    pub fn update_entry(&self, id: i64, mut updates: EntryUpdate) -> Result<Entry, VaultError> {
        // Verify entry exists first
        let existing = self.get_entry(id)?;

        let title = updates
            .title
            .take()
            .unwrap_or_else(|| existing.title.clone());
        let username = updates
            .username
            .take()
            .unwrap_or_else(|| existing.username.clone());
        let password = updates
            .password
            .take()
            .unwrap_or_else(|| existing.password.clone());
        let url = updates.url.take().unwrap_or_else(|| existing.url.clone());
        let notes = updates
            .notes
            .take()
            .unwrap_or_else(|| existing.notes.clone());
        let totp_secret = updates
            .totp_secret
            .take()
            .unwrap_or_else(|| existing.totp_secret.clone());
        let category_id = updates.category_id.take().unwrap_or(existing.category_id);
        let favorite = updates.favorite.take().unwrap_or(existing.favorite);

        let favorite_int: i64 = if favorite { 1 } else { 0 };
        self.conn.execute(
            "UPDATE entries
             SET title=?1, username=?2, password=?3, url=?4, notes=?5,
                 totp_secret=?6, category_id=?7, favorite=?8,
                 updated_at=unixepoch()
             WHERE id=?9",
            rusqlite::params![
                title,
                username,
                password,
                url,
                notes,
                totp_secret,
                category_id,
                favorite_int,
                id
            ],
        )?;
        let entry = self.get_entry(id)?;
        log_action(&self.conn, "update", Some(id), Some(&entry.title))?;
        Ok(entry)
    }

    /// Deletes the entry with the given id.
    pub fn delete_entry(&self, id: i64) -> Result<(), VaultError> {
        let rows = self
            .conn
            .execute("DELETE FROM entries WHERE id=?1", rusqlite::params![id])?;
        if rows == 0 {
            return Err(VaultError::EntryNotFound(id));
        }
        log_action(
            &self.conn,
            "delete",
            None,
            Some(&format!("entry_id={}", id)),
        )?;
        Ok(())
    }

    /// Searches entries whose title, username, url, or notes contain `query` (case-insensitive).
    pub fn search_entries(&self, query: &str) -> Result<Vec<Entry>, VaultError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, username, password, url, notes, totp_secret,
                    category_id, favorite, created_at, updated_at
             FROM entries
             WHERE title LIKE ?1 OR username LIKE ?1 OR url LIKE ?1 OR notes LIKE ?1
             ORDER BY title ASC",
        )?;
        let entries: Result<Vec<Entry>, rusqlite::Error> = stmt
            .query_map(rusqlite::params![pattern], row_to_entry)?
            .collect();
        Ok(entries?)
    }
}

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        title: row.get(1)?,
        username: row.get(2)?,
        password: row.get(3)?,
        url: row.get(4)?,
        notes: row.get(5)?,
        totp_secret: row.get(6)?,
        category_id: row.get(7)?,
        favorite: row.get::<_, i64>(8)? != 0,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

/// Writes a record to the audit_log table.
fn log_action(
    conn: &Connection,
    action: &str,
    entry_id: Option<i64>,
    detail: Option<&str>,
) -> Result<(), VaultError> {
    conn.execute(
        "INSERT INTO audit_log (action, entry_id, detail) VALUES (?1, ?2, ?3)",
        rusqlite::params![action, entry_id, detail],
    )?;
    Ok(())
}
