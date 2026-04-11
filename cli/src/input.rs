use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::Unlock => handle_unlock(app, key),
        Screen::EntryList => handle_entry_list(app, key),
        Screen::EntryDetail => handle_entry_detail(app, key),
        Screen::Help => handle_help(app, key),
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
