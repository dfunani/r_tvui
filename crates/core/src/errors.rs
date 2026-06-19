use crossterm::event::KeyEvent;
use std::{io::Error, path::PathBuf};

#[derive(Debug)]
pub enum RTVUIError {
    FileSystemError(FileSystemErrors),
    SystemError(SystemError),
}

#[derive(Debug)]
pub enum FileSystemErrors {
    NotFound(PathBuf),
    PermissionDenied(PathBuf),
    Io(Error),
    InvalidPath(String),
}

#[derive(Debug)]
pub enum SystemError {
    EventKeyError(KeyEvent),
}

pub fn raise_filesystem_error(error_type: &str, path: PathBuf, error: Error) -> RTVUIError {
    match error_type {
        "NotFound" => RTVUIError::FileSystemError(FileSystemErrors::NotFound(path)),
        "PermissionDenied" => RTVUIError::FileSystemError(FileSystemErrors::PermissionDenied(path)),
        "Io" => RTVUIError::FileSystemError(FileSystemErrors::Io(error)),
        "InvalidPath" => RTVUIError::FileSystemError(FileSystemErrors::InvalidPath(
            path.to_string_lossy().to_string(),
        )),
        _ => RTVUIError::FileSystemError(FileSystemErrors::Io(error)),
    }
}
