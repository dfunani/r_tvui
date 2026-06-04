pub mod utils;

#[cfg(test)]
mod test;

use std::path::PathBuf;
use std::time::SystemTime;

const KILOBYTE: u64 = 1024;
const MEGABYTE: u64 = KILOBYTE * 1024;
const GIGABYTE: u64 = MEGABYTE * 1024;

#[cfg(not(test))]
const MAX_ENTRIES: usize = 50_000;

#[cfg(test)]
const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactType {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Default)]
pub struct ArtifactOptions {
    pub show_hidden: bool,
}

pub struct ArtifactListResult {
    pub artifacts: Vec<Artifact>,
    pub partial: bool,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl Artifact {
    pub fn format_size(&self) -> String {
        match self.size {
            0..=KILOBYTE => format!("{} B", self.size),
            n if n < MEGABYTE => format!("{:.1} KB", n as f64 / KILOBYTE as f64),
            n if n < GIGABYTE => format!("{:.1} MB", n as f64 / MEGABYTE as f64),
            n => format!("{:.1} GB", n as f64 / GIGABYTE as f64),
        }
    }
}
