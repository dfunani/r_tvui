#[cfg(test)]
mod test_core {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;

    use crate::errors::{FileSystemErrors, RTVUIError, raise_filesystem_error};
    use crate::utils::{get_artifact_entries, get_artifact_entries_async, is_hidden};
    use crate::{Artifact, ArtifactOptions, ArtifactSort, ArtifactType, MAX_ENTRIES};

    // ---- helpers -----------------------------------------------------------

    /// A throwaway directory that is cleaned up when the returned guard drops.
    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &Path, name: &str, bytes: usize) {
        fs::write(dir.join(name), vec![b'x'; bytes]).unwrap();
    }

    fn options(show_hidden: bool, sort: ArtifactSort) -> ArtifactOptions {
        ArtifactOptions { show_hidden, sort }
    }

    fn names(artifacts: &[Artifact]) -> Vec<String> {
        artifacts.iter().map(|a| a.name.clone()).collect()
    }

    /// Directory containing one visible and one hidden file.
    fn simple_context() -> tempfile::TempDir {
        let dir = temp_dir();
        fs::write(dir.path().join("test.txt"), "test").unwrap();
        fs::write(dir.path().join(".hidden.txt"), "hidden").unwrap();
        dir
    }

    // ---- is_hidden ---------------------------------------------------------

    #[test]
    fn is_hidden_detects_dotfiles() {
        assert!(is_hidden(".bashrc"));
        assert!(is_hidden(".hidden.txt"));
        assert!(!is_hidden("visible.txt"));
        assert!(!is_hidden("a.b.c"));
        assert!(!is_hidden(""));
    }

    // ---- listing basics ----------------------------------------------------

    #[test]
    fn get_artifact_entries_ok() {
        let dir = simple_context();
        let result = get_artifact_entries(&dir.path().to_path_buf(), &ArtifactOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn get_artifact_entries_hides_dotfiles_by_default() {
        let dir = simple_context();
        let listing =
            get_artifact_entries(&dir.path().to_path_buf(), &ArtifactOptions::default()).unwrap();
        let names = names(&listing.artifacts);
        assert!(names.contains(&"test.txt".to_string()));
        assert!(!names.contains(&".hidden.txt".to_string()));
    }

    #[test]
    fn get_artifact_entries_shows_hidden_when_requested() {
        let dir = simple_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(true, ArtifactSort::Name),
        )
        .unwrap();
        let names = names(&listing.artifacts);
        assert!(names.contains(&"test.txt".to_string()));
        assert!(names.contains(&".hidden.txt".to_string()));
    }

    #[test]
    fn get_artifact_entries_reports_metadata() {
        let dir = simple_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(true, ArtifactSort::Name),
        )
        .unwrap();
        let file = listing
            .artifacts
            .iter()
            .find(|a| a.name == "test.txt")
            .unwrap();
        assert_eq!(file.artifact_type, ArtifactType::File);
        assert_eq!(file.size, 4);
        assert_eq!(file.path, dir.path().join("test.txt"));
    }

    #[test]
    fn get_artifact_entries_errors_on_missing_path() {
        let missing = PathBuf::from("/this/path/should/not/exist/rtvui");
        let result = get_artifact_entries(&missing, &ArtifactOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn get_artifact_entries_empty_directory() {
        let dir = temp_dir();
        let listing =
            get_artifact_entries(&dir.path().to_path_buf(), &ArtifactOptions::default()).unwrap();
        assert!(listing.artifacts.is_empty());
        assert!(!listing.partial);
    }

    // ---- MAX_ENTRIES / partial --------------------------------------------

    #[test]
    fn get_artifact_entries_marks_partial_over_limit() {
        let dir = temp_dir();
        for i in 0..(MAX_ENTRIES + 10) {
            write_file(dir.path(), &format!("f_{i}.txt"), 1);
        }
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(true, ArtifactSort::Name),
        )
        .unwrap();
        assert!(listing.partial);
        assert_eq!(listing.artifacts.len(), MAX_ENTRIES);
    }

    // ---- sorting -----------------------------------------------------------

    /// Build a directory where name-order and size-order diverge:
    /// dirs `alpha`, `beta`; files `a.txt` (large) and `z.txt` (small).
    fn sort_context() -> tempfile::TempDir {
        let dir = temp_dir();
        fs::create_dir(dir.path().join("beta")).unwrap();
        fs::create_dir(dir.path().join("alpha")).unwrap();
        write_file(dir.path(), "a.txt", 100);
        write_file(dir.path(), "z.txt", 1);
        dir
    }

    #[test]
    fn sort_directories_before_files() {
        let dir = sort_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(false, ArtifactSort::Name),
        )
        .unwrap();
        let types: Vec<&ArtifactType> =
            listing.artifacts.iter().map(|a| &a.artifact_type).collect();
        // The two directories must come before the two files.
        assert_eq!(types[0], &ArtifactType::Directory);
        assert_eq!(types[1], &ArtifactType::Directory);
        assert_eq!(types[2], &ArtifactType::File);
        assert_eq!(types[3], &ArtifactType::File);
    }

    #[test]
    fn sort_by_name_is_alphabetical_within_group() {
        let dir = sort_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(false, ArtifactSort::Name),
        )
        .unwrap();
        assert_eq!(
            names(&listing.artifacts),
            vec!["alpha", "beta", "a.txt", "z.txt"]
        );
    }

    #[test]
    fn sort_by_size_orders_files_by_bytes() {
        let dir = sort_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(false, ArtifactSort::Size),
        )
        .unwrap();
        // Files come after dirs; size sort is descending => a.txt (100 bytes) before z.txt (1 byte).
        let files: Vec<String> = listing
            .artifacts
            .iter()
            .filter(|a| a.artifact_type == ArtifactType::File)
            .map(|a| a.name.clone())
            .collect();
        assert_eq!(files, vec!["a.txt", "z.txt"]);
    }

    #[test]
    fn sort_by_modified_keeps_dirs_first_and_lists_all() {
        let dir = sort_context();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(false, ArtifactSort::Modified),
        )
        .unwrap();
        assert_eq!(listing.artifacts.len(), 4);
        assert_eq!(listing.artifacts[0].artifact_type, ArtifactType::Directory);
        assert_eq!(listing.artifacts[1].artifact_type, ArtifactType::Directory);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_type_is_detected() {
        let dir = temp_dir();
        fs::write(dir.path().join("target.txt"), "t").unwrap();
        std::os::unix::fs::symlink(dir.path().join("target.txt"), dir.path().join("link.txt"))
            .unwrap();
        let listing = get_artifact_entries(
            &dir.path().to_path_buf(),
            &options(true, ArtifactSort::Name),
        )
        .unwrap();
        let link = listing
            .artifacts
            .iter()
            .find(|a| a.name == "link.txt")
            .unwrap();
        assert_eq!(link.artifact_type, ArtifactType::Symlink);
    }

    #[test]
    fn directory_type_is_detected() {
        let dir = temp_dir();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let listing =
            get_artifact_entries(&dir.path().to_path_buf(), &ArtifactOptions::default()).unwrap();
        let sub = listing.artifacts.iter().find(|a| a.name == "sub").unwrap();
        assert_eq!(sub.artifact_type, ArtifactType::Directory);
    }

    // ---- format_size -------------------------------------------------------

    fn artifact_of_size(size: u64) -> Artifact {
        Artifact {
            name: "x".to_string(),
            path: PathBuf::from("x"),
            artifact_type: ArtifactType::File,
            size,
            modified: Some(SystemTime::now()),
        }
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(artifact_of_size(0).format_size(), "0 B");
        assert_eq!(artifact_of_size(512).format_size(), "512 B");
        assert_eq!(artifact_of_size(1024).format_size(), "1024 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert!(artifact_of_size(2048).format_size().contains("KB"));
    }

    #[test]
    fn format_size_megabytes() {
        assert!(
            artifact_of_size(5 * 1024 * 1024)
                .format_size()
                .contains("MB")
        );
    }

    #[test]
    fn format_size_gigabytes() {
        assert!(
            artifact_of_size(3 * 1024 * 1024 * 1024)
                .format_size()
                .contains("GB")
        );
    }

    // ---- async wrapper -----------------------------------------------------

    #[tokio::test]
    async fn async_listing_matches_sync() {
        let dir = simple_context();
        let path = dir.path().to_path_buf();
        let sync = get_artifact_entries(&path, &options(true, ArtifactSort::Name)).unwrap();
        let async_result = get_artifact_entries_async(path, options(true, ArtifactSort::Name))
            .await
            .unwrap();
        assert_eq!(names(&sync.artifacts), names(&async_result.artifacts));
    }

    #[tokio::test]
    async fn async_listing_errors_on_missing_path() {
        let result = get_artifact_entries_async(
            PathBuf::from("/no/such/rtvui/path"),
            ArtifactOptions::default(),
        )
        .await;
        assert!(result.is_err());
    }

    // ---- error mapping -----------------------------------------------------

    fn io_error() -> std::io::Error {
        std::io::Error::other("boom")
    }

    #[test]
    fn raise_filesystem_error_maps_variants() {
        let path = PathBuf::from("/tmp/x");
        assert!(matches!(
            raise_filesystem_error("NotFound", path.clone(), io_error()),
            RTVUIError::FileSystemError(FileSystemErrors::NotFound(_))
        ));
        assert!(matches!(
            raise_filesystem_error("PermissionDenied", path.clone(), io_error()),
            RTVUIError::FileSystemError(FileSystemErrors::PermissionDenied(_))
        ));
        assert!(matches!(
            raise_filesystem_error("Io", path.clone(), io_error()),
            RTVUIError::FileSystemError(FileSystemErrors::Io(_))
        ));
        assert!(matches!(
            raise_filesystem_error("InvalidPath", path.clone(), io_error()),
            RTVUIError::FileSystemError(FileSystemErrors::InvalidPath(_))
        ));
        // Unknown kinds fall back to an Io error.
        assert!(matches!(
            raise_filesystem_error("???", path, io_error()),
            RTVUIError::FileSystemError(FileSystemErrors::Io(_))
        ));
    }
}
