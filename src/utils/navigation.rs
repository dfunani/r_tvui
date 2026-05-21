use rtvui_core::paths::{AbsolutePath, FileType};
use filesystem::{
    list_directories, parent, DirectoryListOptions, FileSystemError,
};

use crate::models::app::{App, SidePane};
use crate::utils::browser::update_side_pane;
use crate::utils::filter::apply_name_filter;
use crate::utils::opener;

pub fn list_options(app: &App) -> DirectoryListOptions {
    DirectoryListOptions {
        show_hidden: app.show_hidden,
        sort: app.sort,
        include_parent_link: true,
    }
}

pub fn refresh_listing(app: &mut App) {
    match list_directories(app.cwd.clone(), list_options(app)) {
        Ok(result) => {
            app.list_partial = result.partial;
            app.all_entries = result.entries;
            apply_filter_to_app(app);
            app.status = build_status(app, &result.error_rows);
            maybe_update_side_pane(app);
        }
        Err(FileSystemError::Io(err)) => {
            app.all_entries.clear();
            app.entries.clear();
            app.side_pane = SidePane::Preview {
                title: "Error".to_string(),
                body: format!("Failed to read directory:\n{err}"),
            };
            app.status = format!("Error: {err}");
        }
        Err(err) => {
            app.all_entries.clear();
            app.entries.clear();
            app.side_pane = SidePane::Preview {
                title: "Error".to_string(),
                body: format!("{err:?}"),
            };
            app.status = format!("Error: {err:?}");
        }
    }
}

pub fn apply_filter_to_app(app: &mut App) {
    app.entries = apply_name_filter(&app.all_entries, &app.filter_query);
    if app.selected >= app.entries.len() {
        app.selected = app.entries.len().saturating_sub(1);
    }
}

pub fn navigate_to(app: &mut App, cwd: AbsolutePath, record_history: bool) {
    if record_history {
        app.history.push_visit(app.cwd.clone());
    }
    app.cwd = cwd;
    app.selected = 0;
    refresh_listing(app);
}

pub fn move_selection(app: &mut App, delta: isize) {
    if app.entries.is_empty() {
        return;
    }
    let len = app.entries.len() as isize;
    let next = (app.selected as isize + delta).rem_euclid(len);
    app.selected = next as usize;
    maybe_update_side_pane(app);
}

fn maybe_update_side_pane(app: &mut App) {
    if app.preview_on_move {
        update_side_pane(app);
    }
}

