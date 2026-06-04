use crate::Artifact;
use crate::ArtifactListResult;
use crate::ArtifactOptions;
use crate::ArtifactType;
use crate::MAX_ENTRIES;
use std::cmp::Ordering;
use std::fs::{DirEntry, read_dir};
use std::io::Result;
use std::path::PathBuf;

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

pub async fn get_artifact_entries_async(
    path: &PathBuf,
    options: &ArtifactOptions,
) -> Result<ArtifactListResult> {
    let artifacts = get_artifact_entries(path, options)?;
    Ok(artifacts)
}

pub fn get_artifact_entries(
    path: &PathBuf,
    options: &ArtifactOptions,
) -> Result<ArtifactListResult> {
    let mut artifacts = Vec::new();
    let mut partial = false;

    for entry in read_dir(path)? {
        if artifacts.len() >= MAX_ENTRIES {
            partial = true;
            break;
        }

        let artifact = get_artifact_entry(&entry?)?;
        if !options.show_hidden && is_hidden(&artifact.name) {
            continue;
        }
        artifacts.push(artifact);
    }

    artifacts.sort_by(sort_directory_function);
    Ok(ArtifactListResult { artifacts, partial })
}

fn get_artifact_entry(entry: &DirEntry) -> Result<Artifact> {
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

fn get_artifact_type(entry: &DirEntry) -> Result<ArtifactType> {
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

fn sort_directory_function(a: &Artifact, b: &Artifact) -> Ordering {
    artifact_sort_rank(&a.artifact_type)
        .cmp(&artifact_sort_rank(&b.artifact_type))
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}
