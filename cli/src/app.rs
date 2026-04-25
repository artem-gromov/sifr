use crate::theme_bridge::ThemeBridge;
use ratatui_textarea::TextArea;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    VaultPicker,
    Unlock,
    EntryList,
    Help,
    AddEntry,
    EditEntry,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub value: String,
    pub required: bool,
    pub secret: bool,
}

impl Drop for FormField {
    fn drop(&mut self) {
        if self.secret {
            zeroize::Zeroize::zeroize(&mut self.value);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnlockMode {
    Open,
    Create,
}

#[derive(Debug, Clone)]
pub struct PickerEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_vault: bool,
}

pub const FIELD_INDEX_PASSWORD: usize = 2;
pub const FIELD_INDEX_NOTES: usize = 5;

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub vault_path: String,
    pub search_query: String,
    pub search_cursor: usize,
    pub search_active: bool,
    pub selected_index: usize,
    pub vault: Option<sifr_core::Vault>,
    pub entries: Vec<sifr_core::models::Entry>,
    pub error_message: Option<String>,
    pub error_clear_at: Option<std::time::Instant>,
    pub started_from_picker: bool,
    pub password_input: zeroize::Zeroizing<String>,
    pub password_confirm: zeroize::Zeroizing<String>,
    pub password_cursor: usize,
    pub password_confirm_cursor: usize,
    pub confirm_active: bool,
    pub password_visible: bool,
    pub unlock_mode: UnlockMode,
    pub clipboard_notification: Option<String>,
    pub clipboard_clear_at: Option<std::time::Instant>,
    pub last_activity_at: Option<std::time::Instant>,
    pub auto_lock_timeout_secs: u64,
    // Double-click tracking
    pub last_click: Option<(std::time::Instant, u16, u16)>,
    pub column_boundaries: Vec<u16>,
    // Vault picker state
    pub picker_path: std::path::PathBuf,
    pub picker_entries: Vec<PickerEntry>,
    pub picker_selected: usize,
    pub picker_scroll_offset: usize,
    pub picker_naming: Option<String>,
    pub picker_naming_cursor: usize,
    // Entry form state
    pub form_fields: Vec<FormField>,
    pub form_focused: usize,
    pub form_editing_id: Option<i64>,
    pub form_editing_field: Option<usize>,
    pub form_field_cursors: Vec<usize>,
    pub form_created_at: i64,
    pub form_updated_at: i64,
    pub form_modal_area: Option<ratatui::layout::Rect>,
    pub confirm_delete: Option<i64>,
    pub filtered_indices: Vec<usize>,
    pub entry_scroll_offset: usize,
    // Maps form field index → row range (start_y, end_y) for mouse detection
    pub form_field_rows: Vec<(u16, u16)>,
    // TOTP code row range for click detection
    pub form_totp_row: Option<(u16, u16)>,
    // Notes field textarea when editing notes (field index 5)
    pub form_notes_textarea: Option<TextArea<'static>>,
    // Help screen scroll
    pub help_scroll_offset: usize,
    pub help_total_lines: usize,
    pub help_visible_lines: usize,
}

impl App {
    pub fn new(vault_path: String) -> Self {
        let picker_path = std::env::current_dir().unwrap_or_default();
        let mut app = Self {
            screen: Screen::Unlock,
            running: true,
            vault_path,
            search_query: String::new(),
            search_cursor: 0,
            search_active: false,
            selected_index: 0,
            vault: None,
            entries: Vec::new(),
            error_message: None,
            error_clear_at: None,
            started_from_picker: false,
            password_input: zeroize::Zeroizing::new(String::new()),
            password_confirm: zeroize::Zeroizing::new(String::new()),
            password_cursor: 0,
            password_confirm_cursor: 0,
            confirm_active: false,
            password_visible: false,
            unlock_mode: UnlockMode::Open,
            clipboard_notification: None,
            clipboard_clear_at: None,
            last_activity_at: None,
            auto_lock_timeout_secs: 300,
            last_click: None,
            column_boundaries: Vec::new(),
            picker_path,
            picker_entries: Vec::new(),
            picker_selected: 0,
            picker_scroll_offset: 0,
            picker_naming: None,
            picker_naming_cursor: 0,
            form_fields: Vec::new(),
            form_focused: 0,
            form_editing_id: None,
            form_editing_field: None,
            form_field_cursors: Vec::new(),
            form_created_at: 0,
            form_updated_at: 0,
            form_modal_area: None,
            confirm_delete: None,
            filtered_indices: Vec::new(),
            entry_scroll_offset: 0,
            form_field_rows: Vec::new(),
            form_totp_row: None,
            form_notes_textarea: None,
            help_scroll_offset: 0,
            help_total_lines: 0,
            help_visible_lines: 0,
        };
        app.refresh_picker();
        app
    }

    pub fn refresh_entries(&mut self) {
        if let Some(ref vault) = self.vault {
            match vault.list_entries() {
                Ok(list) => self.entries = list,
                Err(e) => self.set_error(&format!("Failed to load entries: {}", e)),
            }
        }
        self.refilter();
    }

