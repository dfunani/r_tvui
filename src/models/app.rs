use core::paths::{AbsolutePath, File};

use crate::theme::ThemeId;
use crate::utils::config::{self, AppConfig};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AppState {
    Exit,
    Running,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Running
    }
}

/// Right-hand pane: Finder-style folder browser or file preview.
#[derive(Debug, Clone)]
pub enum SidePane {
    /// Full-width file list (e.g. `..` selected or no side content).
    Hidden,
    /// Contents of the highlighted folder (Mac column view).
    Folder {
        path: AbsolutePath,
        entries: Vec<File>,
    },
    /// In-terminal preview for the highlighted file.
    Preview {
        title: String,
        body: String,
    },
}

impl SidePane {
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

impl Default for SidePane {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Debug)]
pub struct App {
    pub cwd: AbsolutePath,
    pub entries: Vec<File>,
    pub selected: usize,
    pub show_hidden: bool,
    pub side_pane: SidePane,
    pub status: String,
    pub state: AppState,
    pub list_partial: bool,
    pub theme: ThemeId,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            cwd: filesystem::absolute(std::path::Path::new(".")),
            entries: Vec::new(),
            selected: 0,
            show_hidden: false,
            side_pane: SidePane::Hidden,
            status: String::new(),
            state: AppState::default(),
            list_partial: false,
            theme: AppConfig::load().theme,
        };
        crate::utils::navigation::refresh_listing(&mut app);
        app
    }
}

impl App {
    pub fn selected_entry(&self) -> Option<&File> {
        self.entries.get(self.selected)
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.status = format!("Theme: {} (saved)", self.theme.name());
        let _ = config::save_theme(self.theme);
    }

    pub fn persist_config(&self) {
        let _ = AppConfig { theme: self.theme }.save();
    }
}
