use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();

    // Full screen background
    let full = f.size();
    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    // Center the modal
    let modal_width = 36u16;
    let modal_height = 10u16;
    let area = centered_rect(modal_width, modal_height, full);

    // Clear behind modal
    f.render_widget(Clear, area);

    let password_display = if app.password_visible {
        app.password_input.clone()
    } else {
        "\u{2022}".repeat(app.password_input.len())
    };
    let input_line = format!("[{:<20}]", password_display);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled("  Sifr", tb.title())),
        Line::from(""),
        Line::from(Span::styled("  Master Password:", tb.text())),
        Line::from(Span::styled(format!("  {}", input_line), tb.accent())),
        Line::from(""),
        Line::from(Span::styled("  Enter unlock  q quit", tb.muted())),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(" Sifr Password Manager ", tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let modal = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(modal, area);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
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
