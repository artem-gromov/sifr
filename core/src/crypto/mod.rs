use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
}

/// Derives a 32-byte vault key from a master password using Argon2id.
/// Returns a Zeroizing wrapper so memory is cleared on drop.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let mut key = Zeroizing::new([0u8; 32]);
    let params =
        Params::new(19456, 2, 1, Some(32)).expect("valid argon2 params: checked at compile time");
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

/// Generates a random password with given length and character set flags.
/// Returns a `Zeroizing<String>` so the password is wiped from memory on drop.
pub fn generate_password(
    length: usize,
    uppercase: bool,
    numbers: bool,
    symbols: bool,
) -> Zeroizing<String> {
    use rand::seq::SliceRandom;
    let mut chars: Vec<char> = ('a'..='z').collect();
    if uppercase {
        chars.extend('A'..='Z');
    }
    if numbers {
        chars.extend('0'..='9');
    }
    if symbols {
        chars.extend("!@#$%^&*()-_=+[]{}|;:,.<>?".chars());
    }
    let mut rng = OsRng;
    let s: String = (0..length)
        .map(|_| *chars.choose(&mut rng).unwrap_or(&'a'))
        .collect();
    Zeroizing::new(s)
}
