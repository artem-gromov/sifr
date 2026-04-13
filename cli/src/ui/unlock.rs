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
    let vault_display = &app.vault_path;

    // Width adapts to vault path length (min 44, max 72)
    let path_width = vault_display.len() as u16 + 6;
    let modal_width = path_width.clamp(44, 72);
    let modal_height = if is_create { 15u16 } else { 13u16 };
    let area = centered_rect(modal_width, modal_height, full);

    // Clear behind modal
    f.render_widget(Clear, area);

    // Input box inner width adapts to modal
    let box_inner = (modal_width as usize).saturating_sub(8).max(20);

    let password_display = if app.password_visible {
        app.password_input.clone()
    } else {
        "\u{2022}".repeat(app.password_input.len())
    };

    let confirm_display = if app.password_visible {
        app.password_confirm.clone()
    } else {
        "\u{2022}".repeat(app.password_confirm.len())
    };

    // Cursor marker on the active field
    let pw_active = !is_create || !app.confirm_active;
    let confirm_active = is_create && app.confirm_active;

    let pw_text = if pw_active {
        format!("{}▌", password_display)
    } else {
        password_display
    };
    let input_line = format!("[{:<width$}]", pw_text, width = box_inner);

    let confirm_text = if confirm_active {
        format!("{}▌", confirm_display)
    } else {
        confirm_display
    };
    let confirm_line = format!("[{:<width$}]", confirm_text, width = box_inner);

    let mut content = vec![Line::from("")];

    if is_create {
        content.push(Line::from(Span::styled(
            format!("  {}", vault_display),
            tb.subtext(),
        )));
        content.push(Line::from(""));

        // Password field
        let pw_style = if pw_active { tb.accent() } else { tb.muted() };
        content.push(Line::from(Span::styled("  Master Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", input_line),
            pw_style,
        )));
        content.push(Line::from(""));

        // Confirm field
        let cf_style = if confirm_active {
            tb.accent()
        } else {
            tb.muted()
        };
        content.push(Line::from(Span::styled("  Confirm Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", confirm_line),
            cf_style,
        )));
        content.push(Line::from(""));

        // Error or hints
        if let Some(ref err) = app.error_message {
            content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
        } else {
            content.push(Line::from(vec![
                Span::styled("  Enter", tb.accent()),
                Span::styled(" confirm  ", tb.muted()),
                Span::styled("Tab", tb.accent()),
                Span::styled(" switch field  ", tb.muted()),
                Span::styled("Esc", tb.accent()),
                Span::styled(" back", tb.muted()),
            ]));
        }
    } else {
        content.push(Line::from(Span::styled(
            format!("  {}", vault_display),
            tb.subtext(),
        )));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled("  Master Password:", tb.text())));
        content.push(Line::from(Span::styled(
            format!("  {}", input_line),
            tb.accent(),
        )));
        content.push(Line::from(""));

        // Error or hints
        if let Some(ref err) = app.error_message {
            content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
        } else {
            content.push(Line::from(vec![
                Span::styled("  Enter", tb.accent()),
                Span::styled(" unlock  ", tb.muted()),
                Span::styled("Tab", tb.accent()),
                Span::styled(" browse  ", tb.muted()),
                Span::styled("Esc", tb.accent()),
                Span::styled(" quit", tb.muted()),
            ]));
        }
    }

    content.push(Line::from(""));

    let title = if is_create {
        " Create New Vault "
    } else {
        " Unlock Vault "
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
