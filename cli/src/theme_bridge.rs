use ratatui::style::{Modifier, Style};

/// Provides semantic styles for the TUI using terminal-native colors.
pub struct ThemeBridge;

#[allow(dead_code)]
impl ThemeBridge {
    pub fn new() -> Self {
        Self
    }

    pub fn bg(&self) -> Style {
        Style::default()
    }
    pub fn surface(&self) -> Style {
        Style::default()
    }
    pub fn text(&self) -> Style {
        Style::default()
    }
    pub fn subtext(&self) -> Style {
        Style::default()
    }
    pub fn muted(&self) -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }
    pub fn accent(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }
    pub fn green(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Green)
    }
    pub fn red(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Red)
    }
    pub fn yellow(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Yellow)
    }
    pub fn blue(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Blue)
    }
    pub fn purple(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Magenta)
    }
    pub fn cyan(&self) -> Style {
        Style::default().fg(ratatui::style::Color::Cyan)
    }
    pub fn border(&self) -> Style {
        Style::default().add_modifier(Modifier::DIM)
    }
    pub fn selection(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }
    pub fn title(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }
    pub fn status_bar(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }
}
