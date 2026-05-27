use rtvui_core::paths::{File, FileType};

use crate::models::app::{App, SidePane};
use crate::utils::navigation::side_list_options;
use crate::utils::previewer;

pub fn update_side_pane(app: &mut App) {
    let Some(entry) = app.selected_entry().cloned() else {
        app.side_pane = SidePane::Hidden;
        app.side_loading = false;
        return;
    };

    if entry.is_parent_link {
        app.side_pane = SidePane::Hidden;
        app.side_loading = false;
        return;
    }

    match resolve_entry(&entry) {
        ResolvedEntry::Directory(path) => load_folder_pane(app, path),
        ResolvedEntry::File(path) => load_file_pane(app, &entry.name, &path),
        ResolvedEntry::Broken(err) => {
            app.side_loading = false;
            app.side_pane = SidePane::Preview {
                title: entry.name,
                body: format!("Cannot resolve entry:\n{err}"),
            };
        }
    }
}

enum ResolvedEntry {
    Directory(rtvui_core::paths::AbsolutePath),
    File(std::path::PathBuf),
    Broken(String),
}

fn resolve_entry(entry: &File) -> ResolvedEntry {
    if entry.kind == FileType::Directory {
        return ResolvedEntry::Directory(entry.path.clone());
    }

    if entry.kind == FileType::Symlink {
        return match std::fs::canonicalize(&entry.path.0) {
            Ok(path) if path.is_dir() => {
                ResolvedEntry::Directory(filesystem::absolute(&path))
            }
            Ok(path) => ResolvedEntry::File(path),
            Err(err) => ResolvedEntry::Broken(err.to_string()),
        };
    }

    ResolvedEntry::File(entry.path.0.clone())
}

fn load_folder_pane(app: &mut App, path: rtvui_core::paths::AbsolutePath) {
    app.side_pane_gen = app.side_pane_gen.wrapping_add(1);
    let generation = app.side_pane_gen;
    app.side_loading = true;
    app.listing
        .request_side_folder(generation, path, side_list_options(app));
}

fn load_file_pane(app: &mut App, name: &str, path: &std::path::Path) {
    if previewer::is_terminal_previewable(path) {
        app.side_pane_gen = app.side_pane_gen.wrapping_add(1);
        let generation = app.side_pane_gen;
        app.side_loading = true;
        app.listing.request_side_preview(
            generation,
            name.to_string(),
            path.to_path_buf(),
            FileType::File,
        );
        return;
    }

    app.side_loading = false;
    app.side_pane = SidePane::Preview {
        title: name.to_string(),
        body: file_open_hint(path),
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
