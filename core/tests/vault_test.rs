use sifr_core::{EntryUpdate, NewEntry, Vault, VaultError};
use tempfile::TempDir;

fn temp_vault_path(dir: &TempDir) -> String {
    dir.path().join("test.vault").to_str().unwrap().to_owned()
}

fn new_entry(
    title: &str,
    username: Option<&str>,
    password: Option<&str>,
    url: Option<&str>,
    notes: Option<&str>,
) -> NewEntry {
    NewEntry {
        title: title.into(),
        username: username.map(Into::into),
        password: password.map(Into::into),
        url: url.map(Into::into),
        notes: notes.map(Into::into),
        totp_secret: None,
        category_id: None,
    }
}

// ---------------------------------------------------------------------------
// Vault lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_vault_create_and_open() {
    let dir = TempDir::new().unwrap();
    let path = temp_vault_path(&dir);

    // Create vault
    let vault = Vault::create(&path, "correct-horse-battery-staple").unwrap();
    // Basic sanity: list entries on fresh vault returns empty vec
    let entries = vault.list_entries().unwrap();
    assert!(entries.is_empty());
    drop(vault);

    // Reopen with same password
    let vault2 = Vault::open(&path, "correct-horse-battery-staple").unwrap();
    let entries2 = vault2.list_entries().unwrap();
    assert!(entries2.is_empty());
}

#[test]
fn test_vault_wrong_password() {
    let dir = TempDir::new().unwrap();
    let path = temp_vault_path(&dir);

    Vault::create(&path, "right-password").unwrap();

    let result = Vault::open(&path, "wrong-password");
    assert!(
        matches!(result, Err(VaultError::WrongPassword)),
        "Expected WrongPassword, got {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// Entry CRUD
// ---------------------------------------------------------------------------

fn make_vault(dir: &TempDir) -> Vault {
    let path = temp_vault_path(dir);
    Vault::create(&path, "test-password").unwrap()
}

#[test]
fn test_entry_crud() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    // Add
    let ne = new_entry(
        "GitHub",
        Some("alice"),
        Some("s3cr3t"),
        Some("https://github.com"),
        Some("work account"),
    );
    let entry = vault.add_entry(&ne).unwrap();
    assert_eq!(entry.title, "GitHub");
    assert_eq!(entry.username.as_deref(), Some("alice"));
    assert_eq!(entry.password.as_deref(), Some("s3cr3t"));
    assert!(!entry.favorite);
    let id = entry.id;

    // Get
    let fetched = vault.get_entry(id).unwrap();
    assert_eq!(fetched.id, id);
    assert_eq!(fetched.title, "GitHub");

    // List
    let list = vault.list_entries().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, id);

    // Update
    let upd = EntryUpdate {
        title: Some("GitHub (work)".into()),
        username: None,
        password: None,
        url: None,
        notes: None,
        totp_secret: None,
        category_id: None,
        favorite: Some(true),
    };
    let updated = vault.update_entry(id, upd).unwrap();
    assert_eq!(updated.title, "GitHub (work)");
    assert!(updated.favorite);
    // Password unchanged
    assert_eq!(updated.password.as_deref(), Some("s3cr3t"));

    // Delete
    vault.delete_entry(id).unwrap();
    let list_after = vault.list_entries().unwrap();
    assert!(list_after.is_empty());

    // Get after delete should return EntryNotFound
    let result = vault.get_entry(id);
    assert!(
        matches!(result, Err(VaultError::EntryNotFound(_))),
        "Expected EntryNotFound, got {:?}",
        result
    );
}

