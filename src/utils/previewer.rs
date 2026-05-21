use core::formatters::format_size;
use core::paths::{File, FileType};
use std::collections::HashMap;
use std::fs::File as FsFile;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::models::app::App;

const PREVIEW_MAX_BYTES: usize = 64 * 1024;

pub struct JSONPreviewer {
    data: String,
}

impl JSONPreviewer {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn to_hashmap(&self) -> HashMap<String, serde_json::Value> {
        serde_json::from_str(&self.data).unwrap_or_default()
    }

    pub fn to_json(&self) -> String {
        let data = self.to_hashmap();
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| self.data.clone())
    }
}

pub fn read_json_file(path: &Path) -> std::io::Result<String> {
    let file = FsFile::open(path)?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();
    reader.read_to_string(&mut data)?;
    Ok(data)
}

pub fn refresh_preview(app: &mut App) {
    let Some(entry) = app.selected_entry() else {
        app.preview = "No selection".to_string();
        return;
    };

    if entry.is_parent_link {
        app.preview = format!(
            "Parent directory\n\n{}",
            entry.path.0.display()
        );
        return;
    }

    app.preview = preview_path(&entry.path.0, entry.kind);
}

pub fn preview_path(path: &Path, kind: FileType) -> String {
    match kind {
        FileType::Directory => preview_directory(path),
        FileType::Symlink => preview_symlink(path),
        FileType::File => preview_file(path),
        FileType::Other => format!("Other entry\n\n{}", path.display()),
    }
}

fn preview_directory(path: &Path) -> String {
    match std::fs::read_dir(path) {
        Ok(read_dir) => {
            let mut dirs = 0usize;
            let mut files = 0usize;
            for entry in read_dir.flatten().take(10_000) {
                if entry.path().is_dir() {
                    dirs += 1;
                } else {
                    files += 1;
                }
            }
            format!(
                "Directory\n{}\n\n{dirs} subdirectories\n{files} files",
                path.display()
            )
        }
        Err(err) => format!("Directory\n{}\n\nCannot read: {err}", path.display()),
    }
}

fn preview_symlink(path: &Path) -> String {
    match std::fs::read_link(path) {
        Ok(target) => format!(
            "Symlink\n{}\n\n→ {}",
            path.display(),
            target.display()
        ),
        Err(err) => format!("Symlink\n{}\n\n{err}", path.display()),
    }
}

fn preview_file(path: &Path) -> String {
    let meta = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) => return format!("Cannot read metadata:\n{err}"),
    };

    let size_line = format_size(meta.len());
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "json" {
        return match read_json_file(path) {
            Ok(data) => JSONPreviewer::new(data).to_json(),
            Err(err) => format!("JSON preview failed:\n{err}"),
        };
    }

    if is_probably_text(&ext) {
        return match read_text_prefix(path) {
            Ok(text) => text,
            Err(err) => format!("Text preview failed:\n{err}"),
        };
    }

    format!(
        "Binary or unsupported file\n\n{}\nSize: {size_line}",
        path.display()
    )
}

fn is_probably_text(ext: &str) -> bool {
    matches!(
        ext,
        "txt" | "md"
            | "rs"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "html"
            | "css"
            | "js"
            | "ts"
            | "py"
            | "sh"
            | "zsh"
            | "bash"
            | "log"
            | "cfg"
            | "ini"
            | ""
    )
}

fn read_text_prefix(path: &Path) -> std::io::Result<String> {
    let mut file = FsFile::open(path)?;
    let mut buffer = vec![0u8; PREVIEW_MAX_BYTES];
    let read = file.read(&mut buffer)?;
    buffer.truncate(read);

    let mut text = String::from_utf8_lossy(&buffer).into_owned();
    if read == PREVIEW_MAX_BYTES {
        text.push_str("\n\n… truncated …");
    }
    Ok(text)
}

pub fn entry_label(entry: &File) -> String {
    let icon = match entry.kind {
        FileType::Directory if entry.is_parent_link => "⬆",
        FileType::Directory => "📁",
        FileType::Symlink => "🔗",
        FileType::File => "📄",
        FileType::Other => "•",
    };
    let size = entry
        .size
        .map(format_size)
        .unwrap_or_else(|| "-".to_string());
    format!("{icon} {:<24} {size:>8}", entry.name)
}

pub fn cwd_display(path: &Path) -> String {
    path.display().to_string()
}
