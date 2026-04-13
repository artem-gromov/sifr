use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, Screen, UnlockMode};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::VaultPicker => handle_vault_picker(app, key),
        Screen::Unlock => handle_unlock(app, key),
        Screen::EntryList => handle_entry_list(app, key),
        Screen::EntryDetail => handle_entry_detail(app, key),
        Screen::Help => handle_help(app, key),
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => match app.screen {
            Screen::VaultPicker => {
                picker_move_up(app);
            }
            Screen::EntryList => {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                }
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
            _ => {}
        },
        MouseEventKind::Down(MouseButton::Left) => {
            match app.screen {
                Screen::VaultPicker => {
                    let row = mouse.row as usize;
                    if row >= 4 {
                        let index = app.picker_scroll_offset + (row - 4);
                        if index < app.picker_entries.len() {
                            app.picker_selected = index;
                        }
                    }
                }
                Screen::EntryList => {
                    // Layout: row 0-2 = search bar, row 3 = table header, row 4+ = entries
                    let row = mouse.row as usize;
                    let col = mouse.column;
                    if row >= 4 {
                        let index = row - 4;
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
                                    1 => {
                                        // Title → open detail
                                        app.screen = Screen::EntryDetail;
                                    }
                                    2 => {
                                        // Username → copy
                                        if let Some(e) =
                                            app.filtered_entries().get(app.selected_index)
                                        {
                                            let username =
                                                e.username.as_deref().unwrap_or("").to_string();
                                            app.copy_to_clipboard(&username);
                                        }
                                    }
                                    3 => {
                                        // Password → copy
                                        if let Some(e) =
                                            app.filtered_entries().get(app.selected_index)
                                        {
                                            let password =
                                                e.password.as_deref().unwrap_or("").to_string();
                                            app.copy_to_clipboard(&password);
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
                Screen::EntryDetail | Screen::Help => {
                    // Click outside modal → go back to EntryList
                    app.screen = Screen::EntryList;
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
            let vault_path = app.picker_path.join("new.sifr");
            app.vault_path = vault_path.to_string_lossy().to_string();
            app.unlock_mode = UnlockMode::Create;
            app.confirm_active = false;
            app.password_input.clear();
            app.password_confirm.clear();
            app.screen = Screen::Unlock;
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
                } else {
                    // Second Enter: compare and create
                    if app.password_input != app.password_confirm {
                        app.set_error("Passwords don't match");
                        zeroize::Zeroize::zeroize(&mut app.password_confirm);
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
                            zeroize::Zeroize::zeroize(&mut app.password_input);
                            zeroize::Zeroize::zeroize(&mut app.password_confirm);
                            app.confirm_active = false;
                            app.screen = Screen::EntryList;
                        }
                        Err(e) => {
                            app.set_error(&e.to_string());
                            zeroize::Zeroize::zeroize(&mut app.password_input);
                            zeroize::Zeroize::zeroize(&mut app.password_confirm);
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
                        app.screen = Screen::EntryList;
                    }
                    Err(sifr_core::vault::VaultError::WrongPassword) => {
                        app.set_error("Wrong password");
                    }
                    Err(e) => {
                        app.set_error(&e.to_string());
                    }
                }
                zeroize::Zeroize::zeroize(&mut app.password_input);
            }
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                app.running = false;
            } else {
                // Clear error when user types
                app.error_message = None;
                app.error_clear_at = None;
                if app.unlock_mode == UnlockMode::Create && app.confirm_active {
                    app.password_confirm.push(c);
                } else {
                    app.password_input.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if app.unlock_mode == UnlockMode::Create && app.confirm_active {
                app.password_confirm.pop();
            } else {
                app.password_input.pop();
            }
        }
        KeyCode::Esc => {
            if app.unlock_mode == UnlockMode::Create && app.confirm_active {
                // Go back to password field
                app.confirm_active = false;
                zeroize::Zeroize::zeroize(&mut app.password_confirm);
            } else if app.started_from_picker {
                app.screen = Screen::VaultPicker;
            } else {
                app.running = false;
            }
        }
        _ => {}
    }
}

fn handle_entry_list(app: &mut App, key: KeyEvent) {
    if app.search_active {
        match key.code {
            KeyCode::Esc => {
                app.search_active = false;
                app.search_query.clear();
                app.selected_index = 0;
            }
            KeyCode::Enter => {
                app.search_active = false;
                app.selected_index = 0;
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.selected_index = 0;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.selected_index = 0;
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
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
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Enter => {
            app.screen = Screen::EntryDetail;
        }
        KeyCode::Char('/') => {
            app.search_active = true;
        }
        KeyCode::Char('a') => {
            // Placeholder: add entry
        }
        KeyCode::Char('?') => {
            app.screen = Screen::Help;
        }
        KeyCode::Char('t') => {
            app.cycle_theme();
        }
        KeyCode::Char('y') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let password = e.password.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&password);
            }
        }
        KeyCode::Char('u') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let username = e.username.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&username);
            }
        }
        _ => {}
    }
}

fn handle_entry_detail(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.screen = Screen::EntryList;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        KeyCode::Char('y') | KeyCode::Char('c') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let password = e.password.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&password);
            }
        }
        KeyCode::Char('u') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let username = e.username.as_deref().unwrap_or("").to_string();
                app.copy_to_clipboard(&username);
            }
        }
        _ => {}
    }
}

fn handle_help(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.screen = Screen::EntryList;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        _ => {}
    }
}
