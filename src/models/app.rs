use core::paths::{AbsolutePath, File};

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

#[derive(Debug)]
pub struct App {
    pub cwd: AbsolutePath,
    pub entries: Vec<File>,
    pub selected: usize,
    pub show_hidden: bool,
    pub preview: String,
    pub status: String,
    pub state: AppState,
    pub list_partial: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            cwd: filesystem::absolute(std::path::Path::new(".")),
            entries: Vec::new(),
            selected: 0,
            show_hidden: false,
            preview: String::from("Loading…"),
            status: String::new(),
            state: AppState::default(),
            list_partial: false,
        };
        crate::utils::navigation::refresh_listing(&mut app);
        app
    }
}

impl App {
    pub fn selected_entry(&self) -> Option<&File> {
        self.entries.get(self.selected)
    }
}
