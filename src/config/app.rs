use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub theme: Themes,
    pub settings: Settings,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub sort: Sort,
    pub enable_trash: bool,
    pub preview: Preview,
    pub show_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    pub bookmarks: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Themes::Forest,
            cache: CacheConfig { bookmarks: vec![] },
            settings: Settings::default(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sort: Sort::Name,
            enable_trash: true,
            preview: Preview::OnMove,
            show_hidden: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Themes {
    #[default]
    Forest,
    Midnight,
    Solar,
    Mono,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub highlight: Color,
    pub accent: Color,
    pub header: Color,
    pub background: Color,
    pub text: Color,
}

impl Themes {
    pub fn palette(&self) -> Palette {
        match self {
            Themes::Forest => Palette {
                highlight: Color::Green,
                accent: Color::LightGreen,
                header: Color::Green,
                background: Color::Rgb(12, 20, 12),
                text: Color::Rgb(200, 220, 200),
            },
            Themes::Midnight => Palette {
                highlight: Color::Cyan,
                accent: Color::LightCyan,
                header: Color::Blue,
                background: Color::Rgb(10, 12, 24),
                text: Color::Rgb(200, 210, 235),
            },
            Themes::Solar => Palette {
                highlight: Color::Yellow,
                accent: Color::LightYellow,
                header: Color::LightRed,
                background: Color::Rgb(28, 24, 10),
                text: Color::Rgb(230, 220, 180),
            },
            Themes::Mono => Palette {
                highlight: Color::White,
                accent: Color::Gray,
                header: Color::White,
                background: Color::Rgb(16, 16, 16),
                text: Color::Rgb(210, 210, 210),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Sort {
    #[default]
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Preview {
    #[default]
    OnMove,
    Always,
    Never,
}