    pub fn refilter(&mut self) {
        self.entry_scroll_offset = 0;
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.entries.len()).collect();
        } else {
            let q = self.search_query.to_lowercase();
            self.filtered_indices = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.title.to_lowercase().contains(&q)
                        || e.username
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q)
                        || e.url.as_deref().unwrap_or("").to_lowercase().contains(&q)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    pub fn set_error(&mut self, msg: &str) {
        self.error_message = Some(msg.to_string());
        self.error_clear_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
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
            self.unlock_mode = UnlockMode::Open;
            self.screen = Screen::Unlock;
        }
    }

    pub fn copy_to_clipboard(&mut self, text: &str, label: &str) {
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(text.to_string()).is_ok() {
                    self.clipboard_notification = Some(format!("Copied {}", label));
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

    pub fn filtered_entries(&self) -> Vec<&sifr_core::models::Entry> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    /// Returns the appropriate `ThemeBridge` for the current theme state.
    /// When no theme is active, returns a terminal-native bridge (all `Style::default()`).
    pub fn theme_bridge(&self) -> ThemeBridge {
        ThemeBridge::new()
    }

    fn make_form_fields() -> Vec<FormField> {
        vec![
            FormField {
                label: "Title".to_string(),
                value: String::new(),
                required: true,
                secret: false,
            },
            FormField {
                label: "Username".to_string(),
                value: String::new(),
                required: false,
                secret: false,
            },
            FormField {
                label: "Password".to_string(),
                value: String::new(),
                required: false,
                secret: true,
            },
            FormField {
                label: "TOTP Secret".to_string(),
                value: String::new(),
                required: false,
                secret: true,
            },
            FormField {
                label: "URL".to_string(),
                value: String::new(),
                required: false,
                secret: false,
            },
            FormField {
                label: "Notes".to_string(),
                value: String::new(),
                required: false,
                secret: false,
            },
        ]
    }

    pub fn init_add_form(&mut self) {
        self.form_fields = Self::make_form_fields();
        self.form_field_cursors = self
            .form_fields
            .iter()
            .map(|f| f.value.chars().count())
            .collect();
        self.form_focused = 0;
        self.form_editing_id = None;
        self.form_editing_field = Some(0);
        self.form_created_at = 0;
        self.form_updated_at = 0;
        self.form_modal_area = None;
        self.form_notes_textarea = None;
        self.error_message = None;
        self.error_clear_at = None;
        self.screen = Screen::AddEntry;
    }

    pub fn init_edit_form(&mut self, entry: &sifr_core::models::Entry) {
        let mut fields = Self::make_form_fields();
        fields[0].value = entry.title.clone();
        fields[1].value = entry.username.as_deref().unwrap_or("").to_string();
        fields[2].value = entry.password.as_deref().unwrap_or("").to_string();
        fields[3].value = entry.totp_secret.as_deref().unwrap_or("").to_string();
        fields[4].value = entry.url.as_deref().unwrap_or("").to_string();
        fields[5].value = entry.notes.as_deref().unwrap_or("").to_string();
        self.form_fields = fields;
        self.form_field_cursors = self
            .form_fields
            .iter()
            .map(|f| f.value.chars().count())
            .collect();
        self.form_focused = 0;
        self.form_editing_id = Some(entry.id);
        self.form_editing_field = None;
        self.form_created_at = entry.created_at;
        self.form_updated_at = entry.updated_at;
        self.form_modal_area = None;
        self.form_notes_textarea = None;
        self.error_message = None;
        self.error_clear_at = None;
        self.screen = Screen::EditEntry;
    }

    pub fn zeroize_form_password(&mut self) {
        for field in &mut self.form_fields {
            if field.secret {
                zeroize::Zeroize::zeroize(&mut field.value);
            }
        }
    }

    pub fn lock(&mut self) {
        self.vault = None;
        self.entries.clear();
        self.search_query.clear();
        self.filtered_indices.clear();
        *self.password_input = String::new();
        *self.password_confirm = String::new();
        self.password_cursor = 0;
        self.password_confirm_cursor = 0;
        self.confirm_active = false;
        self.password_visible = false;
        self.last_activity_at = None;
        if self.started_from_picker {
            self.screen = Screen::VaultPicker;
        } else {
            self.screen = Screen::Unlock;
        }
    }

    pub fn record_activity(&mut self) {
        if self.vault.is_some() {
            self.last_activity_at = Some(std::time::Instant::now());
        }
    }

    pub fn check_auto_lock(&mut self) -> bool {
        if let (Some(last_activity), Some(_vault)) = (self.last_activity_at, &self.vault) {
            if self.auto_lock_timeout_secs > 0 {
                let elapsed = std::time::Instant::now().duration_since(last_activity);
                if elapsed.as_secs() >= self.auto_lock_timeout_secs {
                    self.lock();
                    self.set_error("Vault locked due to inactivity");
                    return true;
                }
            }
        }
        false
    }
}
