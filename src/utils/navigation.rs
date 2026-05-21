use core::paths::FileType;
use filesystem::{
    list_directories, parent, DirectoryListOptions, DirectorySortOrder, FileSystemError,
};

use crate::models::app::{App, SidePane};
use crate::utils::browser::update_side_pane;
use crate::utils::opener;

pub fn refresh_listing(app: &mut App) {
    let opts = DirectoryListOptions {
        show_hidden: app.show_hidden,
        sort: DirectorySortOrder::Name,
        include_parent_link: true,
    };

    match list_directories(app.cwd.clone(), opts) {
        Ok(result) => {
            app.list_partial = result.partial;
            app.entries = result.entries;
            if app.selected >= app.entries.len() {
                app.selected = app.entries.len().saturating_sub(1);
            }
            app.status = build_status(app, &result.error_rows);
            update_side_pane(app);
        }
        Err(FileSystemError::Io(err)) => {
            app.entries.clear();
            app.side_pane = SidePane::Preview {
                title: "Error".to_string(),
                body: format!("Failed to read directory:\n{err}"),
            };
            app.status = format!("Error: {err}");
        }
        Err(err) => {
            app.entries.clear();
            app.side_pane = SidePane::Preview {
                title: "Error".to_string(),
                body: format!("{err:?}"),
            };
            app.status = format!("Error: {err:?}");
        }
    }
}

pub fn move_selection(app: &mut App, delta: isize) {
    if app.entries.is_empty() {
        return;
    }
    let len = app.entries.len() as isize;
    let next = (app.selected as isize + delta).rem_euclid(len);
    app.selected = next as usize;
    update_side_pane(app);
}

pub fn navigate_into_selected(app: &mut App) {
    let Some(entry) = app.selected_entry().cloned() else {
        return;
    };

    if entry.is_parent_link {
        go_parent(app);
        return;
    }

    match entry.kind {
        FileType::Directory => {
            app.cwd = entry.path;
            app.selected = 0;
            refresh_listing(app);
        }
        FileType::Symlink => {
            let resolved = std::fs::canonicalize(&entry.path.0);
            match resolved {
                Ok(path) if path.is_dir() => {
                    app.cwd = filesystem::absolute(&path);
                    app.selected = 0;
                    refresh_listing(app);
                }
                Ok(_) => update_side_pane(app),
                Err(err) => {
                    app.side_pane = SidePane::Preview {
                        title: entry.name,
                        body: format!("Broken symlink:\n{err}"),
                    };
                }
            }
        }
        _ => update_side_pane(app),
    }
}

pub fn activate_selected(app: &mut App) {
    let Some(entry) = app.selected_entry().cloned() else {
        return;
    };

    if entry.is_parent_link {
        go_parent(app);
        return;
    }

    match entry.kind {
        FileType::Directory => {
            app.cwd = entry.path;
            app.selected = 0;
            refresh_listing(app);
        }
        FileType::Symlink => {
            let resolved = std::fs::canonicalize(&entry.path.0);
            match resolved {
                Ok(path) if path.is_dir() => {
                    app.cwd = filesystem::absolute(&path);
                    app.selected = 0;
                    refresh_listing(app);
                }
                Ok(path) => open_file(app, &path, &entry.name),
                Err(err) => {
                    app.side_pane = SidePane::Preview {
                        title: entry.name,
                        body: format!("Broken symlink:\n{err}"),
                    };
                }
            }
        }
        FileType::File | FileType::Other => {
            open_file(app, &entry.path.0, &entry.name);
        }
    }
}

fn open_file(app: &mut App, path: &std::path::Path, name: &str) {
    match opener::open_with_system_default(path) {
        Ok(()) => {
            app.status = format!("Opened · {name}");
            app.side_pane = SidePane::Preview {
                title: name.to_string(),
                body: format!(
                    "Opened with default application\n\n{}",
                    path.display()
                ),
            };
        }
        Err(err) => {
            app.side_pane = SidePane::Preview {
                title: name.to_string(),
                body: format!(
                    "Could not open with default application\n\n{}\n\n{err}",
                    path.display()
                ),
            };
            app.status = format!("Open failed: {err}");
        }
    }
}

pub fn go_parent(app: &mut App) {
    if let Some(parent_path) = parent(&app.cwd) {
        app.cwd = parent_path;
        app.selected = 0;
        refresh_listing(app);
    }
}

pub fn go_home(app: &mut App) {
    if let Some(home) = std::env::var_os("HOME") {
        app.cwd = filesystem::absolute(std::path::Path::new(&home));
        app.selected = 0;
        refresh_listing(app);
    }
}

pub fn toggle_hidden(app: &mut App) {
    app.show_hidden = !app.show_hidden;
    refresh_listing(app);
}

fn build_status(
    app: &App,
    errors: &[filesystem::DirectoryListErrorRow],
) -> String {
    let mut status = format!(
        "{} entries",
        app.entries
            .iter()
            .filter(|e| !e.is_parent_link)
            .count()
    );
    if app.list_partial {
        status.push_str(" (truncated)");
    }
    if !errors.is_empty() {
        status.push_str(&format!(" · {} unreadable", errors.len()));
    }
    status
}
