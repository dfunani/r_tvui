use core::paths::FileType;
use std::fs::Metadata;
use std::path::Path;

pub fn classify_file_type(path: &Path, meta: &Metadata) -> FileType {
    let file_type = meta.file_type();
    if file_type.is_dir() {
        return FileType::Directory;
    }
    if file_type.is_file() {
        return FileType::File;
    }
    if file_type.is_symlink() || path.is_symlink() {
        return FileType::Symlink;
    }
    FileType::Other
}

pub fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') && name != ".."
}
