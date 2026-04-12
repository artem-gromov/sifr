use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp_secret: Option<String>,
    pub category_id: Option<i64>,
    pub favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field("totp_secret", &"[REDACTED]")
            .field("category_id", &self.category_id)
            .field("favorite", &self.favorite)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        if let Some(ref mut p) = self.password {
            p.zeroize();
        }
        if let Some(ref mut t) = self.totp_secret {
            t.zeroize();
        }
    }
}

/// Data for creating a new entry.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NewEntry {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp_secret: Option<String>,
    pub category_id: Option<i64>,
}

impl std::fmt::Debug for NewEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewEntry")
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field("totp_secret", &"[REDACTED]")
            .field("category_id", &self.category_id)
            .finish()
    }
}

impl Drop for NewEntry {
    fn drop(&mut self) {
        if let Some(ref mut p) = self.password {
            p.zeroize();
        }
        if let Some(ref mut t) = self.totp_secret {
            t.zeroize();
        }
    }
}

/// Fields that can be updated on an existing entry.
/// Only `Some` fields are applied; `None` fields are left unchanged.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct EntryUpdate {
    pub title: Option<String>,
    pub username: Option<Option<String>>,
    pub password: Option<Option<String>>,
    pub url: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub totp_secret: Option<Option<String>>,
    pub category_id: Option<Option<i64>>,
    pub favorite: Option<bool>,
}

impl std::fmt::Debug for EntryUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryUpdate")
            .field("title", &self.title)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("url", &self.url)
            .field("notes", &self.notes)
            .field(
                "totp_secret",
                &self.totp_secret.as_ref().map(|_| "[REDACTED]"),
            )
            .field("category_id", &self.category_id)
            .field("favorite", &self.favorite)
            .finish()
    }
}

impl Drop for EntryUpdate {
    fn drop(&mut self) {
        if let Some(Some(ref mut p)) = self.password {
            p.zeroize();
        }
        if let Some(Some(ref mut t)) = self.totp_secret {
            t.zeroize();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTag {
    pub entry_id: i64,
    pub tag_id: i64,
}
