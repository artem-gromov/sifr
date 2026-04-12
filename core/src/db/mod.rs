use rusqlite::Connection;
use thiserror::Error;
use zeroize::Zeroizing;

const SCHEMA: &str = include_str!("schema.sql");

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
}

/// Opens (or creates) an encrypted SQLCipher vault file.
/// The key is derived externally via `crypto::derive_key` and passed as hex.
pub fn open(path: &str, key: &Zeroizing<[u8; 32]>) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    // cipher_page_size MUST be set before PRAGMA key for SQLCipher
    conn.execute_batch("PRAGMA cipher_page_size = 4096;")?;
    let pragma = hex_key_pragma(key);
    conn.execute_batch(&pragma)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// Initialises schema on a freshly created vault.
pub fn init_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

/// Returns true if the schema has already been applied.
pub fn is_initialised(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Formats the key as a hex string wrapped in `Zeroizing` so it is wiped on drop.
fn hex_key(key: &Zeroizing<[u8; 32]>) -> Zeroizing<String> {
    Zeroizing::new(key.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Builds the full PRAGMA key statement, also wrapped in `Zeroizing`.
fn hex_key_pragma(key: &Zeroizing<[u8; 32]>) -> Zeroizing<String> {
    let hex = hex_key(key);
    Zeroizing::new(format!("PRAGMA key = \"x'{}'\"", *hex))
}
