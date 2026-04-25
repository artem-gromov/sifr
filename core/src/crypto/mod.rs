use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use std::time::SystemTime;
use thiserror::Error;
use totp_rs::{Algorithm as TotpAlgorithm, Secret, TOTP};
use zeroize::Zeroizing;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("TOTP generation failed: {0}")]
    Totp(String),
}

/// Derives a 32-byte vault key from a master password using Argon2id.
/// Returns a Zeroizing wrapper so memory is cleared on drop.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let mut key = Zeroizing::new([0u8; 32]);
    let params =
        Params::new(65536, 3, 1, Some(32)).expect("valid argon2 params: checked at compile time");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    Ok(key)
}

/// Generates a cryptographically secure random salt (16 bytes).
pub fn generate_salt() -> [u8; 16] {
    use rand::RngCore;
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generates a TOTP code from a base32-encoded secret.
/// Returns (6-digit code string, seconds remaining until next code).
pub fn generate_totp(secret: &str) -> Result<(String, u8), CryptoError> {
    let normalized: String = secret
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_uppercase())
        .collect();
    let secret_bytes = Secret::Encoded(normalized)
        .to_bytes()
        .map_err(|e| CryptoError::Totp(e.to_string()))?;
    let totp = TOTP::new_unchecked(TotpAlgorithm::SHA1, 6, 1, 30, secret_bytes);
    let code = totp
        .generate_current()
        .map_err(|e| CryptoError::Totp(e.to_string()))?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| CryptoError::Totp(e.to_string()))?;
    let seconds_remaining = 30 - (now.as_secs() % 30) as u8;
    Ok((code, seconds_remaining))
}

/// Generates a random password with given length and character set flags.
/// Returns a `Zeroizing<String>` so the password is wiped from memory on drop.
pub fn generate_password(
    length: usize,
    uppercase: bool,
    numbers: bool,
    symbols: bool,
) -> Zeroizing<String> {
    use rand::seq::SliceRandom;

    let lower: Vec<char> = ('a'..='z').collect();
    let upper: Vec<char> = ('A'..='Z').collect();
    let digits: Vec<char> = ('0'..='9').collect();
    let syms: Vec<char> = "!@#$%^&*()-_=+[]{}|;:,.<>?".chars().collect();

    let mut all_chars: Vec<char> = lower.clone();
    let mut required_classes: Vec<&[char]> = Vec::new();

    if uppercase {
        all_chars.extend(&upper);
        required_classes.push(&upper);
    }
    if numbers {
        all_chars.extend(&digits);
        required_classes.push(&digits);
    }
    if symbols {
        all_chars.extend(&syms);
        required_classes.push(&syms);
    }

    let mut rng = OsRng;
    let mut result: Vec<char> = (0..length)
        .map(|_| *all_chars.choose(&mut rng).unwrap_or(&'a'))
        .collect();

    // Guarantee each required class is present (if length permits)
    if length >= required_classes.len() {
        let mut positions_used = Vec::new();
        for class in &required_classes {
            let already_present = result.iter().any(|c| class.contains(c));
            if !already_present {
                // Pick a random position not already used for a forced char
                let available: Vec<usize> = (0..length)
                    .filter(|i| !positions_used.contains(i))
                    .collect();
                if let Some(&pos) = available.choose(&mut rng) {
                    if let Some(&ch) = class.choose(&mut rng) {
                        result[pos] = ch;
                        positions_used.push(pos);
                    }
                }
            }
        }
        // Fisher-Yates shuffle
        result.shuffle(&mut rng);
    }

    Zeroizing::new(result.into_iter().collect())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

impl PasswordStrength {
    pub fn label(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "weak",
            PasswordStrength::Medium => "medium",
            PasswordStrength::Strong => "strong",
        }
    }
}

pub fn calculate_password_strength(password: &str) -> PasswordStrength {
    if password.is_empty() {
        return PasswordStrength::Weak;
    }

    let mut pool_size: usize = 0;
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    if has_lower {
        pool_size += 26;
    }
    if has_upper {
        pool_size += 26;
    }
    if has_digit {
        pool_size += 10;
    }
    if has_symbol {
        pool_size += 32;
    }

    if pool_size == 0 {
        return PasswordStrength::Weak;
    }

    let entropy = (password.len() as f64) * (pool_size as f64).log2();

    if entropy < 60.0 {
        PasswordStrength::Weak
    } else if entropy < 80.0 {
        PasswordStrength::Medium
    } else {
        PasswordStrength::Strong
    }
}
