use ratatui::{
    layout::Alignment,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.size();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    // Modal: 50 wide, tall enough for 5 fields + hints
    let modal_width = 52u16;
    let modal_height = 23u16;
    let area = super::centered_rect(modal_width, modal_height, full);
    f.render_widget(Clear, area);

    let is_add = app.form_editing_id.is_none();
    let title = if is_add {
        " Add Entry "
    } else {
        " Edit Entry "
    };

    let mut content: Vec<Line> = vec![Line::from("")];

    for (i, field) in app.form_fields.iter().enumerate() {
        let is_focused = i == app.form_focused;
        let label_style = if is_focused { tb.accent() } else { tb.muted() };

        let mut display_value: String = if field.secret && !app.form_password_visible {
            "\u{2022}".repeat(field.value.len())
        } else {
            field.value.clone()
        };

        // Add cursor to focused field
        if is_focused {
            display_value.push('\u{258c}'); // ▌
        }

        // Field input box: [value               ]
        let box_inner_width = 36usize;
        let truncated: String = if display_value.len() > box_inner_width {
            display_value[display_value.len() - box_inner_width..].to_string()
        } else {
            display_value.clone()
        };
        let padded = format!("{:<width$}", truncated, width = box_inner_width);
        let input_line = format!("[{}]", padded);

        let label_text = if field.required {
            format!("  {}*", field.label)
        } else {
            format!("  {}", field.label)
        };

        content.push(Line::from(Span::styled(label_text, label_style)));
        content.push(Line::from(Span::styled(
            format!("  {}", input_line),
            label_style,
        )));

        // Extra hint for password field
        if field.secret && is_focused {
            content.push(Line::from(Span::styled(
                "  Ctrl+V toggle  Ctrl+G generate",
                tb.muted(),
            )));
        } else {
            content.push(Line::from(""));
        }
    }

    // Error message
    if let Some(ref err) = app.error_message {
        content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
    } else {
        content.push(Line::from(""));
    }

    content.push(Line::from(vec![
        Span::styled("  Tab", tb.accent()),
        Span::styled(" next field  ", tb.muted()),
        Span::styled("Ctrl+S", tb.accent()),
        Span::styled(" save  ", tb.muted()),
        Span::styled("Esc", tb.accent()),
        Span::styled(" cancel", tb.muted()),
    ]));
    content.push(Line::from(""));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(title, tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}
