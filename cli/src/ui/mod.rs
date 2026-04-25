pub mod entry_form;
pub mod entry_list;
pub mod status_bar;
pub mod unlock;
pub mod vault_picker;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Screen};
use crate::theme_bridge::ThemeBridge;

pub fn format_inline_input(
    value: &str,
    cursor: usize,
    width: usize,
    mask: bool,
    show_cursor: bool,
) -> String {
    let rendered = if mask {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.to_string()
    };

    let chars: Vec<char> = rendered.chars().collect();
    let cursor = cursor.min(chars.len());
    let text_width = if show_cursor {
        width.saturating_sub(1)
    } else {
        width
    };
    let mut start = cursor.saturating_sub(text_width);
    if chars.len().saturating_sub(start) > text_width {
        start = chars.len().saturating_sub(text_width);
    }
    let end = (start + text_width).min(chars.len());

    let mut visible: Vec<char> = chars[start..end].to_vec();
    let cursor_pos = cursor.saturating_sub(start).min(visible.len());
    if show_cursor {
        visible.insert(cursor_pos, '\u{258c}');
    }

    let mut text: String = visible.into_iter().collect();
    let count = text.chars().count();
    if count < width {
        text.push_str(&" ".repeat(width - count));
    }
    text
}

pub fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(r.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(r.width.saturating_sub(width) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::VaultPicker => vault_picker::draw(f, app),
        Screen::Unlock => unlock::draw(f, app),
        Screen::EntryList => entry_list::draw(f, app),
        Screen::Help => draw_help(f, app),
        Screen::AddEntry | Screen::EditEntry => entry_form::draw(f, app),
    }

    // Delete confirmation overlay (drawn on top of any screen)
    if app.confirm_delete.is_some() {
        draw_delete_confirm(f, app);
    }
}

fn build_help_content(tb: &ThemeBridge) -> Vec<Line<'_>> {
    vec![
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
            Span::styled("    Enter / e  ", tb.text()),
            Span::styled("Edit selected entry", tb.muted()),
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
            Span::styled("    y          ", tb.text()),
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
            Span::styled("    Click bar  ", tb.text()),
            Span::styled("Focus search", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Dbl-click  ", tb.text()),
            Span::styled("Title=edit, Username/Password=copy", tb.muted()),
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
            Span::styled("    d          ", tb.text()),
            Span::styled("Delete entry", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    f          ", tb.text()),
            Span::styled("Toggle favorite", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    L          ", tb.text()),
            Span::styled("Lock vault", tb.muted()),
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
            Span::styled("Copy TOTP code", tb.muted()),
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
        Line::from(Span::styled("  CLI Commands", tb.accent())),
        Line::from(vec![
            Span::styled("  sifr gen [opts]        ", tb.text()),
            Span::styled("Generate password", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("  sifr export <vault>     ", tb.text()),
            Span::styled("Export vault to JSON", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("  sifr import <vault>     ", tb.text()),
            Span::styled("Import CSV entries", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("    CSV: title,username,password,url,notes,totp_secret", tb.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Password strength", tb.accent())),
        Line::from(vec![
            Span::styled("  Weak    ", tb.red()),
            Span::styled(" < 60 bits entropy", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("  Medium  ", tb.yellow()),
            Span::styled(" 60-80 bits", tb.muted()),
        ]),
        Line::from(vec![
            Span::styled("  Strong  ", tb.green()),
            Span::styled(" > 80 bits", tb.muted()),
        ]),
        Line::from(""),
    ]
}

fn draw_help(f: &mut Frame, app: &mut App) {
    let tb = app.theme_bridge();
    let full = f.area();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let modal_area = centered_rect_pct(55, 80, full);
    f.render_widget(Clear, modal_area);

    let help_lines = build_help_content(&tb);

    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let total_lines = help_lines.len();
    let visible_lines = inner_height.saturating_sub(1);
    let max_offset = total_lines.saturating_sub(visible_lines);

    app.help_total_lines = total_lines;
    app.help_visible_lines = visible_lines;
    app.help_scroll_offset = app.help_scroll_offset.min(max_offset);
    let scroll_offset = app.help_scroll_offset;

    let visible: Vec<Line> = help_lines
        .into_iter()
        .skip(scroll_offset)
        .take(visible_lines)
        .collect();

    let content_len = visible.len();
    let missing = (inner_height.saturating_sub(1)).saturating_sub(content_len);
    let mut output = visible;
    for _ in 0..missing {
        output.push(Line::from(""));
    }
    output.push(Line::from(Span::styled(
        "  j/k scroll \u{2022} q/? close",
        tb.muted(),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(tb.border())
        .title(Span::styled(" Keybindings ", tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(output).block(block);
    f.render_widget(para, modal_area);

    if total_lines > visible_lines {
        let thumb_range = max_offset.saturating_add(1);
        let mut scroll_state = ScrollbarState::new(thumb_range).position(scroll_offset);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(tb.accent())
                .track_style(tb.muted()),
            modal_area,
            &mut scroll_state,
        );
    }
}

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.area();

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
