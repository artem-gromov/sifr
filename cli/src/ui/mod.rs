pub mod entry_form;
pub mod entry_list;
pub mod status_bar;
pub mod unlock;
pub mod vault_picker;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::VaultPicker => vault_picker::draw(f, app),
        Screen::Unlock => unlock::draw(f, app),
        Screen::EntryList => entry_list::draw(f, app),
        Screen::EntryDetail => draw_entry_detail(f, app),
        Screen::Help => draw_help(f, app),
        Screen::AddEntry | Screen::EditEntry => entry_form::draw(f, app),
    }

    // Delete confirmation overlay (drawn on top of any screen)
    if app.confirm_delete.is_some() {
        draw_delete_confirm(f, app);
    }
}

fn format_timestamp(ts: i64) -> String {
    // Simple human-readable format without external dependencies
    // Show as YYYY-MM-DD using basic arithmetic
    if ts <= 0 {
        return "—".to_string();
    }
    // Days since Unix epoch
    let days = ts / 86400;
    // Gregorian calendar calculation
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn draw_entry_detail(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.size();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let entries = app.filtered_entries();
    let entry = entries.get(app.selected_index);

    let modal_area = centered_rect_pct(60, 70, full);
    f.render_widget(Clear, modal_area);

    let content = if let Some(e) = entry {
        let fav_char = if e.favorite { "\u{2605}" } else { "\u{2606}" };
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Title:    ", tb.muted()),
                Span::styled(e.title.clone(), tb.text()),
                Span::styled(format!("  {}", fav_char), tb.accent()),
            ]),
            Line::from(vec![
                Span::styled("  Username: ", tb.muted()),
                Span::styled(e.username.as_deref().unwrap_or("—").to_string(), tb.text()),
            ]),
            Line::from(vec![
                Span::styled("  Password: ", tb.muted()),
                Span::styled(
                    "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                    tb.accent(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  URL:      ", tb.muted()),
                Span::styled(e.url.as_deref().unwrap_or("—").to_string(), tb.blue()),
            ]),
        ];

        if let Some(ref notes) = e.notes {
            if !notes.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Notes:    ", tb.muted()),
                    Span::styled(notes.clone(), tb.text()),
                ]));
            }
        }

        lines.push(Line::from(vec![
            Span::styled("  Created:  ", tb.muted()),
            Span::styled(format_timestamp(e.created_at), tb.subtext()),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Updated:  ", tb.muted()),
            Span::styled(format_timestamp(e.updated_at), tb.subtext()),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  y/c", tb.accent()),
            Span::styled(" copy pw  ", tb.muted()),
            Span::styled("u", tb.accent()),
            Span::styled(" copy user  ", tb.muted()),
            Span::styled("e", tb.accent()),
            Span::styled(" edit  ", tb.muted()),
            Span::styled("d", tb.accent()),
            Span::styled(" delete  ", tb.muted()),
            Span::styled("Esc/q", tb.accent()),
            Span::styled(" back", tb.muted()),
        ]));
        lines.push(Line::from(""));
        lines
    } else {
        vec![Line::from(Span::styled("  No entry selected", tb.muted()))]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(" Entry Detail ", tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, modal_area);
}

fn draw_help(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.size();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let modal_area = centered_rect_pct(55, 80, full);
    f.render_widget(Clear, modal_area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", tb.accent())),
        Line::from(vec![
            Span::styled("    j / Down   ", tb.text()),
            Span::styled("Move selection down", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    k / Up     ", tb.text()),
            Span::styled("Move selection up", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Enter      ", tb.text()),
            Span::styled("View entry detail", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Search", tb.accent())),
        Line::from(vec![
            Span::styled("    /          ", tb.text()),
            Span::styled("Focus search bar", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Esc        ", tb.text()),
            Span::styled("Cancel search", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Clipboard (List & Detail)", tb.accent())),
        Line::from(vec![
            Span::styled("    y / c      ", tb.text()),
            Span::styled("Copy password (auto-clears in 30s)", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    u          ", tb.text()),
            Span::styled("Copy username", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Mouse", tb.accent())),
        Line::from(vec![
            Span::styled("    Click      ", tb.text()),
            Span::styled("Select entry", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Dbl-click  ", tb.text()),
            Span::styled("Title=open, Username/Password=copy", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Scroll     ", tb.text()),
            Span::styled("Navigate up / down", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Actions", tb.accent())),
        Line::from(vec![
            Span::styled("    a          ", tb.text()),
            Span::styled("Add new entry", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    e          ", tb.text()),
            Span::styled("Edit entry (on detail)", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    d          ", tb.text()),
            Span::styled("Delete entry", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Ctrl+G     ", tb.text()),
            Span::styled("Generate password (in form)", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Ctrl+S     ", tb.text()),
            Span::styled("Save form", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    t          ", tb.text()),
            Span::styled("Cycle theme", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    ?          ", tb.text()),
            Span::styled("Toggle this help", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    q / Esc    ", tb.text()),
            Span::styled("Quit", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press q or ? to close", tb.muted())),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(" Keybindings ", tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, modal_area);
}

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.size();

    // Find the entry title for the confirmation message
    let title_str = if let Some(id) = app.confirm_delete {
        app.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.title.clone())
            .unwrap_or_else(|| "this entry".to_string())
    } else {
        "this entry".to_string()
    };

    let modal_width = 44u16;
    let modal_height = 5u16;
    let area = {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(full.height.saturating_sub(modal_height) / 2),
                Constraint::Length(modal_height),
                Constraint::Min(0),
            ])
            .split(full);
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(full.width.saturating_sub(modal_width) / 2),
                Constraint::Length(modal_width),
                Constraint::Min(0),
            ])
            .split(popup_layout[1])[1]
    };

    f.render_widget(Clear, area);

    let msg = format!("  Delete \"{}\"?", title_str);
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(msg, tb.text())),
        Line::from(vec![
            Span::styled("  ", tb.muted()),
            Span::styled("y", tb.red()),
            Span::styled(" yes  ", tb.muted()),
            Span::styled("n / Esc", tb.accent()),
            Span::styled(" cancel", tb.muted()),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.red())
        .title(Span::styled(" Confirm Delete ", tb.red()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}

fn centered_rect_pct(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
