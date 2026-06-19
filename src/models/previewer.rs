use rtvui_core::Artifact;
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

const MAX_PREVIEW_BYTES: usize = 64 * 1024;

pub enum Previewer {
    Empty,
    Folder(FolderPreviewer),
    Preview(PreviewPreviewer),
}

pub struct FolderPreviewer {
    pub title: String,
    pub artifacts: Vec<Artifact>,
}

pub struct PreviewPreviewer {
    pub title: String,
    pub body: String,
}

pub fn read_text_prefix(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; MAX_PREVIEW_BYTES];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}
