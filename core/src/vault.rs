use rusqlite::Connection;
use thiserror::Error;

use crate::crypto;
use crate::db;
use crate::models::{Entry, EntryExport, EntryUpdate, NewEntry};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault file not found: {0}")]
    FileNotFound(String),

    #[error("Vault file too small to contain salt header")]
    FileTooSmall,

    #[error("Wrong master password or corrupt vault")]
    WrongPassword,

    #[error("Entry not found: {0}")]
    EntryNotFound(i64),

    #[error("Entry {0} has no TOTP secret")]
    NoTotpSecret(i64),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database layer error: {0}")]
    Db(#[from] db::DbError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),

    #[error("Master password too short (minimum {0} characters)")]
    PasswordTooShort(usize),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

const MIN_PASSWORD_LENGTH: usize = 8;

/// High-level handle to an open, authenticated vault.
#[derive(Debug)]
pub struct Vault {
    conn: Connection,
}

impl Vault {
    /// Creates a new vault at `path` protected by `master_password`.
    /// The Argon2id salt is stored in the SQLCipher file header (first 16 bytes).
    /// Single file — no companion `.salt` file needed.
    pub fn create(path: &str, master_password: &str) -> Result<Self, VaultError> {
        if master_password.len() < MIN_PASSWORD_LENGTH {
            return Err(VaultError::PasswordTooShort(MIN_PASSWORD_LENGTH));
        }
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::create(path, &key, &salt)?;
        db::init_schema(&conn)?;

        // Restrict permissions to owner-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(Self { conn })
    }

    /// Opens an existing vault file.
    /// Reads the Argon2id salt from the first 16 bytes of the SQLCipher file header,
    /// re-derives the key, and verifies with a probe query.
    pub fn open(path: &str, master_password: &str) -> Result<Self, VaultError> {
        Self::check_permissions(path)?;

        // Read the salt from the SQLCipher file header (first 16 bytes)
        let salt = Self::read_salt(path)?;

        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::open(path, &key)?;

        // Probe to verify the key is correct
        conn.query_row("SELECT COUNT(*) FROM entries", [], |_| Ok(()))
            .map_err(|_| VaultError::WrongPassword)?;

        Ok(Self { conn })
    }

    #[cfg(unix)]
    fn check_permissions(path: &str) -> Result<(), VaultError> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        let mode = metadata.permissions().mode();
        let readable_by_group_or_others = mode & 0o077;
        if readable_by_group_or_others != 0 {
            eprintln!(
                "Warning: Vault file '{}' has permissions {:o}. Consider running: chmod 600 '{}'",
                path, mode, path
            );
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn check_permissions(_path: &str) -> Result<(), VaultError> {
        Ok(())
    }

    /// Reads the 16-byte salt from the SQLCipher file header.
    fn read_salt(path: &str) -> Result<[u8; 16], VaultError> {
        use std::io::Read;
        let mut file =
            std::fs::File::open(path).map_err(|_| VaultError::FileNotFound(path.to_string()))?;
        let mut salt = [0u8; 16];
        file.read_exact(&mut salt)
            .map_err(|_| VaultError::FileTooSmall)?;
        Ok(salt)
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
        let entry = self.get_entry_internal(id)?;
        log_action(&self.conn, "create", Some(id), Some(&new.title))?;
        Ok(entry)
    }

    /// Fetches a single entry by id (internal, no audit log).
    fn get_entry_internal(&self, id: i64) -> Result<Entry, VaultError> {
        self.conn
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
            })
    }

    /// Fetches a single entry by id.
    pub fn get_entry(&self, id: i64) -> Result<Entry, VaultError> {
        let entry = self.get_entry_internal(id)?;
        log_action(&self.conn, "read", Some(id), None)?;
        Ok(entry)
    }

    /// Returns all entries ordered by favorite first, then by title.
    pub fn list_entries(&self) -> Result<Vec<Entry>, VaultError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, username, password, url, notes, totp_secret,
                    category_id, favorite, created_at, updated_at
             FROM entries ORDER BY favorite DESC, title ASC",
        )?;
        let entries: Result<Vec<Entry>, rusqlite::Error> =
            stmt.query_map([], row_to_entry)?.collect();
        Ok(entries?)
    }

    /// Applies the non-None fields of `updates` to the entry with the given id.
    pub fn update_entry(&self, id: i64, mut updates: EntryUpdate) -> Result<Entry, VaultError> {
        // Verify entry exists first
        let existing = self.get_entry_internal(id)?;

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
        let entry = self.get_entry_internal(id)?;
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

    /// Returns the current TOTP code and seconds remaining for an entry.
    pub fn get_totp_code(&self, entry_id: i64) -> Result<(String, u8), VaultError> {
        let entry = self.get_entry_internal(entry_id)?;
        let secret = entry
            .totp_secret
            .as_deref()
            .ok_or(VaultError::NoTotpSecret(entry_id))?;
        let (code, remaining) = crypto::generate_totp(secret)?;
        Ok((code, remaining))
    }

    /// Searches entries whose title, username, url, or notes contain `query` (case-insensitive).
    /// Results are ordered by favorite first, then by title.
    pub fn search_entries(&self, query: &str) -> Result<Vec<Entry>, VaultError> {
        let escaped: String = query
            .chars()
            .flat_map(|c| match c {
                '%' => vec!['\\', '%'],
                '_' => vec!['\\', '_'],
                '\\' => vec!['\\', '\\'],
                other => vec![other],
            })
            .collect();
        let pattern = format!("%{}%", escaped);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, username, password, url, notes, totp_secret,
                    category_id, favorite, created_at, updated_at
             FROM entries
             WHERE title LIKE ?1 ESCAPE '\\' OR username LIKE ?1 ESCAPE '\\' OR url LIKE ?1 ESCAPE '\\' OR notes LIKE ?1 ESCAPE '\\'
             ORDER BY favorite DESC, title ASC",
        )?;
        let entries: Result<Vec<Entry>, rusqlite::Error> = stmt
            .query_map(rusqlite::params![pattern], row_to_entry)?
            .collect();
        Ok(entries?)
    }

    // -------------------------------------------------------------------------
    // Import / Export
    // -------------------------------------------------------------------------

    /// Exports all entries as JSON.
    /// WARNING: passwords and TOTP secrets are included in plaintext.
    /// Only export to trusted locations.
    pub fn export_json(&self) -> Result<String, VaultError> {
        let entries = self.list_entries()?;
        let exports: Vec<EntryExport> = entries.iter().map(EntryExport::from).collect();
        let json = serde_json::to_string_pretty(&exports).map_err(|e| VaultError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        eprintln!("WARNING: Export contains plaintext passwords and TOTP secrets.");
        Ok(json)
    }

    /// Imports entries from CSV. Each row: title,username,password,url,notes,totp_secret,favorite
    /// Returns the number of entries imported.
    pub fn import_csv(&self, csv_data: &str) -> Result<usize, VaultError> {
        let mut count = 0;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(csv_data.as_bytes());
        for result in rdr.records() {
            let record = result.map_err(|e| {
                VaultError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?;
            let title = record.get(0).map(|s| s.trim()).unwrap_or("").to_string();
            if title.is_empty() {
                continue;
            }
            let new = NewEntry {
                title,
                username: record.get(1).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
                password: record.get(2).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
                url: record.get(3).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
                notes: record.get(4).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
                totp_secret: record.get(5).and_then(|s| {
                    let s = s.trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
                category_id: None,
            };
            self.add_entry(&new)?;
            count += 1;
        }
        Ok(count)
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
