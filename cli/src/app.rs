use sifr_core::theme::ThemeRegistry;

use crate::theme_bridge::ThemeBridge;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    VaultPicker,
    Unlock,
    EntryList,
    EntryDetail,
    Help,
}

#[derive(Debug, Clone)]
pub struct PickerEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_vault: bool,
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
    // Vault picker state
    pub picker_path: std::path::PathBuf,
    pub picker_entries: Vec<PickerEntry>,
    pub picker_selected: usize,
    pub picker_scroll_offset: usize,
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

        let picker_path = std::env::current_dir().unwrap_or_default();
        let mut app = Self {
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
            picker_path,
            picker_entries: Vec::new(),
            picker_selected: 0,
            picker_scroll_offset: 0,
        };
        app.refresh_picker();
        app
    }

    /// Reads `picker_path` and populates `picker_entries`.
    /// Sort order: `..` first, then dirs alphabetically, then .sifr files, then other files.
    /// Only dirs, .sifr files, and `..` are shown.
    pub fn refresh_picker(&mut self) {
        let mut dirs: Vec<PickerEntry> = Vec::new();
        let mut vaults: Vec<PickerEntry> = Vec::new();

        match std::fs::read_dir(&self.picker_path) {
            Ok(read_dir) => {
                for entry_result in read_dir {
                    let dir_entry = match entry_result {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let name = dir_entry.file_name().to_string_lossy().to_string();
                    // Skip hidden files (starting with .) except we already add ".." manually
                    let Ok(metadata) = dir_entry.metadata() else {
                        continue;
                    };
                    let is_dir = metadata.is_dir();
                    let is_vault = !is_dir && name.ends_with(".sifr");
                    if is_dir {
                        dirs.push(PickerEntry {
                            name,
                            is_dir: true,
                            is_vault: false,
                        });
                    } else if is_vault {
                        vaults.push(PickerEntry {
                            name,
                            is_dir: false,
                            is_vault: true,
                        });
                    }
                }
            }
            Err(_) => {
                // Access denied or other error — leave entries empty except for ".."
            }
        }

        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        vaults.sort_by(|a, b| a.name.cmp(&b.name));

        let mut entries: Vec<PickerEntry> = Vec::new();
        // Always add parent dir entry if not at filesystem root
        entries.push(PickerEntry {
            name: "..".to_string(),
            is_dir: true,
            is_vault: false,
        });
        entries.extend(dirs);
        entries.extend(vaults);

        self.picker_entries = entries;
        self.picker_selected = 0;
        self.picker_scroll_offset = 0;
    }

    /// Navigate picker: cd into dir or open vault.
    pub fn picker_enter(&mut self) {
        let Some(entry) = self.picker_entries.get(self.picker_selected) else {
            return;
        };
        if entry.name == ".." {
            // Go up
            if let Some(parent) = self.picker_path.parent().map(|p| p.to_path_buf()) {
                self.picker_path = parent;
            }
            self.refresh_picker();
        } else if entry.is_dir {
            let new_path = self.picker_path.join(&entry.name);
            self.picker_path = new_path;
            self.refresh_picker();
        } else if entry.is_vault {
            let vault_path = self.picker_path.join(&entry.name);
            self.vault_path = vault_path.to_string_lossy().to_string();
            self.screen = Screen::Unlock;
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
