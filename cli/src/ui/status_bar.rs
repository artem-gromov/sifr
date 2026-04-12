use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let tb = app.theme_bridge();

    // Show clipboard countdown when active
    if let Some(clear_at) = app.clipboard_clear_at {
        let now = std::time::Instant::now();
        if now < clear_at {
            let remaining = (clear_at - now).as_secs();
            let msg = format!(" Copied! Clears in {}s ", remaining);
            let line = Line::from(Span::styled(msg, tb.accent()));
            let bar = Paragraph::new(line).style(tb.status_bar());
            f.render_widget(bar, area);
            return;
        }
    }

    // Show error notification (e.g., "Clipboard unavailable")
    if let Some(ref notification) = app.clipboard_notification {
        if app.clipboard_clear_at.is_none() {
            let line = Line::from(vec![
                Span::styled(" ", tb.status_bar()),
                Span::styled(notification.clone(), tb.accent()),
            ]);
            let bar = Paragraph::new(line).style(tb.status_bar());
            f.render_widget(bar, area);
            return;
        }
    }

    let theme_name = match app.theme.active() {
        Some(t) => t.name.clone(),
        None => "Terminal".to_string(),
    };
    let entry_count = app.filtered_entries().len();

    let lock_status = Span::styled(" unlocked", tb.green());
    let theme_span = Span::styled(format!(" {} ", theme_name), tb.accent());
    let sep = Span::styled(" | ", tb.muted());
    let count_span = Span::styled(format!("{} entries", entry_count), tb.subtext());
    let lock_icon = Span::styled(" | ", tb.muted());

    let line = Line::from(vec![theme_span, sep, count_span, lock_icon, lock_status]);

    let bar = Paragraph::new(line).style(tb.status_bar());
    f.render_widget(bar, area);
}
