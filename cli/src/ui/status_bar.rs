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
            let bar = Paragraph::new(line);
            f.render_widget(bar, area);
            return;
        }
    }

    // Show error notification (e.g., "Clipboard unavailable")
    if let Some(ref notification) = app.clipboard_notification {
        if app.clipboard_clear_at.is_none() {
            let line = Line::from(vec![
                Span::styled(" ", tb.muted()),
                Span::styled(notification.clone(), tb.accent()),
            ]);
            let bar = Paragraph::new(line);
            f.render_widget(bar, area);
            return;
        }
    }

    let entry_count = app.filtered_entries().len();

    let lock_status = Span::styled(" unlocked", tb.green());
    let count_span = Span::styled(format!(" {} entries", entry_count), tb.subtext());
    let sep = Span::styled(" | ", tb.muted());

    let line = Line::from(vec![count_span, sep, lock_status]);

    let bar = Paragraph::new(line);
    f.render_widget(bar, area);
}
