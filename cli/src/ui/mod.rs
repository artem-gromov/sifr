pub mod entry_list;
pub mod status_bar;
pub mod unlock;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::{App, Screen},
    theme_bridge::ThemeBridge,
};

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Unlock => unlock::draw(f, app),
        Screen::EntryList => entry_list::draw(f, app),
        Screen::EntryDetail => draw_entry_detail(f, app),
        Screen::Help => draw_help(f, app),
    }
}

fn draw_entry_detail(f: &mut Frame, app: &App) {
    let palette = &app.theme.active().palette;
    let tb = ThemeBridge::new(palette);
    let full = f.size();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let entries = app.filtered_entries();
    let entry = entries.get(app.selected_index);

    let modal_area = centered_rect_pct(60, 50, full);
    f.render_widget(Clear, modal_area);

    let content = if let Some(e) = entry {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Title:    ", tb.muted()),
                Span::styled(e.title.clone(), tb.text()),
            ]),
            Line::from(vec![
                Span::styled("  URL:      ", tb.muted()),
                Span::styled(e.url.clone(), tb.blue()),
            ]),
            Line::from(vec![
                Span::styled("  Username: ", tb.muted()),
                Span::styled(e.username.clone(), tb.text()),
            ]),
            Line::from(vec![
                Span::styled("  Password: ", tb.muted()),
                Span::styled("**********", tb.accent()),
            ]),
            Line::from(vec![
                Span::styled("  Category: ", tb.muted()),
                Span::styled(e.category.clone(), tb.purple()),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Esc/q to go back", tb.muted())),
            Line::from(""),
        ]
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
    let palette = &app.theme.active().palette;
    let tb = ThemeBridge::new(palette);
    let full = f.size();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let modal_area = centered_rect_pct(50, 70, full);
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
        Line::from(Span::styled("  Actions", tb.accent())),
        Line::from(vec![
            Span::styled("    a          ", tb.text()),
            Span::styled("Add new entry (coming soon)", tb.muted()),
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
