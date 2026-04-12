use sifr_core::theme::ThemeRegistry;

use crate::theme_bridge::ThemeBridge;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Unlock,
    EntryList,
    EntryDetail,
    Help,
}

#[derive(Debug, Clone)]
pub struct MockEntry {
    pub title: String,
    pub url: String,
    pub username: String,
    pub category: String,
}

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub vault_path: String,
    pub search_query: String,
    pub search_active: bool,
    pub selected_index: usize,
    pub entries: Vec<MockEntry>,
    pub theme: ThemeRegistry,
    pub password_input: String,
    pub password_visible: bool,
    pub clipboard_notification: Option<String>,
    pub clipboard_clear_at: Option<std::time::Instant>,
}

impl App {
    pub fn new(vault_path: String) -> Self {
        let entries = vec![
            MockEntry {
                title: "GitHub".into(),
                url: "github.com".into(),
                username: "john@example.com".into(),
                category: "Dev".into(),
            },
            MockEntry {
                title: "AWS Console".into(),
                url: "aws.amazon.com".into(),
                username: "admin@example.com".into(),
                category: "Cloud".into(),
            },
            MockEntry {
                title: "Gmail".into(),
                url: "mail.google.com".into(),
                username: "john@gmail.com".into(),
                category: "Email".into(),
            },
            MockEntry {
                title: "Netflix".into(),
                url: "netflix.com".into(),
                username: "john@gmail.com".into(),
                category: "Media".into(),
            },
            MockEntry {
                title: "Cloudflare".into(),
                url: "cloudflare.com".into(),
                username: "admin@example.com".into(),
                category: "Cloud".into(),
            },
            MockEntry {
                title: "Figma".into(),
                url: "figma.com".into(),
                username: "john@example.com".into(),
                category: "Design".into(),
            },
            MockEntry {
                title: "Spotify".into(),
                url: "spotify.com".into(),
                username: "john@gmail.com".into(),
                category: "Media".into(),
            },
            MockEntry {
                title: "Vercel".into(),
                url: "vercel.com".into(),
                username: "john@example.com".into(),
                category: "Dev".into(),
            },
        ];

        Self {
            screen: Screen::Unlock,
            running: true,
            vault_path,
            search_query: String::new(),
            search_active: false,
            selected_index: 0,
            entries,
            theme: ThemeRegistry::new(),
            password_input: String::new(),
            password_visible: false,
            clipboard_notification: None,
            clipboard_clear_at: None,
        }
    }

    pub fn copy_to_clipboard(&mut self, text: &str) {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(text.to_string()).is_ok() {
                    self.clipboard_notification = Some("Copied! Auto-clears in 30s".into());
                    self.clipboard_clear_at =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(30));
                } else {
                    self.clipboard_notification = Some("Clipboard unavailable".into());
                }
            }
            Err(_) => {
                self.clipboard_notification = Some("Clipboard unavailable".into());
            }
        }
    }

    pub fn filtered_entries(&self) -> Vec<&MockEntry> {
        if self.search_query.is_empty() {
            self.entries.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.entries
                .iter()
                .filter(|e| {
                    e.title.to_lowercase().contains(&q)
                        || e.url.to_lowercase().contains(&q)
                        || e.username.to_lowercase().contains(&q)
                        || e.category.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    /// Returns the appropriate `ThemeBridge` for the current theme state.
    /// When no theme is active, returns a terminal-native bridge (all `Style::default()`).
    pub fn theme_bridge(&self) -> ThemeBridge<'_> {
        match self.theme.active() {
            Some(t) => ThemeBridge::new(&t.palette),
            None => ThemeBridge::terminal(),
        }
    }

    /// Cycles: None → first theme → ... → last theme → None.
    pub fn cycle_theme(&mut self) {
        let themes: Vec<String> = self.theme.list().iter().map(|s| s.to_string()).collect();
        match self.theme.active() {
            None => {
                // Move to first theme
                if let Some(first) = themes.first() {
                    let _ = self.theme.set_active(first);
                }
            }
            Some(current) => {
                let active_key = current.name.to_lowercase().replace(' ', "-");
                let pos = themes.iter().position(|t| t == &active_key).unwrap_or(0);
                if pos + 1 >= themes.len() {
                    // Wrap back to terminal native
                    self.theme.clear_active();
                } else {
                    let next = &themes[pos + 1];
                    let _ = self.theme.set_active(next);
                }
            }
        }
    }
}
