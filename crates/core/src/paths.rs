use std::path::{PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct DisplayPath(pub PathBuf);

#[derive(Debug)]
pub struct AbsolutePath(pub PathBuf);

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub path: AbsolutePath,
    pub kind: FileType,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub hidden: bool,
    pub git_status: Option<GitStatus>
}


#[derive(Debug)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored
}