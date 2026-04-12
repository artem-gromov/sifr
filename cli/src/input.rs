use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::{App, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::Unlock => handle_unlock(app, key),
        Screen::EntryList => handle_entry_list(app, key),
        Screen::EntryDetail => handle_entry_detail(app, key),
        Screen::Help => handle_help(app, key),
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            // Same as `k`: move selection up on EntryList
            if app.screen == Screen::EntryList && app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        MouseEventKind::ScrollDown => {
            // Same as `j`: move selection down on EntryList
            if app.screen == Screen::EntryList {
                let count = app.filtered_entries().len();
                if count > 0 && app.selected_index + 1 < count {
                    app.selected_index += 1;
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            match app.screen {
                Screen::EntryList => {
                    // Layout: row 0-2 = search bar (3 rows), row 3 = table header, row 4+ = entries
                    let row = mouse.row as usize;
                    if row >= 4 {
                        let index = row - 4;
                        let count = app.filtered_entries().len();
                        if count > 0 && index < count {
                            app.selected_index = index;
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

fn handle_unlock(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            // Mock unlock: accept any password
            app.screen = Screen::EntryList;
            app.password_input.clear();
        }
        KeyCode::Char('q') if app.password_input.is_empty() => {
            app.running = false;
        }
        KeyCode::Char(c) => {
            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                app.running = false;
            } else {
                app.password_input.push(c);
            }
        }
        KeyCode::Backspace => {
            app.password_input.pop();
        }
        KeyCode::Esc => {
            app.running = false;
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
            if app.filtered_entries().get(app.selected_index).is_some() {
                app.copy_to_clipboard("**********"); // mock password
            }
        }
        KeyCode::Char('u') => {
            if let Some(e) = app.filtered_entries().get(app.selected_index) {
                let username = e.username.clone();
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