#[test]
fn test_delete_nonexistent_entry() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);
    let result = vault.delete_entry(9999);
    assert!(matches!(result, Err(VaultError::EntryNotFound(9999))));
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[test]
fn test_entry_search() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    let e1 = new_entry(
        "GitHub",
        Some("alice"),
        Some("pw1"),
        Some("https://github.com"),
        None,
    );
    vault.add_entry(&e1).unwrap();

    let e2 = new_entry(
        "GitLab",
        Some("alice"),
        Some("pw2"),
        Some("https://gitlab.com"),
        None,
    );
    vault.add_entry(&e2).unwrap();

    let e3 = new_entry(
        "AWS Console",
        Some("ops@example.com"),
        Some("pw3"),
        Some("https://aws.amazon.com"),
        Some("production account"),
    );
    vault.add_entry(&e3).unwrap();

    // Search by title prefix
    let git_results = vault.search_entries("git").unwrap();
    assert_eq!(git_results.len(), 2);

    // Search by URL fragment
    let aws_results = vault.search_entries("amazon").unwrap();
    assert_eq!(aws_results.len(), 1);
    assert_eq!(aws_results[0].title, "AWS Console");

    // Search by notes
    let prod_results = vault.search_entries("production").unwrap();
    assert_eq!(prod_results.len(), 1);

    // Search with no matches
    let none_results = vault.search_entries("zzz-no-match").unwrap();
    assert!(none_results.is_empty());
}

// ---------------------------------------------------------------------------
// Password generator
// ---------------------------------------------------------------------------

#[test]
fn test_password_generator() {
    use sifr_core::crypto::generate_password;

    // Length check
    let pw = generate_password(20, true, true, true);
    assert_eq!(pw.len(), 20);

    // Lowercase only
    let pw_lower = generate_password(50, false, false, false);
    assert!(pw_lower.chars().all(|c| c.is_ascii_lowercase()));

    // With uppercase: at least some uppercase expected over 100 chars
    let pw_upper = generate_password(100, true, false, false);
    assert!(pw_upper.chars().any(|c| c.is_ascii_uppercase()));

    // With numbers
    let pw_num = generate_password(100, false, true, false);
    assert!(pw_num.chars().any(|c| c.is_ascii_digit()));

    // With symbols
    let symbols = "!@#$%^&*()-_=+[]{}|;:,.<>?";
    let pw_sym = generate_password(100, false, false, true);
    assert!(pw_sym.chars().any(|c| symbols.contains(c)));

    // Minimum length 1
    let pw_one = generate_password(1, false, false, false);
    assert_eq!(pw_one.len(), 1);
}

// ---------------------------------------------------------------------------
// Password length validation
// ---------------------------------------------------------------------------

#[test]
fn test_vault_create_short_password() {
    let dir = TempDir::new().unwrap();
    let path = temp_vault_path(&dir);
    let result = Vault::create(&path, "short");
    assert!(matches!(result, Err(VaultError::PasswordTooShort(8))));
}

#[test]
fn test_vault_create_minimum_password() {
    let dir = TempDir::new().unwrap();
    let path = temp_vault_path(&dir);
    let vault = Vault::create(&path, "exactly8");
    assert!(vault.is_ok());
}

// ---------------------------------------------------------------------------
// TOTP generation
// ---------------------------------------------------------------------------

#[test]
fn test_generate_totp_produces_six_digit_code() {
    use sifr_core::crypto::generate_totp;

    // Known valid base32 secret (decodes to "12345678901234567890")
    let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    let (code, _remaining) = generate_totp(secret).unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_generate_totp_seconds_remaining_in_range() {
    use sifr_core::crypto::generate_totp;

    let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    let (_code, remaining) = generate_totp(secret).unwrap();
    assert!((1..=30).contains(&remaining));
}

#[test]
fn test_generate_totp_invalid_base32_returns_error() {
    use sifr_core::crypto::generate_totp;

    let result = generate_totp("not-valid-base32!!!");
    assert!(result.is_err());
}

#[test]
fn test_vault_get_totp_code() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    let ne = NewEntry {
        title: "TOTP Test".into(),
        username: None,
        password: None,
        url: None,
        notes: None,
        totp_secret: Some("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".into()),
        category_id: None,
    };
    let entry = vault.add_entry(&ne).unwrap();

    let (code, remaining) = vault.get_totp_code(entry.id).unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
    assert!((1..=30).contains(&remaining));
}

#[test]
fn test_vault_get_totp_code_no_secret() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    let ne = new_entry("No TOTP", Some("alice"), Some("pw"), None, None);
    let entry = vault.add_entry(&ne).unwrap();

    let result = vault.get_totp_code(entry.id);
    assert!(matches!(result, Err(VaultError::NoTotpSecret(_))));
}