pub fn force_preview(app: &mut App) {
    update_side_pane(app);
    app.status = "Preview updated".to_string();
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
            navigate_to(app, entry.path, true);
        }
        FileType::Symlink => {
            let resolved = std::fs::canonicalize(&entry.path.0);
            match resolved {
                Ok(path) if path.is_dir() => {
                    navigate_to(app, filesystem::absolute(&path), true);
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
            navigate_to(app, entry.path, true);
        }
        FileType::Symlink => {
            let resolved = std::fs::canonicalize(&entry.path.0);
            match resolved {
                Ok(path) if path.is_dir() => {
                    navigate_to(app, filesystem::absolute(&path), true);
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

fn open_file(_app: &mut App, path: &std::path::Path, _name: &str) {
    let _ = opener::try_open_with_system_default(path);
    // No app: silent — keep listing/preview as-is (no status, no stderr in TUI).
}

pub fn go_parent(app: &mut App) {
    if let Some(parent_path) = parent(&app.cwd) {
        navigate_to(app, parent_path, true);
    }
}

pub fn go_home(app: &mut App) {
    if let Some(home) = std::env::var_os("HOME") {
        navigate_to(
            app,
            filesystem::absolute(std::path::Path::new(&home)),
            true,
        );
    }
}

pub fn history_back(app: &mut App) {
    let current = app.cwd.clone();
    if let Some(prev) = app.history.go_back(current) {
        app.cwd = prev;
        app.selected = 0;
        refresh_listing(app);
        app.status = "History back".to_string();
    }
}

pub fn history_forward(app: &mut App) {
    let current = app.cwd.clone();
    if let Some(next) = app.history.go_forward(current) {
        app.cwd = next;
        app.selected = 0;
        refresh_listing(app);
        app.status = "History forward".to_string();
    }
}

pub fn toggle_hidden(app: &mut App) {
    app.show_hidden = !app.show_hidden;
    refresh_listing(app);
}

pub fn confirm_goto_path(app: &mut App, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        app.status = "Go to path: cancelled".to_string();
        return;
    }
    let path = shellexpand::tilde(trimmed).into_owned();
    let abs = filesystem::absolute(std::path::Path::new(&path));
    if abs.0.is_dir() {
        navigate_to(app, abs, true);
        app.status = format!("→ {}", app.cwd.0.display());
    } else {
        app.status = format!("Not a directory: {}", abs.0.display());
    }
}

pub fn confirm_filter(app: &mut App, raw: &str) {
    app.filter_query = raw.trim().to_string();
    apply_filter_to_app(app);
    maybe_update_side_pane(app);
    if app.filter_query.is_empty() {
        app.status = "Filter cleared".to_string();
    } else {
        app.status = format!("Filter: {}", app.filter_query);
    }
}

pub fn confirm_rename(app: &mut App, new_name: &str) {
    let Some(entry) = app.selected_entry().cloned() else {
        return;
    };
    if entry.is_parent_link {
        app.status = "Cannot rename ..".to_string();
        return;
    }
    let new_name = new_name.trim();
    if new_name.is_empty() || new_name.contains(std::path::MAIN_SEPARATOR) {
        app.status = "Invalid name".to_string();
        return;
    }
    let dest = entry.path.0.parent().map(|p| p.join(new_name));
    let Some(dest) = dest else {
        app.status = "Rename failed: no parent".to_string();
        return;
    };
    match crate::utils::ops::rename_path(&entry.path.0, &dest) {
        Ok(()) => {
            app.status = format!("Renamed → {new_name}");
            refresh_listing(app);
        }
        Err(err) => app.status = format!("Rename failed: {err}"),
    }
}

pub fn start_delete_confirm(app: &mut App) {
    let Some(entry) = app.selected_entry().cloned() else {
        return;
    };
    if entry.is_parent_link {
        app.status = "Cannot delete ..".to_string();
        return;
    }
    app.delete_target = Some(entry.path.clone());
    app.mode = crate::models::mode::AppMode::ConfirmDelete;
    app.status = format!("Delete {}? y/n", entry.name);
}

pub fn confirm_delete(app: &mut App) {
    let Some(path) = app.delete_target.take() else {
        return;
    };
    let name = path
        .0
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.0.display().to_string());

    match crate::utils::ops::delete_path(&path.0, app.use_trash) {
        Ok(()) => {
            let via = if app.use_trash { "trash" } else { "permanent" };
            app.status = format!("Deleted ({via}) · {name}");
            app.mode = crate::models::mode::AppMode::Normal;
            refresh_listing(app);
        }
        Err(err) => {
            app.status = format!("Delete failed: {err}");
            app.mode = crate::models::mode::AppMode::Normal;
        }
    }
}

pub fn bookmark_cwd(app: &mut App) {
    let path = app.cwd.0.display().to_string();
    if app.bookmarks.iter().any(|b| b == &path) {
        app.status = "Already bookmarked".to_string();
        return;
    }
    if app.bookmarks.len() >= 9 {
        app.status = "Bookmark slots full (max 9)".to_string();
        return;
    }
    app.bookmarks.push(path.clone());
    app.status = format!("Bookmark {} → {}", app.bookmarks.len(), path);
    app.persist_config();
}

pub fn goto_bookmark(app: &mut App, index: usize) {
    let Some(path) = app.bookmarks.get(index).cloned() else {
        app.status = format!("No bookmark {}", index + 1);
        return;
    };
    let expanded = shellexpand::tilde(&path).into_owned();
    let abs = filesystem::absolute(std::path::Path::new(&expanded));
    if abs.0.is_dir() {
        navigate_to(app, abs, true);
        app.status = format!("Bookmark {} → {}", index + 1, path);
    } else {
        app.status = format!("Bookmark missing: {path}");
    }
}

fn build_status(
    app: &App,
    errors: &[filesystem::DirectoryListErrorRow],
) -> String {
    let mut status = format!(
        "{} entries · sort:{}",
        app.entries
            .iter()
            .filter(|e| !e.is_parent_link)
            .count(),
        app.sort_pref.label()
    );
    if !app.filter_query.is_empty() {
        status.push_str(&format!(" · filter:{}", app.filter_query));
    }
    if app.list_partial {
        status.push_str(" (truncated)");
    }
    if !errors.is_empty() {
        status.push_str(&format!(" · {} unreadable", errors.len()));
    }
    status
}
