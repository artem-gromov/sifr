use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::{app::App, theme_bridge::ThemeBridge};

pub fn draw(f: &mut Frame, app: &App) {
    let tb = app.theme_bridge();

    let full = f.size();
    let bg = Block::default().style(tb.bg());
    f.render_widget(bg, full);

    // Layout: header + search + table + hint + status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // entry table
            Constraint::Length(2), // hint line
            Constraint::Length(1), // status bar
        ])
        .split(full);

    draw_search_bar(f, app, &tb, chunks[0]);
    draw_table(f, app, &tb, chunks[1]);
    draw_hints(f, app, &tb, chunks[2]);
    crate::ui::status_bar::draw(f, app, chunks[3]);
}

fn draw_search_bar(f: &mut Frame, app: &App, tb: &ThemeBridge<'_>, area: Rect) {
    let query = &app.search_query;
    let vault_name = std::path::Path::new(&app.vault_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&app.vault_path);

    let search_content = if app.search_active {
        format!("Search: {}_", query)
    } else if query.is_empty() {
        "Search: (press / to search)".into()
    } else {
        format!("Search: {}", query)
    };

    let search_style = if app.search_active {
        tb.accent()
    } else {
        tb.muted()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(tb.border())
        .title(Span::styled(format!(" Sifr  {} ", vault_name), tb.title()))
        .style(tb.surface());

    let para =
        Paragraph::new(Span::styled(format!(" {}", search_content), search_style)).block(block);
    f.render_widget(para, area);
}

fn draw_table(f: &mut Frame, app: &App, tb: &ThemeBridge<'_>, area: Rect) {
    let entries = app.filtered_entries();

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let marker = if i == app.selected_index { ">" } else { " " };
            let cells = vec![
                Cell::from(Span::styled(marker, tb.accent())),
                Cell::from(Span::styled(e.title.clone(), tb.text())),
                Cell::from(Span::styled(e.url.clone(), tb.subtext())),
                Cell::from(Span::styled(e.username.clone(), tb.muted())),
                Cell::from(Span::styled(e.category.clone(), tb.blue())),
            ];
            let style = if i == app.selected_index {
                tb.selection()
            } else {
                tb.bg()
            };
            Row::new(cells).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(20),
        Constraint::Length(22),
        Constraint::Min(20),
        Constraint::Length(10),
    ];

    let header = Row::new(vec![
        Cell::from(""),
        Cell::from(Span::styled("Title", tb.accent())),
        Cell::from(Span::styled("URL", tb.accent())),
        Cell::from(Span::styled("Username", tb.accent())),
        Cell::from(Span::styled("Category", tb.accent())),
    ])
    .style(tb.surface());

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_type(BorderType::Rounded)
                .border_style(tb.border())
                .style(tb.bg()),
        )
        .highlight_style(tb.selection());

    let mut state = TableState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_hints(f: &mut Frame, _app: &App, tb: &ThemeBridge<'_>, area: Rect) {
    let hints = vec![Line::from(vec![
        Span::styled(" jk", tb.accent()),
        Span::styled(" navigate", tb.muted()),
        Span::styled("  Enter", tb.accent()),
        Span::styled(" view", tb.muted()),
        Span::styled("  a", tb.accent()),
        Span::styled(" add", tb.muted()),
        Span::styled("  /", tb.accent()),
        Span::styled(" search", tb.muted()),
        Span::styled("  t", tb.accent()),
        Span::styled(" theme", tb.muted()),
        Span::styled("  ?", tb.accent()),
        Span::styled(" help", tb.muted()),
        Span::styled("  q", tb.accent()),
        Span::styled(" quit", tb.muted()),
    ])];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let sep = Block::default()
        .borders(Borders::TOP)
        .border_style(tb.border());
    f.render_widget(sep, chunks[0]);

    let hint_para = Paragraph::new(hints).style(tb.status_bar());
    f.render_widget(hint_para, chunks[1]);
}
