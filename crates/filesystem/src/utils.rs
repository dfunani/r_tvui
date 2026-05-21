use core::paths::FileType;
use std::fs::Metadata;
use std::path::Path;

pub fn classify_file_type(path: &Path, meta: &Metadata) -> FileType {
    if path.is_dir() && meta.is_dir() {
        return FileType::Directory;
    }
    if path.is_file() && meta.is_file() {
        return FileType::File;
    }
    if path.is_symlink() && meta.is_symlink() {
        return FileType::Symlink;
    }
    return FileType::Other;
}

pub fn is_hidden_file_type(name: &Path, file_type: &FileType) -> bool {
    match file_type {
        FileType::Directory => {
            return name.file_name().unwrap_or_default().to_str().unwrap().starts_with(".");
        }
        FileType::File => {
            return name.to_str().unwrap_or_default().starts_with(".");
        }
        FileType::Symlink => {
            return name.is_symlink().to_string().starts_with(".");
        }
        FileType::Other => {
            return name.to_str().unwrap_or_default().starts_with(".");
        }
    }
}