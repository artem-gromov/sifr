use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

/// A color in #RRGGBB hex format.
pub type Color = String;

/// Full color palette for a Sifr theme.
/// Mirrors terminal color semantics so terminal themes translate 1:1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    // Base surfaces
    pub background: Color,
    pub surface: Color,
    pub overlay: Color,

    // Text
    pub text: Color,
    pub subtext: Color,
    pub muted: Color,

    // Accent colors
    pub accent: Color, // primary highlight
    pub green: Color,  // success / strength-high
    pub yellow: Color, // warning / strength-medium
    pub red: Color,    // error / strength-low / danger
    pub blue: Color,   // info / links
    pub purple: Color, // TOTP / special
    pub cyan: Color,   // secondary accent

    // UI chrome
    pub border: Color,
    pub selection: Color,
    pub cursor: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub author: Option<String>,
    pub dark: bool,
    pub palette: Palette,
}

impl Theme {
    pub fn from_toml(src: &str) -> Result<Self, ThemeError> {
        toml::from_str(src).map_err(|e| ThemeError::Parse(e.to_string()))
    }
}

/// Built-in theme registry.
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
    active: String,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut r = Self {
            themes: HashMap::new(),
            active: "dracula".into(),
        };
        r.load_bundled();
        r
    }

    fn load_bundled(&mut self) {
        let bundled: &[(&str, &str)] = &[
            ("dracula", include_str!("../../themes/dracula.toml")),
            (
                "solarized-dark",
                include_str!("../../themes/solarized-dark.toml"),
            ),
            (
                "solarized-light",
                include_str!("../../themes/solarized-light.toml"),
            ),
            ("nord", include_str!("../../themes/nord.toml")),
            (
                "catppuccin-mocha",
                include_str!("../../themes/catppuccin-mocha.toml"),
            ),
            (
                "gruvbox-dark",
                include_str!("../../themes/gruvbox-dark.toml"),
            ),
            ("tokyo-night", include_str!("../../themes/tokyo-night.toml")),
        ];
        for (id, src) in bundled {
            if let Ok(t) = Theme::from_toml(src) {
                self.themes.insert(id.to_string(), t);
            }
        }
    }

    pub fn get(&self, id: &str) -> Result<&Theme, ThemeError> {
        self.themes
            .get(id)
            .ok_or_else(|| ThemeError::NotFound(id.into()))
    }

    pub fn active(&self) -> &Theme {
        self.themes.get(&self.active).unwrap()
    }

    pub fn set_active(&mut self, id: &str) -> Result<(), ThemeError> {
        if self.themes.contains_key(id) {
            self.active = id.into();
            Ok(())
        } else {
            Err(ThemeError::NotFound(id.into()))
        }
    }

    pub fn list(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.themes.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Load a user-supplied theme from a TOML file and register it.
    pub fn load_file(&mut self, path: &str) -> Result<(), ThemeError> {
        let src = std::fs::read_to_string(path).map_err(|e| ThemeError::Parse(e.to_string()))?;
        let theme = Theme::from_toml(&src)?;
        self.themes
            .insert(theme.name.to_lowercase().replace(' ', "-"), theme);
        Ok(())
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
