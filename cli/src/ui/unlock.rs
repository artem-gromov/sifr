use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, UnlockMode};

pub fn draw(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();

    // Full screen background
    let full = f.size();
    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let is_create = app.unlock_mode == UnlockMode::Create;

    // Taller modal for create mode (two fields + error line)
    let modal_width = 42u16;
    let modal_height = if is_create { 14u16 } else { 12u16 };
    let area = centered_rect(modal_width, modal_height, full);

    // Clear behind modal
    f.render_widget(Clear, area);

    let vault_name = std::path::Path::new(&app.vault_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&app.vault_path);

    let password_display = if app.password_visible {
        app.password_input.clone()
    } else {
        "\u{2022}".repeat(app.password_input.len())
    };
    let input_line = format!("[{:<22}]", password_display);

    let mut content = vec![Line::from("")];

    if is_create {
        content.push(Line::from(Span::styled(
            format!("  {}", vault_name),
            tb.subtext(),
        )));
        content.push(Line::from(""));

        // Password field
        let pw_style = if !app.confirm_active {
            tb.accent()
        } else {
            tb.muted()
        };
        content.push(Line::from(Span::styled("  Master Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", input_line),
            pw_style,
        )));
        content.push(Line::from(""));

        // Confirm field
        let confirm_display = if app.password_visible {
            app.password_confirm.clone()
        } else {
            "\u{2022}".repeat(app.password_confirm.len())
        };
        let confirm_line = format!("[{:<22}]", confirm_display);
        let confirm_style = if app.confirm_active {
            tb.accent()
        } else {
            tb.muted()
        };
        content.push(Line::from(Span::styled("  Confirm Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", confirm_line),
            confirm_style,
        )));
        content.push(Line::from(""));

        // Error message or hint
        if let Some(ref err) = app.error_message {
            content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
        } else {
            content.push(Line::from(Span::styled(
                "  Enter confirm  Esc back",
                tb.muted(),
            )));
        }
    } else {
        content.push(Line::from(Span::styled("  Sifr", tb.title())));
        content.push(Line::from(Span::styled(
            format!("  {}", vault_name),
            tb.subtext(),
        )));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled("  Master Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", input_line),
            tb.accent(),
        )));
        content.push(Line::from(""));

        // Error message or hint
        if let Some(ref err) = app.error_message {
            content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
        } else {
            content.push(Line::from(Span::styled(
                "  Enter unlock  Esc quit",
                tb.muted(),
            )));
        }
    }

    content.push(Line::from(""));

    let title = if is_create {
        " Create New Vault "
    } else {
        " Sifr Password Manager "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(title, tb.title()))
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
