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

#[allow(dead_code)]
pub struct ThemeBridge<'a> {
    pub palette: &'a Palette,
}

#[allow(dead_code)]
impl<'a> ThemeBridge<'a> {
    pub fn new(palette: &'a Palette) -> Self {
        Self { palette }
    }

    pub fn bg(&self) -> Style {
        Style::default().bg(hex_to_rgb(&self.palette.background))
    }

    pub fn surface(&self) -> Style {
        Style::default().bg(hex_to_rgb(&self.palette.surface))
    }

    pub fn text(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.text))
    }

    pub fn subtext(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.subtext))
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.muted))
    }

    pub fn accent(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.accent))
    }

    pub fn green(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.green))
    }

    pub fn red(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.red))
    }

    pub fn yellow(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.yellow))
    }

    pub fn blue(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.blue))
    }

    pub fn purple(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.purple))
    }

    pub fn border(&self) -> Style {
        Style::default().fg(hex_to_rgb(&self.palette.border))
    }

    pub fn selection(&self) -> Style {
        Style::default()
            .bg(hex_to_rgb(&self.palette.selection))
            .fg(hex_to_rgb(&self.palette.text))
            .add_modifier(Modifier::BOLD)
    }

    pub fn title(&self) -> Style {
        Style::default()
            .fg(hex_to_rgb(&self.palette.accent))
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_bar(&self) -> Style {
        Style::default()
            .bg(hex_to_rgb(&self.palette.surface))
            .fg(hex_to_rgb(&self.palette.subtext))
    }
}
