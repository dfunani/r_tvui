use std::path::PathBuf;
use std::io::Error;

#[derive(Debug)]
pub enum FileSystemError {
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    Io(Error),
    InvalidPath(String),
}

