//! Theme palettes for the TUI.
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use rtvui_core::paths::{File, FileType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeId {
    #[default]
    Gotyme,
    Midnight,
    Forest,
    Solar,
    Mono,
}

impl ThemeId {
    pub const COUNT: usize = 5;

    pub fn slug(self) -> &'static str {
        match self {
            Self::Gotyme => "gotyme",
            Self::Midnight => "midnight",
            Self::Forest => "forest",
            Self::Solar => "solar",
            Self::Mono => "mono",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "gotyme" => Some(Self::Gotyme),
            "midnight" => Some(Self::Midnight),
            "forest" => Some(Self::Forest),
            "solar" => Some(Self::Solar),
            "mono" => Some(Self::Mono),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Gotyme => "GoTyme",
            Self::Midnight => "Midnight",
            Self::Forest => "Forest",
            Self::Solar => "Solar",
            Self::Mono => "Mono",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Gotyme => Self::Midnight,
            Self::Midnight => Self::Forest,
            Self::Forest => Self::Solar,
            Self::Solar => Self::Mono,
            Self::Mono => Self::Gotyme,
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Gotyme => ThemePalette {
                bg: rgb(11, 16, 32),
                bg_2: rgb(16, 23, 48),
                panel: rgb(20, 27, 54),
                code: rgb(14, 20, 48),
                ink: rgb(238, 241, 255),
                muted: rgb(167, 175, 214),
                code_text: rgb(205, 224, 255),
                accent: rgb(124, 156, 255),
                accent_2: rgb(94, 224, 200),
                ok: rgb(122, 240, 196),
                warn: rgb(255, 184, 107),
                border: rgb(72, 82, 120),
                border_dim: rgb(48, 56, 88),
            },
            Self::Midnight => ThemePalette {
                bg: rgb(13, 17, 23),
                bg_2: rgb(22, 27, 34),
                panel: rgb(33, 38, 45),
                code: rgb(13, 17, 23),
                ink: rgb(230, 237, 243),
                muted: rgb(139, 148, 158),
                code_text: rgb(201, 209, 217),
                accent: rgb(88, 166, 255),
                accent_2: rgb(126, 231, 135),
                ok: rgb(126, 231, 135),
                warn: rgb(247, 129, 102),
                border: rgb(48, 54, 61),
                border_dim: rgb(33, 38, 45),
            },
            Self::Forest => ThemePalette {
                bg: rgb(15, 23, 18),
                bg_2: rgb(20, 32, 26),
                panel: rgb(26, 42, 34),
                code: rgb(12, 20, 16),
                ink: rgb(220, 237, 228),
                muted: rgb(140, 170, 155),
                code_text: rgb(180, 220, 195),
                accent: rgb(115, 198, 160),
                accent_2: rgb(94, 224, 180),
                ok: rgb(130, 220, 170),
                warn: rgb(220, 180, 90),
                border: rgb(55, 85, 68),
                border_dim: rgb(35, 55, 45),
            },
            Self::Solar => ThemePalette {
                bg: rgb(28, 22, 16),
                bg_2: rgb(38, 30, 22),
                panel: rgb(48, 38, 28),
                code: rgb(24, 18, 12),
                ink: rgb(250, 235, 210),
                muted: rgb(180, 155, 120),
                code_text: rgb(255, 220, 170),
                accent: rgb(255, 184, 107),
                accent_2: rgb(255, 210, 120),
                ok: rgb(210, 230, 140),
                warn: rgb(255, 140, 90),
                border: rgb(90, 70, 50),
                border_dim: rgb(60, 45, 32),
            },
            Self::Mono => ThemePalette {
                bg: rgb(18, 18, 18),
                bg_2: rgb(28, 28, 28),
                panel: rgb(36, 36, 36),
                code: rgb(24, 24, 24),
                ink: rgb(230, 230, 230),
                muted: rgb(140, 140, 140),
                code_text: rgb(200, 200, 200),
                accent: rgb(200, 200, 200),
                accent_2: rgb(180, 180, 180),
                ok: rgb(220, 220, 220),
                warn: rgb(160, 160, 160),
                border: rgb(80, 80, 80),
                border_dim: rgb(55, 55, 55),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg: Color,
    pub bg_2: Color,
    pub panel: Color,
    pub code: Color,
    pub ink: Color,
    pub muted: Color,
    pub code_text: Color,
    pub accent: Color,
    pub accent_2: Color,
    pub ok: Color,
    pub warn: Color,
    pub border: Color,
    pub border_dim: Color,
}

impl ThemePalette {
    pub fn base(&self) -> Style {
        Style::default().bg(self.bg).fg(self.ink)
    }

    pub fn panel_block(&self, title: &str) -> Block<'static> {
        Block::default()
            .title(format!(" {title} "))
            .title_style(
                Style::default()
                    .fg(self.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border))
            .style(Style::default().bg(self.panel).fg(self.ink))
    }

    pub fn path_bar_block(&self) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border))
            .style(Style::default().bg(self.bg_2).fg(self.ink))
    }

    pub fn status_block(&self) -> Block<'static> {
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(self.border_dim))
            .style(Style::default().bg(self.bg).fg(self.muted))
    }

    pub fn brand_style(&self) -> Style {
        Style::default()
            .fg(self.accent_2)
            .add_modifier(Modifier::BOLD)
    }

    pub fn kicker_style(&self) -> Style {
        Style::default().fg(self.accent_2)
    }

    pub fn path_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn hint_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.accent)
            .fg(self.bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn preview_body_style(&self) -> Style {
        Style::default().fg(self.code_text).bg(self.code)
    }

    pub fn preview_placeholder_style(&self) -> Style {
        Style::default().fg(self.muted).bg(self.code)
    }

    pub fn status_value_style(&self) -> Style {
        Style::default().fg(self.ok).add_modifier(Modifier::BOLD)
    }

    pub fn theme_badge_style(&self) -> Style {
        Style::default().fg(self.warn)
    }

    pub fn entry_style(&self, entry: &File) -> Style {
        if entry.hidden {
            return Style::default().fg(self.muted);
        }
        let color = match entry.kind {
            FileType::Directory if entry.is_parent_link => self.accent,
            FileType::Directory => self.accent_2,
            FileType::Symlink => self.warn,
            FileType::File => self.ink,
            FileType::Other => self.muted,
        };
        Style::default().fg(color)
    }

    pub fn entry_glyph(entry: &File) -> &'static str {
        match entry.kind {
            FileType::Directory if entry.is_parent_link => "▲",
            FileType::Directory => "◆",
            FileType::Symlink => "⇢",
            FileType::File => "·",
            FileType::Other => "?",
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}
