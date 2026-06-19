#[cfg(test)]
mod test_cli {
    use std::fs;

    use crate::cli::utils::get_start_path;

    #[test]
    fn none_falls_back_to_current_dir() {
        let resolved = get_start_path(None).unwrap();
        assert_eq!(resolved, std::env::current_dir().unwrap());
    }

    #[test]
    fn directory_is_canonicalized() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = get_start_path(Some(dir.path().to_path_buf())).unwrap();
        assert_eq!(resolved, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn file_resolves_to_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("inside.txt");
        fs::write(&file, "x").unwrap();
        let resolved = get_start_path(Some(file)).unwrap();
        assert_eq!(resolved, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn nonexistent_path_falls_back_to_current_dir() {
        let resolved = get_start_path(Some("/no/such/rtvui/start".into())).unwrap();
        assert_eq!(resolved, std::env::current_dir().unwrap());
    }
}
