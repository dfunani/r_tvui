use crate::errors::FileSystemError;
use crate::utils::{classify_file_type, is_hidden_name};
use rtvui_core::paths::{AbsolutePath, File, FileType};
use std::cmp::Ordering;
use std::fs;
use std::time::Instant;

const MAX_ENTRIES: usize = 50_000;

#[derive(Debug, Clone)]
pub struct DirectoryListOptions {
    pub show_hidden: bool,
    pub sort: DirectorySortOrder,
    pub include_parent_link: bool,
}

impl Default for DirectoryListOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort: DirectorySortOrder::Name,
            include_parent_link: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DirectorySortOrder {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone)]
pub struct DirectoryListResult {
    pub path: AbsolutePath,
    pub entries: Vec<File>,
    pub read_at: Instant,
    pub partial: bool,
    pub error_rows: Vec<DirectoryListErrorRow>,
}

#[derive(Debug, Clone)]
pub struct DirectoryListErrorRow {
    pub name: String,
    pub message: String,
}

pub fn list_directories(
    path: AbsolutePath,
    opts: DirectoryListOptions,
) -> Result<DirectoryListResult, FileSystemError> {
    let read_dir = fs::read_dir(&path.0).map_err(FileSystemError::Io)?;
    let mut entries = Vec::new();
    let mut error_rows = Vec::new();
    let mut partial = false;

    for entry in read_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error_rows.push(DirectoryListErrorRow {
                    name: "(read error)".to_string(),
                    message: e.to_string(),
                });
                continue;
            }
        };

        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| entry_path.to_string_lossy().into_owned());

        let meta = match entry.metadata() {
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
        let hidden = is_hidden_name(&name);
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
            is_parent_link: false,
        });

        if entries.len() >= MAX_ENTRIES {
            partial = true;
            break;
        }
    }

    sort_entries(&mut entries, opts.sort);
    if opts.include_parent_link {
        prepend_parent_link(&path, &mut entries);
    }

    Ok(DirectoryListResult {
        path,
        entries,
        read_at: Instant::now(),
        partial,
        error_rows,
    })
}

pub async fn list_directories_async(
    path: AbsolutePath,
    opts: DirectoryListOptions,
) -> Result<DirectoryListResult, FileSystemError> {
    let path_for_task = path.clone();
    tokio::task::spawn_blocking(move || list_directories(path_for_task, opts))
        .await
        .map_err(|e| FileSystemError::InvalidPath(e.to_string()))?
}

fn prepend_parent_link(path: &AbsolutePath, entries: &mut Vec<File>) {
    let Some(parent_path) = path.0.parent() else {
        return;
    };
    entries.insert(
        0,
        File {
            name: "..".to_string(),
            path: AbsolutePath(parent_path.to_path_buf()),
            kind: FileType::Directory,
            size: None,
            modified: None,
            hidden: false,
            git_status: None,
            is_parent_link: true,
        },
    );
}

fn sort_entries(entries: &mut [File], order: DirectorySortOrder) {
    entries.sort_by(|a, b| {
        if a.is_parent_link {
            return Ordering::Less;
        }
        if b.is_parent_link {
            return Ordering::Greater;
        }

        let kind_order = match (&a.kind, &b.kind) {
            (FileType::Directory, FileType::Directory) => Ordering::Equal,
            (FileType::Directory, _) => Ordering::Less,
            (_, FileType::Directory) => Ordering::Greater,
            _ => Ordering::Equal,
        };
        if kind_order != Ordering::Equal {
            return kind_order;
        }

        match order {
            DirectorySortOrder::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            DirectorySortOrder::Size => {
                let a_size = a.size.unwrap_or(0);
                let b_size = b.size.unwrap_or(0);
                b_size.cmp(&a_size).then_with(|| a.name.cmp(&b.name))
            }
            DirectorySortOrder::Modified => b
                .modified
                .cmp(&a.modified)
                .then_with(|| a.name.cmp(&b.name)),
        }
    });
}
