use rtvui_core::formatters::format_size;
use rtvui_core::paths::{File, FileType};
use std::collections::HashMap;
use std::fs::File as FsFile;
use std::io::Read;
use std::path::Path;

const PREVIEW_MAX_BYTES: usize = 64 * 1024;
const SNIFF_BYTES: usize = 8192;

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
    read_text_prefix(path)
}

pub fn is_terminal_previewable(path: &Path) -> bool {
    if path.is_dir() {
        return true;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if is_text_extension(&ext) {
        return true;
    }

    looks_like_text(path)
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
        Ok(target) => format!("Symlink\n{}\n\n→ {}", path.display(), target.display()),
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

    if is_text_extension(&ext) || looks_like_text(path) {
        return match read_text_prefix(path) {
            Ok(text) => text,
            Err(err) => format!("Text preview failed:\n{err}"),
        };
    }

    format!(
        "Not previewable in terminal\n\n{}\nSize: {size_line}",
        path.display()
    )
}

fn is_text_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "rs"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "py"
            | "sh"
            | "zsh"
            | "bash"
            | "log"
            | "cfg"
            | "ini"
            | "env"
            | "csv"
            | "sql"
            | "c"
            | "h"
            | "cpp"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "rb"
            | "php"
            | "vue"
            | "svelte"
            | ""
    )
}

fn looks_like_text(path: &Path) -> bool {
    let Ok(mut file) = FsFile::open(path) else {
        return false;
    };
    let mut buffer = vec![0u8; SNIFF_BYTES];
    let Ok(read) = file.read(&mut buffer) else {
        return false;
    };
    if read == 0 {
        return true;
    }
    buffer.truncate(read);
    !buffer.contains(&0) && std::str::from_utf8(&buffer).is_ok()
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
    let size = entry
        .size
        .map(format_size)
        .unwrap_or_else(|| "—".to_string());
    format!("{:<28} {size:>10}", entry.name)
}

pub fn cwd_display(path: &Path) -> String {
    path.display().to_string()
}
