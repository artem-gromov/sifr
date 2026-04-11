use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewEntry {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub totp_secret: Option<String>,
    pub category_id: Option<i64>,
}

/// Fields that can be updated on an existing entry.
/// Only `Some` fields are applied; `None` fields are left unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
