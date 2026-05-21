use std::path::{Path, PathBuf};

use rtvui_core::paths::AbsolutePath;

/// Resolve CLI start path: directory → open there; file → open parent and select it;
/// missing/invalid → current directory.
pub fn resolve_start_path(
    start_path: Option<PathBuf>,
) -> (AbsolutePath, Option<String>, Option<String>) {
    let Some(raw) = start_path else {
        return (
            filesystem::absolute(Path::new(".")),
            None,
            None,
        );
    };

    let expanded = shellexpand::tilde(&raw.to_string_lossy()).into_owned();
    let path = PathBuf::from(expanded);

    if path.is_dir() {
        return (filesystem::absolute(&path), None, None);
    }

    if path.is_file() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let parent = path.parent().unwrap_or(Path::new("."));
        return (
            filesystem::absolute(parent),
            Some(name),
            None,
        );
    }

    (
        filesystem::absolute(Path::new(".")),
        None,
        Some(format!(
            "Path not found · using {}",
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string())
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn missing_path_uses_current_dir() {
        let (cwd, select, warn) = resolve_start_path(None);
        assert!(select.is_none());
        assert!(warn.is_none());
        assert!(cwd.0.is_dir());
    }

    #[test]
    fn invalid_path_falls_back_with_warning() {
        let (cwd, select, warn) = resolve_start_path(Some(PathBuf::from(
            "/nonexistent-rtvui-path-xyz-99999",
        )));
        assert!(select.is_none());
        assert!(warn.is_some());
        assert!(cwd.0.is_dir());
    }

    #[test]
    fn tilde_expands_to_home() {
        let home = env::var_os("HOME").expect("HOME");
        let (cwd, _, _) = resolve_start_path(Some(PathBuf::from("~")));
        assert_eq!(cwd.0, PathBuf::from(home));
    }
}
