use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct DisplayPath(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(pub PathBuf);

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: AbsolutePath,
    pub kind: FileType,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub hidden: bool,
    pub git_status: Option<GitStatus>,
    /// Parent directory entry (`..`), not a real filesystem child.
    pub is_parent_link: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored,
}
