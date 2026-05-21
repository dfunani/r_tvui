use core::paths::AbsolutePath;
use std::env::current_dir;
use std::path::Path;

pub fn absolute(path: &Path) -> AbsolutePath {
    if path.is_absolute() {
        return AbsolutePath(path.to_path_buf());
    }
    let current_dir = current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    AbsolutePath(current_dir.join(path))
}

pub fn parent(path: &AbsolutePath) -> Option<AbsolutePath> {
    path.0.parent().map(|p| absolute(p))
}

pub fn join(parent: &AbsolutePath, name: &str) -> AbsolutePath {
    AbsolutePath(parent.0.join(name))
}

/// Returns true when `path` is under at least one configured root.
pub fn assert_allowed(path: &AbsolutePath, roots: &[AbsolutePath]) -> bool {
    if roots.is_empty() {
        return true;
    }
    roots
        .iter()
        .any(|root| path.0.starts_with(&root.0))
}
