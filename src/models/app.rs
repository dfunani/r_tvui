use rtvui_core::paths::{AbsolutePath, File};
use std::path::PathBuf;

use filesystem::DirectorySortOrder;

use crate::models::mode::{AppMode, InputKind};
use crate::theme::ThemeId;
use crate::utils::config::{self, AppConfig, SortPreference};
use crate::utils::browser::update_side_pane;
use crate::utils::history::NavHistory;
use crate::utils::startup::resolve_start_path;

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

#[derive(Debug)]
pub struct App {
    pub cwd: AbsolutePath,
    pub all_entries: Vec<File>,
    pub entries: Vec<File>,
    pub selected: usize,
    pub show_hidden: bool,
    pub sort: DirectorySortOrder,
    pub sort_pref: SortPreference,
    pub filter_query: String,
    pub side_pane: SidePane,
    pub status: String,
    pub state: AppState,
    pub list_partial: bool,
    pub theme: ThemeId,
    pub mode: AppMode,
    pub input_buffer: String,
    pub history: NavHistory,
    pub bookmarks: Vec<String>,
    pub use_trash: bool,
    pub preview_on_move: bool,
    pub delete_target: Option<AbsolutePath>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(None)
    }
}

impl App {
    pub fn new(start_path: Option<PathBuf>) -> Self {
        let cfg = AppConfig::load();
        let (cwd, select_name, start_warning) = resolve_start_path(start_path);

        let mut app = Self {
            cwd,
            all_entries: Vec::new(),
            entries: Vec::new(),
            selected: 0,
            show_hidden: false,
            sort: cfg.sort.into(),
            sort_pref: cfg.sort,
            filter_query: String::new(),
            side_pane: SidePane::Hidden,
            status: String::new(),
            state: AppState::default(),
            list_partial: false,
            theme: cfg.theme,
            mode: AppMode::Normal,
            input_buffer: String::new(),
            history: NavHistory::default(),
            bookmarks: cfg.bookmarks,
            use_trash: cfg.use_trash,
            preview_on_move: cfg.preview_on_move,
            delete_target: None,
        };
        crate::utils::navigation::refresh_listing(&mut app);

        if let Some(name) = select_name {
            if let Some(index) = app.entries.iter().position(|e| e.name == name) {
                app.selected = index;
                if app.preview_on_move {
                    update_side_pane(&mut app);
                }
            }
        }

        if let Some(msg) = start_warning {
            app.status = msg;
        }

        app
    }

    /// Active in normal mode (e.g. filter) — Esc dismisses these before quitting.
    pub fn has_active_subquery(&self) -> bool {
        !self.filter_query.is_empty()
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        crate::utils::navigation::apply_filter_to_app(self);
        if self.preview_on_move {
            update_side_pane(self);
        }
        self.status = "Filter cleared".to_string();
    }

    pub fn selected_entry(&self) -> Option<&File> {
        self.entries.get(self.selected)
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.status = format!("Theme: {} (saved)", self.theme.name());
        let _ = config::save_theme(self.theme);
    }

    pub fn cycle_sort(&mut self) {
        self.sort_pref = self.sort_pref.next();
        self.sort = self.sort_pref.into();
        self.status = format!("Sort: {}", self.sort_pref.label());
        crate::utils::navigation::refresh_listing(self);
    }

    pub fn toggle_preview_on_move(&mut self) {
        self.preview_on_move = !self.preview_on_move;
        self.status = if self.preview_on_move {
            "Preview on move: on".to_string()
        } else {
            "Preview on move: off (press p)".to_string()
        };
    }

    pub fn enter_input(&mut self, kind: InputKind, seed: String) {
        self.mode = AppMode::Input(kind);
        self.input_buffer = seed;
    }

    pub fn cancel_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.delete_target = None;
    }

    pub fn persist_config(&self) {
        let config = AppConfig {
            theme: self.theme,
            sort: self.sort_pref,
            use_trash: self.use_trash,
            preview_on_move: self.preview_on_move,
            bookmarks: self.bookmarks.clone(),
        };
        let _ = config.save();
    }
}
