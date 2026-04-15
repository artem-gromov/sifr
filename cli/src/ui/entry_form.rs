use ratatui::{
    layout::Alignment,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let tb = app.theme_bridge();
    let full = f.area();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let is_add = app.form_editing_id.is_none();
    let editing = app.form_editing_field.is_some();

    // Modal sizing: 64 wide, height adapts
    let modal_width = 64u16;
    let modal_height = if is_add { 25u16 } else { 22u16 };
    let area = super::centered_rect(modal_width, modal_height, full);
    app.form_modal_area = Some(area);
    f.render_widget(Clear, area);

    let title = if is_add {
        " Add Entry "
    } else if editing {
        " Edit Entry "
    } else {
        " Entry Detail "
    };

    let mut content: Vec<Line> = vec![Line::from("")];

    for (i, field) in app.form_fields.iter().enumerate() {
        let is_focused = i == app.form_focused;
        let is_editing_this = app.form_editing_field == Some(i);

        let display_value: String = if field.secret && !app.form_password_visible {
            if field.value.is_empty() {
                String::new()
            } else {
                "\u{2022}".repeat(field.value.len())
            }
        } else {
            field.value.clone()
        };

        let label_text = if field.required && editing {
            format!("  {}*", field.label)
        } else {
            format!("  {}", field.label)
        };

        if is_editing_this {
            // Editable input box
            let label_style = tb.accent();
            content.push(Line::from(Span::styled(label_text, label_style)));

            let mut val = display_value.clone();
            val.push('\u{258c}'); // cursor
            let box_inner_width = 48usize;
            let truncated: String = if val.len() > box_inner_width {
                val[val.len() - box_inner_width..].to_string()
            } else {
                val
            };
            let padded = format!("{:<width$}", truncated, width = box_inner_width);
            let input_line = format!("  [{}]", padded);
            content.push(Line::from(Span::styled(input_line, label_style)));

            // Hint for secret fields
            if field.secret {
                content.push(Line::from(Span::styled(
                    "  Ctrl+V toggle  Ctrl+G generate",
                    tb.muted(),
                )));
            } else {
                content.push(Line::from(""));
            }
        } else {
            // Read-only label: value display
            let label_style = if is_focused { tb.accent() } else { tb.muted() };
            let value_style = tb.text();

            let val_display = if display_value.is_empty() {
                "\u{2014}".to_string()
            } else {
                display_value
            };

            content.push(Line::from(vec![
                Span::styled(format!("{:<16}", label_text), label_style),
                Span::styled(val_display, value_style),
            ]));
        }
    }

    // Timestamps for existing entries
    if !is_add {
        content.push(Line::from(""));
        content.push(Line::from(vec![
            Span::styled(format!("{:<16}", "  Created"), tb.muted()),
            Span::styled(format_timestamp(app.form_created_at), tb.muted()),
        ]));
        content.push(Line::from(vec![
            Span::styled(format!("{:<16}", "  Updated"), tb.muted()),
            Span::styled(format_timestamp(app.form_updated_at), tb.muted()),
        ]));
    }

    // Error message
    if let Some(ref err) = app.error_message {
        content.push(Line::from(Span::styled(format!("  {}", err), tb.red())));
    }

    // Pad content to push hints to bottom of modal
    let inner_height = (modal_height - 2) as usize; // subtract top/bottom border
    let hint_line = if editing {
        Line::from(vec![
            Span::styled("  Tab", tb.accent()),
            Span::styled(" next  ", tb.muted()),
            Span::styled("Ctrl+S", tb.accent()),
            Span::styled(" save  ", tb.muted()),
            Span::styled("Esc", tb.accent()),
            if is_add {
                Span::styled(" cancel", tb.muted())
            } else {
                Span::styled(" stop editing", tb.muted())
            },
        ])
    } else {
        Line::from(vec![
            Span::styled("  j/k", tb.accent()),
            Span::styled(" navigate  ", tb.muted()),
            Span::styled("Enter", tb.accent()),
            Span::styled(" edit  ", tb.muted()),
            Span::styled("y", tb.accent()),
            Span::styled(" copy  ", tb.muted()),
            Span::styled("Esc", tb.accent()),
            Span::styled(" close", tb.muted()),
        ])
    };
    while content.len() < inner_height.saturating_sub(1) {
        content.push(Line::from(""));
    }
    content.push(hint_line);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(tb.border())
        .title(Span::styled(title, tb.title()))
        .title_alignment(Alignment::Center)
        .style(tb.surface());

    let para = Paragraph::new(content).block(block);
    f.render_widget(para, area);
}

fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        return "\u{2014}".to_string();
    }
    let secs = ts;
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}", y, m, d, hours, minutes)
}

fn days_to_ymd(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as i64, d as i64)
}
