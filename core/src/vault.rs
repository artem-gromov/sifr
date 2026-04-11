use anyhow::Result;
use rusqlite::Connection;

use crate::crypto;
use crate::db;

/// High-level handle to an open, authenticated vault.
pub struct Vault {
    pub conn: Connection,
}

impl Vault {
    /// Creates a new vault file at `path` protected by `master_password`.
    pub fn create(path: &str, master_password: &str) -> Result<Self> {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::open(path, &key)?;
        db::init_schema(&conn)?;
        // Store salt so we can re-derive the key on next open
        // Salt is stored in plaintext — that's correct: Argon2 salt is not secret
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _vault_meta (k TEXT PRIMARY KEY, v BLOB NOT NULL)",
            [],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO _vault_meta (k, v) VALUES ('salt', ?1)",
            rusqlite::params![salt.to_vec()],
        )?;
        Ok(Self { conn })
    }

    /// Opens an existing vault file.
    pub fn open(path: &str, master_password: &str) -> Result<Self> {
        // We need to open without a key first just to read the salt
        // SQLCipher won't let us read without key — salt is stored as first 16 bytes of file
        // Workaround: store salt in a separate sidecar file or derive it deterministically.
        // For now: store salt in a companion `.salt` file next to the vault.
        let salt_path = format!("{}.salt", path);
        let salt_bytes = std::fs::read(&salt_path)
            .map_err(|_| anyhow::anyhow!("Vault not found or salt file missing"))?;
        let salt: [u8; 16] = salt_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Corrupt salt file"))?;
        let key = crypto::derive_key(master_password, &salt)?;
        let conn = db::open(path, &key)?;
        // Verify the key is correct by probing a known table
        conn.query_row("SELECT COUNT(*) FROM entries", [], |_| Ok(()))
            .map_err(|_| anyhow::anyhow!("Wrong master password or corrupt vault"))?;
        Ok(Self { conn })
    }
}
