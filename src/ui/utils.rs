use rtvui_core::{Artifact, ArtifactType};

pub fn display_artifact_name(artifact: &Artifact) -> String {
    if artifact.artifact_type == ArtifactType::Directory {
        format!("{}/", artifact.name)
    } else {
        artifact.name.to_string()
    }
}
