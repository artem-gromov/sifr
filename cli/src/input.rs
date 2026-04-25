use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui_textarea::{Input as TextAreaInput, Key as TextAreaKey, TextArea};
use tui_input::Input as TuiInput;
use tui_input::backend::crossterm::to_input_request;

use crate::app::{App, FIELD_INDEX_NOTES, Screen, UnlockMode};

const NOTES_FIELD_INDEX: usize = FIELD_INDEX_NOTES;

fn apply_tui_input_to_string(value: &mut String, cursor: &mut usize, key: KeyEvent) -> bool {
    let event = crossterm::event::Event::Key(key);
    let Some(req) = to_input_request(&event) else {
        return false;
    };

    let mut input = TuiInput::new(std::mem::take(value)).with_cursor(*cursor);
    let changed = input.handle(req).is_some();
    *cursor = input.cursor();
    *value = input.to_string();
    changed
}

fn apply_tui_input_to_secret(
    value: &mut zeroize::Zeroizing<String>,
    cursor: &mut usize,
    key: KeyEvent,
) -> bool {
    let event = crossterm::event::Event::Key(key);
    let Some(req) = to_input_request(&event) else {
        return false;
    };

    let mut raw = std::mem::take(&mut **value);
    let mut input = TuiInput::new(raw).with_cursor(*cursor);
    let changed = input.handle(req).is_some();
    *cursor = input.cursor();
    raw = input.to_string();
    **value = raw;
    changed
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    app.record_activity();
    // Delete confirmation overlay takes priority
    if app.confirm_delete.is_some() {
        handle_confirm_delete(app, key);
        return;
    }
    match app.screen {
        Screen::VaultPicker => handle_vault_picker(app, key),
        Screen::Unlock => handle_unlock(app, key),
        Screen::EntryList => handle_entry_list(app, key),
        Screen::Help => handle_help(app, key),
        Screen::AddEntry | Screen::EditEntry => handle_form(app, key),
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    app.record_activity();
    match mouse.kind {
        MouseEventKind::ScrollUp => match app.screen {
            Screen::VaultPicker => {
                picker_move_up(app);
            }
            Screen::EntryList if app.selected_index > 0 => {
                app.selected_index -= 1;
            }
            Screen::Help => {
                app.help_scroll_offset = app.help_scroll_offset.saturating_sub(3);
            }
            _ => {}
        },
        MouseEventKind::ScrollDown => match app.screen {
            Screen::VaultPicker => {
                picker_move_down(app);
            }
            Screen::EntryList => {
                let count = app.filtered_entries().len();
                if count > 0 && app.selected_index + 1 < count {
                    app.selected_index += 1;
                }
            }
            Screen::Help => {
                app.help_scroll_offset = app.help_scroll_offset.saturating_add(3);
            }
            _ => {}
        },
        MouseEventKind::Down(MouseButton::Left) => {
            match app.screen {
                Screen::VaultPicker => {
                    let row = mouse.row as usize;
                    if row >= 4 {
                        let index = app.picker_scroll_offset + (row - 4);
                        if index < app.picker_entries.len() {
                            // Double-click detection for picker
                            let now = std::time::Instant::now();
                            let is_double = if let Some((last_time, _, last_row)) = app.last_click {
                                now.duration_since(last_time).as_millis() < 500
                                    && last_row == mouse.row
                            } else {
                                false
                            };

                            app.picker_selected = index;

                            if is_double {
                                app.picker_enter();
                                app.last_click = None;
                            } else {
                                app.last_click = Some((now, mouse.column, mouse.row));
                            }
                        }
                    }
                }
                Screen::EntryList => {
                    // Layout: row 0-2 = search bar, row 3 = table header,
                    // row 4 = header bottom_margin, row 5+ = entries
                    let row = mouse.row as usize;
                    let col = mouse.column;
                    if row < 3 {
                        // Click on search bar → activate search
                        app.search_active = true;
                        app.search_cursor = app.search_query.chars().count();
                    } else if row >= 5 {
                        let index = app.entry_scroll_offset + (row - 5);
                        let count = app.filtered_entries().len();
                        if count > 0 && index < count {
                            app.selected_index = index;

                            // Double-click detection
                            let now = std::time::Instant::now();
                            let is_double =
                                if let Some((last_time, last_col, last_row)) = app.last_click {
                                    let elapsed = now.duration_since(last_time);
                                    elapsed.as_millis() < 500
                                        && last_row == mouse.row
                                        && (last_col as i32 - col as i32).unsigned_abs() < 3
                                } else {
                                    false
                                };

                            if is_double {
                                let clicked_col = determine_column(col, &app.column_boundaries);
                                match clicked_col {
                                    0 => {
                                        // Title → edit entry
                                        edit_selected(app);
                                    }
                                    1 => {
                                        // Username → copy
                                        if let Some(e) =
                                            app.filtered_entries().get(app.selected_index)
                                        {
                                            let username =
                                                e.username.as_deref().unwrap_or("").to_string();
                                            app.copy_to_clipboard(&username, "username");
                                        }
                                    }
                                    2 => {
                                        // Password → copy
                                        if let Some(e) =
                                            app.filtered_entries().get(app.selected_index)
                                        {
                                            let password =
                                                e.password.as_deref().unwrap_or("").to_string();
                                            app.copy_to_clipboard(&password, "password");
                                        }
                                    }
                                    3 => {
                                        // TOTP → copy
                                        if let Some(e) =
                                            app.filtered_entries().get(app.selected_index)
                                        {
                                            if let Some(ref secret) = e.totp_secret {
                                                if let Ok((code, _)) =
                                                    sifr_core::crypto::generate_totp(secret)
                                                {
                                                    app.copy_to_clipboard(&code, "TOTP code");
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                app.last_click = None;
                            } else {
                                app.last_click = Some((now, col, mouse.row));
                            }
                        }
                    }
                }
                Screen::AddEntry | Screen::EditEntry => {
                    if let Some(area) = app.form_modal_area {
                        if mouse.row < area.y
                            || mouse.row >= area.y + area.height
                            || mouse.column < area.x
                            || mouse.column >= area.x + area.width
                        {
                            // Click outside modal → close form
                            app.zeroize_form_password();
                            app.form_fields.clear();
                            app.form_field_cursors.clear();
                            app.form_notes_textarea = None;
                            app.screen = Screen::EntryList;
                        } else {
                            // Click inside modal → detect which field or TOTP row
                            let clicked_field = app
                                .form_field_rows
                                .iter()
                                .position(|(start, end)| mouse.row >= *start && mouse.row < *end);

                            let clicked_totp = app
                                .form_totp_row
                                .map(|(start, end)| mouse.row >= start && mouse.row < end)
                                .unwrap_or(false);

                            let now = std::time::Instant::now();
                            let is_double =
                                if let Some((last_time, last_col, last_row)) = app.last_click {
                                    now.duration_since(last_time).as_millis() < 500
                                        && last_row == mouse.row
                                        && (last_col as i32 - mouse.column as i32)
                                            .unsigned_abs()
                                            < 5
                                } else {
                                    false
                                };

                            if let Some(field_idx) = clicked_field {
                                app.form_focused = field_idx;

                                if is_double {
                                    // Double-click → copy field value
                                    if let Some(field) = app.form_fields.get(field_idx) {
                                        let val = field.value.clone();
                                        let label = field.label.to_lowercase();
                                        app.copy_to_clipboard(&val, &label);
                                    }
                                    app.last_click = None;
                                } else {
                                    app.last_click =
                                        Some((now, mouse.column, mouse.row));
                                }
                            } else if clicked_totp {
                                if is_double {
                                    // Double-click on TOTP code → copy it
                                    let totp_secret = app
                                        .form_fields
                                        .get(3)
                                        .map(|f| f.value.as_str())
                                        .unwrap_or("");
                                    if !totp_secret.is_empty() {
                                        if let Ok((code, _)) =
                                            sifr_core::crypto::generate_totp(totp_secret)
                                        {
                                            app.copy_to_clipboard(&code, "TOTP code");
                                        }
                                    }
                                    app.last_click = None;
                                } else {
                                    app.last_click =
                                        Some((now, mouse.column, mouse.row));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Determines which column index x falls into based on column boundaries.
fn determine_column(x: u16, boundaries: &[u16]) -> usize {
    for i in (0..boundaries.len()).rev() {
        if x >= boundaries[i] {
            return i;
        }
    }
    0
}

fn picker_move_up(app: &mut App) {
    if app.picker_selected > 0 {
        app.picker_selected -= 1;
        if app.picker_selected < app.picker_scroll_offset {
            app.picker_scroll_offset = app.picker_selected;
        }
    }
}

fn picker_move_down(app: &mut App) {
    let count = app.picker_entries.len();
    if count > 0 && app.picker_selected + 1 < count {
        app.picker_selected += 1;
        let visible = 20usize;
        if app.picker_selected >= app.picker_scroll_offset + visible {
            app.picker_scroll_offset = app.picker_selected.saturating_sub(visible - 1);
        }
    }
}

fn handle_vault_picker(app: &mut App, key: KeyEvent) {
    // When naming a new vault, intercept all key events
    if app.picker_naming.is_some() {
        match key.code {
            KeyCode::Enter => {
                if let Some(name) = app.picker_naming.take() {
                    if !name.is_empty() {
                        let vault_name = if name.ends_with(".sifr") {
                            name
                        } else {
                            format!("{}.sifr", name)
                        };
                        let vault_path = app.picker_path.join(&vault_name);
                        app.vault_path = vault_path.to_string_lossy().to_string();
                        app.unlock_mode = UnlockMode::Create;
                        app.confirm_active = false;
                        app.password_input.clear();
                        app.password_confirm.clear();
                        app.password_cursor = 0;
                        app.password_confirm_cursor = 0;
                        app.screen = Screen::Unlock;
                    } else {
                        app.picker_naming = None;
                        app.picker_naming_cursor = 0;
                    }
                }
            }
            KeyCode::Esc => {
                app.picker_naming = None;
                app.picker_naming_cursor = 0;
            }
            _ => {
                if let Some(ref mut name) = app.picker_naming {
                    let _ = apply_tui_input_to_string(name, &mut app.picker_naming_cursor, key);
                }
            }
        }
        return;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            picker_move_down(app);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            picker_move_up(app);
        }
        KeyCode::Enter => {
            app.picker_enter();
        }
        KeyCode::Char('n') => {
            app.picker_naming = Some(String::new());
            app.picker_naming_cursor = 0;
        }
        KeyCode::Char('~') => {
            if let Some(home) = home_dir() {
                app.picker_path = home;
                app.refresh_picker();
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        _ => {}
    }
}

/// Returns the user's home directory.
fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}

fn handle_unlock(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if app.unlock_mode == UnlockMode::Create {
                if !app.confirm_active {
                    // Move to confirm field
                    if app.password_input.is_empty() {
                        return;
                    }
                    app.confirm_active = true;
                    app.password_confirm_cursor = app.password_confirm.chars().count();
                } else {
                    // Second Enter: compare and create
                    if app.password_input != app.password_confirm {
                        app.set_error("Passwords don't match");
                        *app.password_confirm = String::new();
                        app.password_confirm_cursor = 0;
                        app.confirm_active = false;
                        return;
                    }
                    if app.password_input.is_empty() {
                        app.set_error("Password cannot be empty");
                        return;
                    }
                    let path = app.vault_path.clone();
                    match sifr_core::Vault::create(&path, &app.password_input) {
                        Ok(vault) => {
                            app.vault = Some(vault);
                            app.refresh_entries();
                            crate::config::save_last_vault(&path);
                            app.record_activity();
                            *app.password_input = String::new();
                            *app.password_confirm = String::new();
                            app.password_cursor = 0;
                            app.password_confirm_cursor = 0;
                            app.confirm_active = false;
                            app.screen = Screen::EntryList;
                        }
                        Err(e) => {
                            app.set_error(&e.to_string());
                            *app.password_input = String::new();
                            *app.password_confirm = String::new();
                            app.password_cursor = 0;
                            app.password_confirm_cursor = 0;
                            app.confirm_active = false;
                        }
                    }
                }
            } else {
                // Open mode
                if app.password_input.is_empty() {
                    return;
                }
                let path = app.vault_path.clone();
                match sifr_core::Vault::open(&path, &app.password_input) {
                    Ok(vault) => {
                        app.vault = Some(vault);
                        app.refresh_entries();
                        crate::config::save_last_vault(&path);
                        app.record_activity();
                        app.screen = Screen::EntryList;
                    }
                    Err(sifr_core::vault::VaultError::WrongPassword) => {
                        app.set_error("Wrong password");
                    }
                    Err(e) => {
                        app.set_error(&e.to_string());
                    }
                }
                *app.password_input = String::new();
                app.password_cursor = 0;
            }
        }
        KeyCode::Tab => {
            if app.unlock_mode == UnlockMode::Create {
                // Toggle between password and confirm fields
                app.confirm_active = !app.confirm_active;
            } else {
                // Open mode: switch to vault picker to choose a different file
                *app.password_input = String::new();
                app.password_cursor = 0;
                app.started_from_picker = true;
                app.screen = Screen::VaultPicker;
            }
        }
        KeyCode::Esc => {
            if app.unlock_mode == UnlockMode::Create && app.confirm_active {
                // Go back to password field
                app.confirm_active = false;
                *app.password_confirm = String::new();
                app.password_confirm_cursor = 0;
            } else if app.started_from_picker {
                app.screen = Screen::VaultPicker;
            } else {
                app.running = false;
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        _ => {}
    }

    let changed = if app.unlock_mode == UnlockMode::Create && app.confirm_active {
        apply_tui_input_to_secret(&mut app.password_confirm, &mut app.password_confirm_cursor, key)
    } else {
        apply_tui_input_to_secret(&mut app.password_input, &mut app.password_cursor, key)
    };
    if changed {
        app.error_message = None;
        app.error_clear_at = None;
    }
}

fn edit_selected(app: &mut App) {
    let entry_id = app.filtered_entries().get(app.selected_index).map(|e| e.id);
    if let Some(id) = entry_id {
        if let Some(entry) = app.entries.iter().find(|e| e.id == id).cloned() {
            app.init_edit_form(&entry);
        }
    }
}

fn handle_entry_list(app: &mut App, key: KeyEvent) {
    if app.search_active {
        match key.code {
            KeyCode::Esc => {
                // Unfocus search but keep the filter text
                app.search_active = false;
            }
            KeyCode::Enter => {
                app.search_active = false;
            }
            _ => {
                let changed =
                    apply_tui_input_to_string(&mut app.search_query, &mut app.search_cursor, key);
                if changed {
                    app.selected_index = 0;
                    app.refilter();
                }
            }
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                // Clear search filter
                app.search_query.clear();
                app.search_cursor = 0;
                app.selected_index = 0;
                app.refilter();
            } else {
                app.running = false;
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let count = app.filtered_entries().len();
            if count > 0 && app.selected_index + 1 < count {
                app.selected_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up if app.selected_index > 0 => {
            app.selected_index -= 1;
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            edit_selected(app);
        }
        KeyCode::Char('/') => {
            app.search_active = true;
            app.search_cursor = app.search_query.chars().count();
        }
        KeyCode::Char('a') => {
            app.init_add_form();
        }
        KeyCode::Char('d') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let id = e.id;
                app.confirm_delete = Some(id);
            }
        }
        KeyCode::Char('?') => {
            app.screen = Screen::Help;
        }
        KeyCode::Char('t') => {
            let entry_id = app.filtered_entries().get(app.selected_index).map(|e| e.id);
            if let Some(id) = entry_id {
                if let Some(ref vault) = app.vault {
                    match vault.get_totp_code(id) {
                        Ok((code, _)) => app.copy_to_clipboard(&code, "TOTP code"),
                        Err(_) => app.set_error("No TOTP configured for this entry"),
                    }
                }
            }
        }
        KeyCode::Char('y') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let password = e.password.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&password, "password");
            }
        }
        KeyCode::Char('u') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let username = e.username.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&username, "username");
            }
        }
        KeyCode::Char('f') => {
            let info = app
                .filtered_entries()
                .get(app.selected_index)
                .map(|e| (e.id, !e.favorite));
            if let Some((id, new_fav)) = info {
                if let Some(ref vault) = app.vault {
                    let updates = sifr_core::EntryUpdate {
                        title: None,
                        username: None,
                        password: None,
                        url: None,
                        notes: None,
                        totp_secret: None,
                        category_id: None,
                        favorite: Some(new_fav),
                    };
                    match vault.update_entry(id, updates) {
                        Ok(_) => app.refresh_entries(),
                        Err(e) => app.set_error(&format!("Failed: {}", e)),
                    }
                }
            }
        }
        KeyCode::Char('L') => {
            // Lock vault: clear all sensitive state
            app.vault = None;
            app.entries.clear();
            app.selected_index = 0;
            app.entry_scroll_offset = 0;
            app.search_query.clear();
            app.search_cursor = 0;
            app.search_active = false;
            *app.password_input = String::new();
            app.password_cursor = 0;
            if app.started_from_picker {
                app.screen = Screen::VaultPicker;
            } else {
                app.screen = Screen::Unlock;
            }
        }
        _ => {}
    }
}

fn handle_help(app: &mut App, key: KeyEvent) {
    let max_offset = app
        .help_total_lines
        .saturating_sub(app.help_visible_lines);
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.screen = Screen::EntryList;
            app.help_scroll_offset = 0;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.help_scroll_offset = app.help_scroll_offset.saturating_add(1).min(max_offset);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.help_scroll_offset = app.help_scroll_offset.saturating_sub(1);
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        _ => {}
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn ensure_notes_textarea(app: &mut App) {
    if app.form_notes_textarea.is_some() {
        return;
    }

    let lines = app
        .form_fields
        .get(NOTES_FIELD_INDEX)
        .map(|f| {
            if f.value.is_empty() {
                vec![String::new()]
            } else {
                f.value.lines().map(|s| s.to_string()).collect()
            }
        })
        .unwrap_or_else(|| vec![String::new()]);
    app.form_notes_textarea = Some(TextArea::new(lines));
}

fn sync_notes_textarea(app: &mut App) {
    let Some(textarea) = app.form_notes_textarea.take() else {
        return;
    };
    let text = textarea.into_lines().join("\n");
    if let Some(field) = app.form_fields.get_mut(NOTES_FIELD_INDEX) {
        field.value = text;
    }
}

fn key_event_to_textarea_input(key: KeyEvent) -> TextAreaInput {
    let mut shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let mapped_key = match key.code {
        KeyCode::Backspace => TextAreaKey::Backspace,
        KeyCode::Enter => TextAreaKey::Enter,
        KeyCode::Left => TextAreaKey::Left,
        KeyCode::Right => TextAreaKey::Right,
        KeyCode::Up => TextAreaKey::Up,
        KeyCode::Down => TextAreaKey::Down,
        KeyCode::Tab => TextAreaKey::Tab,
        KeyCode::BackTab => {
            shift = true;
            TextAreaKey::Tab
        }
        KeyCode::Delete => TextAreaKey::Delete,
        KeyCode::Home => TextAreaKey::Home,
        KeyCode::End => TextAreaKey::End,
        KeyCode::PageUp => TextAreaKey::PageUp,
        KeyCode::PageDown => TextAreaKey::PageDown,
        KeyCode::Esc => TextAreaKey::Esc,
        KeyCode::F(n) => TextAreaKey::F(n),
        KeyCode::Char(c) => TextAreaKey::Char(c),
        _ => TextAreaKey::Null,
    };

    TextAreaInput {
        key: mapped_key,
        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
        alt: key.modifiers.contains(KeyModifiers::ALT),
        shift,
    }
}

fn handle_form(app: &mut App, key: KeyEvent) {
    let field_count = app.form_fields.len();
    let is_add = app.form_editing_id.is_none();

    if let Some(editing_idx) = app.form_editing_field {
        if editing_idx == NOTES_FIELD_INDEX {
            ensure_notes_textarea(app);
            match key.code {
                KeyCode::Esc => {
                    if is_add {
                        app.zeroize_form_password();
                        app.form_fields.clear();
                        app.form_field_cursors.clear();
                        app.form_notes_textarea = None;
                        app.screen = Screen::EntryList;
                    } else {
                        sync_notes_textarea(app);
                        app.form_editing_field = None;
                    }
                }
                KeyCode::Tab | KeyCode::Down => {
                    sync_notes_textarea(app);
                    let next = (editing_idx + 1) % field_count;
                    app.form_focused = next;
                    app.form_editing_field = Some(next);
                    if next == NOTES_FIELD_INDEX {
                        ensure_notes_textarea(app);
                    }
                }
                KeyCode::BackTab | KeyCode::Up => {
                    sync_notes_textarea(app);
                    let prev = if editing_idx == 0 {
                        field_count - 1
                    } else {
                        editing_idx - 1
                    };
                    app.form_focused = prev;
                    app.form_editing_field = Some(prev);
                    if prev == NOTES_FIELD_INDEX {
                        ensure_notes_textarea(app);
                    }
                }
                KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => match c {
                    'c' => app.running = false,
                    's' => {
                        sync_notes_textarea(app);
                        submit_form(app);
                    }
                    _ => {}
                },
                _ => {
                    app.error_message = None;
                    app.error_clear_at = None;
                    if let Some(textarea) = app.form_notes_textarea.as_mut() {
                        let _ = textarea.input(key_event_to_textarea_input(key));
                    }
                }
            }
            return;
        }

        // Editing a specific field
        match key.code {
            KeyCode::Esc => {
                if is_add {
                    // Cancel add entirely
                    app.zeroize_form_password();
                    app.form_fields.clear();
                    app.form_field_cursors.clear();
                    app.form_notes_textarea = None;
                    app.screen = Screen::EntryList;
                } else {
                    // Back to view mode
                    app.form_editing_field = None;
                }
            }
            KeyCode::Tab | KeyCode::Down => {
                let next = (editing_idx + 1) % field_count;
                app.form_focused = next;
                app.form_editing_field = Some(next);
                if next == NOTES_FIELD_INDEX {
                    ensure_notes_textarea(app);
                }
            }
            KeyCode::BackTab | KeyCode::Up => {
                let prev = if editing_idx == 0 {
                    field_count - 1
                } else {
                    editing_idx - 1
                };
                app.form_focused = prev;
                app.form_editing_field = Some(prev);
                if prev == NOTES_FIELD_INDEX {
                    ensure_notes_textarea(app);
                }
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => app.running = false,
                        'g' if editing_idx == 2 => {
                            let pwd = sifr_core::crypto::generate_password(16, true, true, true);
                            if let Some(field) = app.form_fields.get_mut(2) {
                                field.value = pwd.as_str().to_string();
                                if let Some(cursor) = app.form_field_cursors.get_mut(2) {
                                    *cursor = field.value.chars().count();
                                }
                            }
                        }
                        's' => {
                            submit_form(app);
                        }
                        _ => {}
                    }
                } else {
                    let fields_len = app.form_fields.len();
                    if app.form_field_cursors.len() <= editing_idx {
                        app.form_field_cursors.resize(fields_len, 0);
                    }
                    if let Some(field) = app.form_fields.get_mut(editing_idx) {
                        if let Some(cursor) = app.form_field_cursors.get_mut(editing_idx) {
                            let changed = apply_tui_input_to_string(&mut field.value, cursor, key);
                            if changed {
                                app.error_message = None;
                                app.error_clear_at = None;
                            }
                        }
                    }
                }
            }
            _ => {
                let fields_len = app.form_fields.len();
                if app.form_field_cursors.len() <= editing_idx {
                    app.form_field_cursors.resize(fields_len, 0);
                }
                if let Some(field) = app.form_fields.get_mut(editing_idx) {
                    if let Some(cursor) = app.form_field_cursors.get_mut(editing_idx) {
                        let changed = apply_tui_input_to_string(&mut field.value, cursor, key);
                        if changed {
                            app.error_message = None;
                            app.error_clear_at = None;
                        }
                    }
                }
            }
        }
    } else {
        // View/detail mode (form_editing_field is None)
        match key.code {
            KeyCode::Esc => {
                app.zeroize_form_password();
                app.form_fields.clear();
                app.form_field_cursors.clear();
                app.form_notes_textarea = None;
                app.screen = Screen::EntryList;
            }
            KeyCode::Char('j') | KeyCode::Down
                if field_count > 0 && app.form_focused + 1 < field_count =>
            {
                app.form_focused += 1;
            }
            KeyCode::Char('k') | KeyCode::Up if app.form_focused > 0 => {
                app.form_focused -= 1;
            }
            KeyCode::Enter => {
                app.form_editing_field = Some(app.form_focused);
                if app.form_focused == NOTES_FIELD_INDEX {
                    ensure_notes_textarea(app);
                }
            }
            KeyCode::Char('y') => {
                if let Some(field) = app.form_fields.get(app.form_focused) {
                    let val = field.value.clone();
                    let label = field.label.to_lowercase();
                    app.copy_to_clipboard(&val, &label);
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.running = false;
            }
            _ => {}
        }
    }
}

fn submit_form(app: &mut App) {
    if app.form_editing_field == Some(NOTES_FIELD_INDEX) {
        sync_notes_textarea(app);
    }

    // Validate required fields
    let title = app
        .form_fields
        .first()
        .map(|f| f.value.trim().to_string())
        .unwrap_or_default();
    if title.is_empty() {
        app.set_error("Title is required");
        return;
    }

    let username = app
        .form_fields
        .get(1)
        .map(|f| non_empty(f.value.trim()))
        .unwrap_or(None);
    let password = app
        .form_fields
        .get(2)
        .map(|f| non_empty(f.value.trim()))
        .unwrap_or(None);
    let totp_secret = app
        .form_fields
        .get(3)
        .map(|f| non_empty(f.value.trim()))
        .unwrap_or(None);
    let url = app
        .form_fields
        .get(4)
        .map(|f| non_empty(f.value.trim()))
        .unwrap_or(None);
    let notes = app
        .form_fields
        .get(5)
        .map(|f| non_empty(f.value.trim()))
        .unwrap_or(None);

    if let Some(editing_id) = app.form_editing_id {
        // Edit mode: build EntryUpdate
        let updates = sifr_core::EntryUpdate {
            title: Some(title),
            username: Some(username),
            password: Some(password),
            url: Some(url),
            notes: Some(notes),
            totp_secret: Some(totp_secret),
            category_id: None,
            favorite: None,
        };
        if let Some(ref vault) = app.vault {
            match vault.update_entry(editing_id, updates) {
                Ok(_) => {
                    app.zeroize_form_password();
                    app.form_fields.clear();
                    app.form_field_cursors.clear();
                    app.form_notes_textarea = None;
                    app.form_editing_id = None;
                    app.refresh_entries();
                    app.screen = Screen::EntryList;
                }
                Err(e) => {
                    app.set_error(&format!("Update failed: {}", e));
                }
            }
        }
    } else {
        // Add mode
        let new = sifr_core::NewEntry {
            title,
            username,
            password,
            url,
            notes,
            totp_secret,
            category_id: None,
        };
        if let Some(ref vault) = app.vault {
            match vault.add_entry(&new) {
                Ok(_) => {
                    app.zeroize_form_password();
                    app.form_fields.clear();
                    app.form_field_cursors.clear();
                    app.form_notes_textarea = None;
                    app.refresh_entries();
                    app.screen = Screen::EntryList;
                }
                Err(e) => {
                    app.set_error(&format!("Add failed: {}", e));
                }
            }
        }
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(id) = app.confirm_delete.take() {
                if let Some(ref vault) = app.vault {
                    match vault.delete_entry(id) {
                        Ok(()) => {
                            app.refresh_entries();
                            // Clamp selected index
                            let count = app.entries.len();
                            if app.selected_index >= count && count > 0 {
                                app.selected_index = count - 1;
                            }
                        }
                        Err(e) => {
                            app.set_error(&format!("Delete failed: {}", e));
                        }
                    }
                }
                app.confirm_delete = None;
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_delete = None;
        }
        _ => {}
    }
}
