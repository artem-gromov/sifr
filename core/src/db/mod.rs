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
/// The key is derived externally via `crypto::derive_key` and passed as raw bytes.
pub fn open(path: &str, key: &Zeroizing<[u8; 32]>) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    // cipher_page_size MUST be set before PRAGMA key for SQLCipher
    conn.execute_batch("PRAGMA cipher_page_size = 4096;")?;
    let pragma = hex_key_pragma(key);
    conn.execute_batch(&pragma)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Opens a new vault with a specific salt written into the SQLCipher header.
/// Must be called for new databases so our Argon2id salt is embedded in the file.
pub fn create(
    path: &str,
    key: &Zeroizing<[u8; 32]>,
    salt: &[u8; 16],
) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA cipher_page_size = 4096;")?;
    let key_pragma = hex_key_pragma(key);
    conn.execute_batch(&key_pragma)?;
    // Write our Argon2id salt into the SQLCipher file header (first 16 bytes)
    let salt_pragma = hex_salt_pragma(salt);
    conn.execute_batch(&salt_pragma)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// Initialises schema on a freshly created vault.
pub fn init_schema(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

/// Returns true if the schema has already been applied.
#[allow(dead_code)]
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

/// Builds the PRAGMA cipher_salt statement to embed our salt in the file header.
fn hex_salt_pragma(salt: &[u8; 16]) -> String {
    let hex: String = salt.iter().map(|b| format!("{:02x}", b)).collect();
    format!("PRAGMA cipher_salt = \"x'{}'\"", hex)
}

/// Applies pending schema migrations. Returns Ok if schema not yet initialized.
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let current = current_version(conn);
    if current < 1 {
        return Ok(()); // schema not initialized yet
    }
    // Future migrations go here:
    // if current < 2 { ... conn.execute_batch(...)?; set_version(conn, 2)?; }
    Ok(())
}

fn current_version(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}
