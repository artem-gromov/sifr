use std::fs;
use std::path::PathBuf;

/// Returns the sifr config directory (~/.config/sifr/).
fn config_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config").join("sifr"))
}

/// Returns the path to the "last vault" state file.
fn last_vault_file() -> Option<PathBuf> {
    Some(config_dir()?.join("last_vault"))
}

/// Reads the last-opened vault path, if it exists and the file is still present.
pub fn load_last_vault() -> Option<String> {
    let file = last_vault_file()?;
    let path = fs::read_to_string(file).ok()?.trim().to_string();
    if path.is_empty() {
        return None;
    }
    // Only return if the vault file actually exists
    if std::path::Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// Saves the vault path as the last-opened vault.
pub fn save_last_vault(vault_path: &str) {
    if vault_path.is_empty() {
        return;
    }
    if let Some(dir) = config_dir() {
        let _ = fs::create_dir_all(&dir);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        let file = dir.join("last_vault");
        let _ = fs::write(&file, vault_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o600));
        }
    }
}
