use std::fs::File;
use std::io::{Read, Result};

use rtvui_core::{Artifact, ArtifactType};

const MAX_PREVIEW_BYTES: usize = 64 * 1024;

pub fn read_artifact_content(artifact: &Artifact) -> Result<String> {
    let mut file = File::open(&artifact.path).unwrap();
    let mut content = vec![0u8; MAX_PREVIEW_BYTES];
    let bytes_read = file.read(&mut content).unwrap();
    content.truncate(bytes_read);
    Ok(String::from_utf8_lossy(&content).into_owned())
}

pub fn display_artifact_name(artifact: &Artifact) -> String {
    if artifact.artifact_type == ArtifactType::Directory {
        format!("{}/", artifact.name)
    } else {
        artifact.name.to_string()
    }
}
