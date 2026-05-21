use std::time::Instant;
use crate::{errors::FileSystemError, utils::{classify_file_type, is_hidden_file_type}};
use core::paths::{File, AbsolutePath};
use tokio::fs::read_dir;

#[derive(Debug)]
pub struct DirectoryListOptions {
    pub show_hidden: bool,
    pub sort: DirectorySortOrder,
}

#[derive(Debug)]
pub enum DirectorySortOrder { Name, Size, Modified }

#[derive(Debug)]
pub struct DirectoryListResult {
    pub path: AbsolutePath,
    pub entries: Vec<File>,
    pub read_at: Instant,
    pub partial: bool,           // true if truncated due to cap
    pub error_rows: Vec<DirectoryListErrorRow>, // permission denied on child, etc.
}

#[derive(Debug)]
pub struct DirectoryListErrorRow {
    pub name: String,
    pub message: String,
}

/// Async: spawns on tokio, sends result on oneshot or mpsc
pub async fn list_directories_async(
    path: AbsolutePath,
    opts: DirectoryListOptions,
) -> Result<DirectoryListResult, FileSystemError> {
    let mut read_dir = read_dir(&path.0).await.map_err(FileSystemError::Io)?;
    let mut entries = Vec::new();
    let mut error_rows = Vec::new();

    while let Some(entry) = read_dir.next_entry().await.map_err(FileSystemError::Io)? {
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| entry_path.to_string_lossy().into_owned());

        let meta = match entry.metadata().await {
            Ok(meta) => meta,
            Err(e) => {
                error_rows.push(DirectoryListErrorRow {
                    name: name.clone(),
                    message: e.to_string(),
                });
                continue;
            }
        };

        let file_type = classify_file_type(&entry_path, &meta);
        let hidden = is_hidden_file_type(&entry_path, &file_type);
        if hidden && !opts.show_hidden {
            continue;
        }

        entries.push(File {
            name,
            path: AbsolutePath(entry_path),
            kind: file_type,
            size: Some(meta.len()),
            modified: meta.modified().ok(),
            hidden,
            git_status: None,
        });
    }

    Ok(DirectoryListResult {
        path,
        entries,
        read_at: Instant::now(),
        partial: false,
        error_rows,
    })
}