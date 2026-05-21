use rtvui_core::paths::{AbsolutePath, File, FileType};
use filesystem::{list_directories, DirectoryListOptions, FileSystemError};

use crate::models::app::{App, SidePane};
use crate::utils::previewer::{self, preview_path};

pub fn update_side_pane(app: &mut App) {
    let Some(entry) = app.selected_entry().cloned() else {
        app.side_pane = SidePane::Hidden;
        return;
    };

    if entry.is_parent_link {
        app.side_pane = SidePane::Hidden;
        return;
    }

    match resolve_entry(&entry) {
        ResolvedEntry::Directory(path) => load_folder_pane(app, path),
        ResolvedEntry::File(path) => load_file_pane(app, &entry.name, &path),
        ResolvedEntry::Broken(err) => {
            app.side_pane = SidePane::Preview {
                title: entry.name,
                body: format!("Cannot resolve entry:\n{err}"),
            };
        }
    }
}

enum ResolvedEntry {
    Directory(AbsolutePath),
    File(std::path::PathBuf),
    Broken(String),
}

fn resolve_entry(entry: &File) -> ResolvedEntry {
    if entry.kind == FileType::Directory {
        return ResolvedEntry::Directory(entry.path.clone());
    }

    if entry.kind == FileType::Symlink {
        return match std::fs::canonicalize(&entry.path.0) {
            Ok(path) if path.is_dir() => ResolvedEntry::Directory(filesystem::absolute(&path)),
            Ok(path) => ResolvedEntry::File(path),
            Err(err) => ResolvedEntry::Broken(err.to_string()),
        };
    }

    ResolvedEntry::File(entry.path.0.clone())
}

fn load_folder_pane(app: &mut App, path: AbsolutePath) {
    let opts = DirectoryListOptions {
        show_hidden: app.show_hidden,
        sort: app.sort,
        include_parent_link: false,
    };

    match list_directories(path.clone(), opts) {
        Ok(result) => {
            let title = path
                .0
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.0.display().to_string());
            app.side_pane = SidePane::Folder {
                path,
                entries: result.entries,
            };
            if result.partial {
                app.status = format!("{title} · truncated");
            }
        }
        Err(FileSystemError::Io(err)) => {
            app.side_pane = SidePane::Preview {
                title: path.0.display().to_string(),
                body: format!("Cannot list folder:\n{err}"),
            };
        }
        Err(err) => {
            app.side_pane = SidePane::Preview {
                title: "Folder".to_string(),
                body: format!("{err:?}"),
            };
        }
    }
}

fn load_file_pane(app: &mut App, name: &str, path: &std::path::Path) {
    let body = if previewer::is_terminal_previewable(path) {
        preview_path(path, FileType::File)
    } else {
        file_open_hint(path)
    };
    app.side_pane = SidePane::Preview {
        title: name.to_string(),
        body,
    };
}

fn file_open_hint(path: &std::path::Path) -> String {
    let meta = std::fs::metadata(path).ok();
    let size = meta
        .as_ref()
        .map(|m| rtvui_core::formatters::format_size(m.len()))
        .unwrap_or_else(|| "—".to_string());
    format!(
        "{}\n\nSize: {size}\n\nNot previewable in the terminal.\nPress Enter to open with the default app (if one is set).",
        path.display()
    )
}
