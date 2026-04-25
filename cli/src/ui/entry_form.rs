use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, FIELD_INDEX_PASSWORD};
use crate::ui::format_inline_input;

const PASSWORD_FIELD_INDEX: usize = FIELD_INDEX_PASSWORD;

pub fn draw(f: &mut Frame, app: &mut App) {
    let tb = app.theme_bridge();
    let full = f.area();

    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    let is_add = app.form_editing_id.is_none();

    // Modal sizing: clamp to viewport so resizing cannot push content outside frame.
    let modal_width = 64u16.min(full.width.saturating_sub(2).max(1));
    let modal_height = (if is_add { 32u16 } else { 30u16 })
        .min(full.height.saturating_sub(2).max(1));
    let area = super::centered_rect(modal_width, modal_height, full);
    app.form_modal_area = Some(area);
    f.render_widget(Clear, area);

    let title = if is_add {
        " Add Entry "
    } else {
        " Entry Detail "
    };

    let mut content: Vec<Line> = vec![Line::from("")];
    let mut field_rows: Vec<(u16, u16)> = Vec::new();
    let mut notes_textarea_area: Option<Rect> = None;
    // Current row inside the modal (1 for top border, 1 for initial empty line)
    let content_start_y = area.y + 1;
    let value_width = area.width.saturating_sub(20) as usize;
    app.form_totp_row = None;

    for (i, field) in app.form_fields.iter().enumerate() {
        let is_focused = i == app.form_focused;
        let is_editing_this = app.form_editing_field == Some(i);

        // Secret fields are visible only when being edited
        let field_visible = is_editing_this;
        let display_value: String = if field.secret && !field_visible {
            if field.value.is_empty() {
                String::new()
            } else {
                "\u{2022}".repeat(field.value.len())
            }
        } else {
            field.value.clone()
        };

        let label_text = if field.required && is_add {
            format!("  {}*", field.label)
        } else {
            format!("  {}", field.label)
        };

        let row_start = content_start_y + content.len() as u16;

        if is_editing_this && field.label == "Notes" {
            let label_style = tb.accent();
            content.push(Line::from(vec![
                Span::styled(format!("{:<16}", label_text), label_style),
                Span::styled("", tb.text()),
            ]));

            let textarea_height = 5u16;
            let textarea_y = content_start_y + content.len() as u16 - 1;
            let padding = " ".repeat(16);
            for _ in 1..5 {
                content.push(Line::from(Span::styled(padding.clone(), label_style)));
            }

            notes_textarea_area = Some(Rect {
                x: area.x + 1 + 16,
                y: textarea_y,
                width: value_width as u16,
                height: textarea_height,
            });
        } else if is_editing_this {
            let label_style = tb.accent();
            let box_inner_width = value_width;
            let cursor = app
                .form_field_cursors
                .get(i)
                .copied()
                .unwrap_or_else(|| field.value.chars().count());
            let padded = format_inline_input(
                &field.value,
                cursor,
                box_inner_width,
                false,
                true,
            );
            content.push(Line::from(vec![
                Span::styled(format!("{:<16}", label_text), label_style),
                Span::styled(format!("[{}]", padded), label_style),
            ]));

            if i == PASSWORD_FIELD_INDEX && !field.value.is_empty() {
                let strength = sifr_core::crypto::calculate_password_strength(&field.value);
                let strength_color = match strength {
                    sifr_core::crypto::PasswordStrength::Weak => tb.red(),
                    sifr_core::crypto::PasswordStrength::Medium => tb.yellow(),
                    sifr_core::crypto::PasswordStrength::Strong => tb.green(),
                };
                content.push(Line::from(vec![
                    Span::styled(format!("{:<16}", ""), label_style),
                    Span::styled(format!("{:?}", strength), strength_color),
                ]));
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

            if field.label == "Notes" {
                // Notes always occupies 5 lines, aligned with other fields
                let max_line_width = value_width;
                let mut note_lines: Vec<String> = Vec::new();
                if !val_display.is_empty() && val_display != "\u{2014}" {
                    for line in val_display.lines() {
                        let mut remaining = line;
                        while !remaining.is_empty() {
                            let (chunk, rest) = if remaining.len() > max_line_width {
                                remaining.split_at(max_line_width)
                            } else {
                                (remaining, "")
                            };
                            note_lines.push(chunk.to_string());
                            remaining = rest;
                        }
                    }
                }
                let first_val = if note_lines.is_empty() {
                    "\u{2014}".to_string()
                } else {
                    note_lines[0].clone()
                };
                content.push(Line::from(vec![
                    Span::styled(format!("{:<16}", label_text), label_style),
                    Span::styled(first_val, value_style),
                ]));
                let padding = " ".repeat(16);
                for j in 1..5 {
                    let line_val = note_lines.get(j).map(|s| s.as_str()).unwrap_or("");
                    content.push(Line::from(vec![
                        Span::styled(padding.clone(), label_style),
                        Span::styled(line_val.to_string(), value_style),
                    ]));
                }
            } else {
                content.push(Line::from(vec![
                    Span::styled(format!("{:<16}", label_text), label_style),
                    Span::styled(val_display, value_style),
                ]));
            }
        }

        let row_end = content_start_y + content.len() as u16;
        field_rows.push((row_start, row_end));

        // Show generated TOTP code right after Password when not editing password
        if field.label == "Password" && !is_editing_this {
            let totp_secret = app.form_fields.get(3).map(|f| f.value.as_str()).unwrap_or("");
            if !totp_secret.is_empty() {
                if let Ok((code, remaining)) = sifr_core::crypto::generate_totp(totp_secret) {
                    let totp_row_start = content_start_y + content.len() as u16;
                    let secs_style = if remaining <= 5 { tb.red() } else { tb.muted() };
                    content.push(Line::from(vec![
                        Span::styled(format!("{:<16}", "  TOTP Code"), tb.muted()),
                        Span::styled(format!("{} {}", &code[..3], &code[3..]), tb.text()),
                        Span::styled(format!(" {:2}s", remaining), secs_style),
                    ]));
                    app.form_totp_row = Some((totp_row_start, content_start_y + content.len() as u16));
                }
            }
        }
    }

    app.form_field_rows = field_rows;

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

    // Clipboard notification with countdown
    if let Some(clear_at) = app.clipboard_clear_at {
        let now = std::time::Instant::now();
        if now < clear_at {
            let remaining = (clear_at - now).as_secs();
            let label = app.clipboard_notification.as_deref().unwrap_or("Copied");
            content.push(Line::from(Span::styled(
                format!("  {} \u{2022} clears in {}s", label, remaining),
                tb.accent(),
            )));
        }
    } else if let Some(ref notification) = app.clipboard_notification {
        content.push(Line::from(Span::styled(
            format!("  {}", notification),
            tb.accent(),
        )));
    }

    // Pad content to push hints to bottom of the *actual* rendered modal.
    let inner_height = area.height.saturating_sub(2) as usize; // subtract top/bottom border
    let editing = app.form_editing_field.is_some();
    let hint_line = if editing {
        Line::from(vec![
            Span::styled("  Tab", tb.accent()),
            Span::styled(" next  ", tb.muted()),
            Span::styled("Ctrl+S", tb.accent()),
            Span::styled(" save  ", tb.muted()),
            Span::styled("Ctrl+G", tb.accent()),
            Span::styled(" gen pw  ", tb.muted()),
            Span::styled("Esc", tb.accent()),
            if is_add {
                Span::styled(" cancel", tb.muted())
            } else {
                Span::styled(" stop", tb.muted())
            },
        ])
    } else {
        Line::from(vec![
            Span::styled("  j/k", tb.accent()),
            Span::styled(" nav  ", tb.muted()),
            Span::styled("Enter", tb.accent()),
            Span::styled(" edit  ", tb.muted()),
            Span::styled("y", tb.accent()),
            Span::styled(" copy  ", tb.muted()),
            Span::styled("Esc", tb.accent()),
            Span::styled(" close", tb.muted()),
        ])
    };
    let body_capacity = inner_height.saturating_sub(1);
    if content.len() > body_capacity {
        content.truncate(body_capacity);
    }
    while content.len() < body_capacity {
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

    if let (Some(textarea_area), Some(textarea)) =
        (notes_textarea_area, app.form_notes_textarea.as_mut())
    {
        textarea.set_style(tb.text());
        textarea.set_cursor_line_style(tb.accent());
        textarea.set_block(Block::default().style(tb.surface()));
        f.render_widget(&*textarea, textarea_area);
    }
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
