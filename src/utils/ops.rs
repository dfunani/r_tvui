use std::path::Path;

pub fn delete_path(path: &Path, use_trash: bool) -> Result<(), String> {
    if use_trash {
        trash::delete(path).map_err(|e| e.to_string())
    } else if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    }
}

pub fn rename_path(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|e| e.to_string())
}
