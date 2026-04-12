use ratatui::style::{Color, Modifier, Style};
use sifr_core::theme::Palette;

fn hex_to_rgb(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

/// A bridge between a `sifr-core` `Palette` and Ratatui `Style` values.
///
/// When constructed with `ThemeBridge::terminal()` (i.e. `palette` is `None`),
/// every method returns `Style::default()` so the terminal's own colors show
/// through.  A few methods add modifiers (Bold, Reversed) so that UI elements
/// like titles and selections remain visually distinct even without explicit
/// colors.
#[allow(dead_code)]
pub struct ThemeBridge<'a> {
    palette: Option<&'a Palette>,
}

#[allow(dead_code)]
impl<'a> ThemeBridge<'a> {
    pub fn new(palette: &'a Palette) -> Self {
        Self {
            palette: Some(palette),
        }
    }

    /// Terminal-native mode: no colors imposed, terminal theme shows through.
    pub fn terminal() -> Self {
        Self { palette: None }
    }

    pub fn bg(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().bg(hex_to_rgb(&p.background)),
            None => Style::default(),
        }
    }

    pub fn surface(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().bg(hex_to_rgb(&p.surface)),
            None => Style::default(),
        }
    }

    pub fn text(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.text)),
            None => Style::default(),
        }
    }

    pub fn subtext(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.subtext)),
            None => Style::default(),
        }
    }

    pub fn muted(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.muted)),
            None => Style::default(),
        }
    }

    pub fn accent(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.accent)),
            None => Style::default(),
        }
    }

    pub fn green(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.green)),
            None => Style::default(),
        }
    }

    pub fn red(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.red)),
            None => Style::default(),
        }
    }

    pub fn yellow(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.yellow)),
            None => Style::default(),
        }
    }

    pub fn blue(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.blue)),
            None => Style::default(),
        }
    }

    pub fn purple(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.purple)),
            None => Style::default(),
        }
    }

    pub fn border(&self) -> Style {
        match self.palette {
            Some(p) => Style::default().fg(hex_to_rgb(&p.border)),
            None => Style::default(),
        }
    }

    pub fn selection(&self) -> Style {
        match self.palette {
            Some(p) => Style::default()
                .bg(hex_to_rgb(&p.selection))
                .fg(hex_to_rgb(&p.text))
                .add_modifier(Modifier::BOLD),
            None => Style::default().add_modifier(Modifier::REVERSED),
        }
    }

    pub fn title(&self) -> Style {
        match self.palette {
            Some(p) => Style::default()
                .fg(hex_to_rgb(&p.accent))
                .add_modifier(Modifier::BOLD),
            None => Style::default().add_modifier(Modifier::BOLD),
        }
    }

    pub fn status_bar(&self) -> Style {
        match self.palette {
            Some(p) => Style::default()
                .bg(hex_to_rgb(&p.surface))
                .fg(hex_to_rgb(&p.subtext)),
            None => Style::default(),
        }
    }
}
