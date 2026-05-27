use std::io::Error;
use std::path::PathBuf;

#[derive(Debug)]
pub enum FileSystemError {
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    Io(Error),
    InvalidPath(String),
}
