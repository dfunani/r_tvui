use std::{io::Result, path::PathBuf};

pub fn global_exception_handler() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}

pub fn get_start_path(path: Option<PathBuf>) -> Result<PathBuf> {
    let Some(start_path) = path else {
        return std::env::current_dir();
    };

    if start_path.is_dir() {
        return Ok(start_path.canonicalize().unwrap_or(start_path));
    }
    if start_path.is_file() {
        let Some(parent) = start_path.parent() else {
            return std::env::current_dir();
        };
        return parent.canonicalize();
    }

    std::env::current_dir()
}
