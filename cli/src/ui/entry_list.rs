use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, HighlightSpacing, Padding, Paragraph, Row, Table,
        TableState,
    },
    Frame,
};

use crate::app::App;
use crate::theme_bridge::ThemeBridge;

fn totp_cell<'a>(entry: &sifr_core::models::Entry, tb: &ThemeBridge) -> Line<'a> {
    let Some(ref secret) = entry.totp_secret else {
        return Line::from("");
    };
    let Ok((code, remaining)) = sifr_core::crypto::generate_totp(secret) else {
        return Line::from(Span::styled("err", tb.red()));
    };
    let secs_style = if remaining <= 5 { tb.red() } else { tb.muted() };
    Line::from(vec![
        Span::styled(format!("{} {}", &code[..3], &code[3..]), tb.accent()),
        Span::styled(format!(" {:2}s", remaining), secs_style),
    ])
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let full = f.area();
    {
        let tb = app.theme_bridge();
        let bg = Block::default().style(tb.bg());
        f.render_widget(bg, full);
    }

    // Layout: search bar + table + hint + status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // entry table
            Constraint::Length(2), // hint line
            Constraint::Length(1), // status bar
        ])
        .split(full);

    draw_search_bar(f, app, chunks[0]);
    draw_table(f, app, chunks[1]);
    draw_hints(f, app, chunks[2]);
    crate::ui::status_bar::draw(f, app, chunks[3]);
}

fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let tb = app.theme_bridge();
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
        .border_style(tb.border())
        .title(Span::styled(format!(" Sifr  {} ", vault_name), tb.title()))
        .style(tb.surface());

    let para =
        Paragraph::new(Span::styled(format!(" {}", search_content), search_style)).block(block);
    f.render_widget(para, area);
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Compute column layout for double-click detection BEFORE borrowing app.
    // highlight_symbol "▸ " takes 2 chars on the left.
    let highlight_w: u16 = 2;
    let border_x = area.x + 1;
    let padding: u16 = 4; // Padding::horizontal(2) = 2 each side
    let spacing: u16 = 2; // column_spacing(2)
    let available = area.width.saturating_sub(2 + padding + highlight_w); // borders + padding + marker
    let num_gaps: u16 = 4; // 5 columns → 4 gaps
    let fixed: u16 = 10 + 11 + 3 + num_gaps * spacing; // password + totp + fav + gaps
    let flex = available.saturating_sub(fixed);
    let title_w = (flex * 55 / 100).max(15);
    let username_w = flex.saturating_sub(title_w).max(10);

    let col_start = border_x + padding / 2 + highlight_w;
    app.column_boundaries = vec![
        col_start,                                                           // Title start
        col_start + title_w + spacing,                                       // Username start
        col_start + title_w + spacing + username_w + spacing,                // Password start
        col_start + title_w + spacing + username_w + spacing + 10 + spacing, // TOTP start
        col_start + title_w + spacing + username_w + spacing + 10 + spacing + 11 + spacing, // Fav start
    ];

    // Scroll: compute visible height (area minus borders minus header minus header margin)
    let visible_height = area.height.saturating_sub(4) as usize; // 2 border + 1 header + 1 margin

    // Clamp scroll offset so selected_index is visible
    if app.selected_index < app.entry_scroll_offset {
        app.entry_scroll_offset = app.selected_index;
    }
    if visible_height > 0 && app.selected_index >= app.entry_scroll_offset + visible_height {
        app.entry_scroll_offset = app.selected_index - visible_height + 1;
    }

    let scroll_offset = app.entry_scroll_offset;
    let selected = app.selected_index;

    let tb = app.theme_bridge();
    let entries = app.filtered_entries();

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(_, e)| {
            let fav = if e.favorite { "\u{2605}" } else { " " };
            let cells = vec![
                Cell::from(Span::styled(e.title.clone(), tb.accent())),
                Cell::from(Span::styled(
                    e.username.as_deref().unwrap_or("").to_string(),
                    tb.accent(),
                )),
                Cell::from(Span::styled(
                    "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                    tb.accent(),
                )),
                Cell::from(totp_cell(e, &tb)),
                Cell::from(Span::styled(fav.to_string(), tb.accent())),
            ];
            Row::new(cells)
        })
        .collect();

    let widths = [
        Constraint::Length(title_w),    // Title
        Constraint::Length(username_w), // Username
        Constraint::Length(10),         // Password
        Constraint::Length(11),         // TOTP
        Constraint::Length(3),          // Fav
    ];

    let header = Row::new(vec![
        Cell::from(Span::styled("Title", tb.text())),
        Cell::from(Span::styled("Username", tb.text())),
        Cell::from(Span::styled("Password", tb.text())),
        Cell::from(Span::styled("TOTP", tb.text())),
        Cell::from(Span::styled("Fav", tb.text())),
    ])
    .style(Style::new().bold())
    .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(spacing)
        .highlight_symbol("▸ ")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::new().bold().reversed())
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .padding(Padding::horizontal(2))
                .border_style(tb.border())
                .style(tb.surface())
        );

    let mut state = TableState::default();
    state.select(Some(selected - scroll_offset));
    f.render_stateful_widget(table, area, &mut state);
}

fn draw_hints(f: &mut Frame, app: &App, area: Rect) {
    let tb = app.theme_bridge();
    let hints = vec![Line::from(vec![
        Span::styled(" jk", tb.accent()),
        Span::styled(" nav", tb.muted()),
        Span::styled("  Enter/e", tb.accent()),
        Span::styled(" edit", tb.muted()),
        Span::styled("  a", tb.accent()),
        Span::styled(" add", tb.muted()),
        Span::styled("  d", tb.accent()),
        Span::styled(" del", tb.muted()),
        Span::styled("  f", tb.accent()),
        Span::styled(" fav", tb.muted()),
        Span::styled("  y", tb.accent()),
        Span::styled(" pw", tb.muted()),
        Span::styled("  u", tb.accent()),
        Span::styled(" user", tb.muted()),
        Span::styled("  t", tb.accent()),
        Span::styled(" totp", tb.muted()),
        Span::styled("  /", tb.accent()),
        Span::styled(" search", tb.muted()),
        Span::styled("  L", tb.accent()),
        Span::styled(" lock", tb.muted()),
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

    let hint_para = Paragraph::new(hints);
    f.render_widget(hint_para, chunks[1]);
}
