use crate::Artifact;
use crate::ArtifactListResult;
use crate::ArtifactOptions;
use crate::ArtifactSort;
use crate::ArtifactType;
use crate::MAX_ENTRIES;
use crate::errors::FileSystemErrors;
use crate::errors::RTVUIError;
use crate::errors::raise_filesystem_error;
use std::cmp::Ordering;
use std::fs::{DirEntry, read_dir};
use std::io::Result as IoResult;
use std::path::PathBuf;

use tokio::task::spawn_blocking;
pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

pub async fn get_artifact_entries_async(
    path: PathBuf,
    options: ArtifactOptions,
) -> Result<ArtifactListResult, RTVUIError> {
    spawn_blocking(move || get_artifact_entries(&path, &options))
        .await
        .map_err(|_| {
            RTVUIError::FileSystemError(FileSystemErrors::InvalidPath("Invalid path".to_string()))
        })?
}

fn handle_error<T>(result: IoResult<T>, path: PathBuf) -> Result<T, RTVUIError> {
    result.map_err(|e| raise_filesystem_error("Io", path.clone(), e))
}

pub fn get_artifact_entries(
    path: &PathBuf,
    options: &ArtifactOptions,
) -> Result<ArtifactListResult, RTVUIError> {
    let mut artifacts = Vec::new();
    let mut partial = false;
    let entries_result = read_dir(path);
    let entries = handle_error(entries_result, path.clone())?;

    for entry in entries {
        if artifacts.len() >= MAX_ENTRIES {
            partial = true;
            break;
        }

        let entry = handle_error(entry, path.clone())?;
        let io_artifact_result = get_artifact_entry(&entry);
        let artifact = handle_error(io_artifact_result, path.clone())?;
        if !options.show_hidden && is_hidden(&artifact.name) {
            continue;
        }
        artifacts.push(artifact);
    }

    artifacts.sort_by(|a, b| sort_directory_function(a, b, options.sort.clone()));
    Ok(ArtifactListResult { artifacts, partial })
}

fn get_artifact_entry(entry: &DirEntry) -> IoResult<Artifact> {
    let name = entry.file_name().to_string_lossy().to_string();
    let artifact_type = get_artifact_type(entry)?;
    let path = entry.path();
    let size = entry.metadata()?.len();
    let modified = entry.metadata()?.modified().ok();

    Ok(Artifact {
        name,
        artifact_type,
        path,
        size,
        modified,
    })
}

fn get_artifact_type(entry: &DirEntry) -> IoResult<ArtifactType> {
    let entry_type = entry.file_type()?;

    if entry_type.is_dir() {
        return Ok(ArtifactType::Directory);
    }
    if entry_type.is_file() {
        return Ok(ArtifactType::File);
    }
    if entry_type.is_symlink() {
        return Ok(ArtifactType::Symlink);
    }
    Ok(ArtifactType::Other)
}

/// Sort key rank: directories first, then symlinks, files, other; names within each group.
fn artifact_sort_rank(artifact_type: &ArtifactType) -> u8 {
    match artifact_type {
        ArtifactType::Directory => 0,
        ArtifactType::Symlink => 1,
        ArtifactType::File => 2,
        ArtifactType::Other => 3,
    }
}

fn sort_directory_function(a: &Artifact, b: &Artifact, sort: ArtifactSort) -> Ordering {
    artifact_sort_rank(&a.artifact_type)
        .cmp(&artifact_sort_rank(&b.artifact_type))
        .then_with(|| match sort {
            ArtifactSort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            ArtifactSort::Size => a.size.cmp(&b.size).reverse(),
            ArtifactSort::Modified => a.modified.cmp(&b.modified).reverse(),
        })
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}
