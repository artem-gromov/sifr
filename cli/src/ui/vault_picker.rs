use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();
    let full = f.size();

    // Background
    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    // Outer layout: top title bar, middle list, bottom hints
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(1),    // list
            Constraint::Length(2), // hints
        ])
        .split(full);

    // ── Title bar ──────────────────────────────────────────────────────────
    let path_str = app.picker_path.display().to_string();
    let title_text = format!(" Select Vault  {path_str} ");
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .style(tb.surface());
    let title_para = Paragraph::new(Span::styled(title_text, tb.title()))
        .block(title_block)
        .alignment(Alignment::Left);
    f.render_widget(title_para, chunks[0]);

    // ── File list ──────────────────────────────────────────────────────────
    let list_area = chunks[1];
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .style(tb.bg());
    f.render_widget(list_block.clone(), list_area);

    // Inner area (inside the block borders)
    let inner = list_block.inner(list_area);
    let visible_height = inner.height as usize;
    let offset = app.picker_scroll_offset;
    let entries = &app.picker_entries;

    let rows: Vec<Line> = entries
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_height)
        .map(|(i, entry)| {
            let icon = if entry.name == ".." {
                "^ "
            } else if entry.is_dir {
                "> "
            } else if entry.is_vault {
                "* "
            } else {
                "  "
            };

            let label = if entry.is_dir && entry.name != ".." {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };

            let text = format!(" {icon}{label}");

            if i == app.picker_selected {
                Line::from(Span::styled(text, tb.selection()))
            } else if entry.is_vault {
                Line::from(Span::styled(text, tb.accent().add_modifier(Modifier::BOLD)))
            } else if entry.is_dir {
                Line::from(Span::styled(text, tb.blue()))
            } else {
                Line::from(Span::styled(text, tb.text()))
            }
        })
        .collect();

    let list_para = Paragraph::new(rows);
    f.render_widget(list_para, inner);

    // Scrollbar indicator (right edge percentage)
    if entries.len() > visible_height && visible_height > 0 {
        let scroll_pct = if entries.len() <= 1 {
            0
        } else {
            offset * (inner.height as usize).saturating_sub(1) / (entries.len() - 1)
        };
        let indicator_row = inner.y + scroll_pct as u16;
        let indicator_col = inner.x + inner.width; // one past inner right edge (border col)
        if indicator_col < full.width && indicator_row < inner.y + inner.height {
            let scrollbar = Paragraph::new(Span::styled("▐", tb.muted()));
            let scrollbar_area = ratatui::layout::Rect {
                x: indicator_col.saturating_sub(1),
                y: indicator_row,
                width: 1,
                height: 1,
            };
            if scrollbar_area.x < full.width && scrollbar_area.y < full.height {
                f.render_widget(scrollbar, scrollbar_area);
            }
        }
    }

    // ── Bottom hints ───────────────────────────────────────────────────────
    let hints = Line::from(vec![
        Span::styled("  j/k", tb.accent()),
        Span::styled(" navigate  ", tb.muted()),
        Span::styled("Enter", tb.accent()),
        Span::styled(" open  ", tb.muted()),
        Span::styled("n", tb.accent()),
        Span::styled(" new vault  ", tb.muted()),
        Span::styled("~", tb.accent()),
        Span::styled(" home  ", tb.muted()),
        Span::styled("q", tb.accent()),
        Span::styled(" quit", tb.muted()),
    ]);
    let hints_block = Block::default().style(tb.bg());
    let hints_para = Paragraph::new(hints).block(hints_block);
    f.render_widget(hints_para, chunks[2]);
}
