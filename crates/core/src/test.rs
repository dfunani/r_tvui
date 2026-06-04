#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{ArtifactOptions, MAX_ENTRIES, utils::get_artifact_entries};

    use std::fs;

    fn create_test_context() -> PathBuf {
        let path = tempfile::tempdir().unwrap().path().to_path_buf();
        let file_path = path.join("test.txt");
        let hidden_file_path = path.join(".hidden.txt");
        fs::create_dir_all(&path).unwrap();
        fs::write(file_path, "test").unwrap();
        fs::write(hidden_file_path, "hidden").unwrap();
        path
    }
    fn create_test_context_with_hidden_files() -> PathBuf {
        let path = create_test_context();
        for i in 0..(MAX_ENTRIES + 10) {
            let hidden_file_path = path.join(format!(".hidden_{}.txt", i));
            fs::write(hidden_file_path, format!("hidden_{}", i)).unwrap();
        }
        path
    }

    #[test]
    fn test_get_artifact_entries() {
        let path = create_test_context();

        let options = ArtifactOptions::default();
        let result = get_artifact_entries(&path, &options);
        assert!(result.is_ok());
    }
    #[test]
    fn test_get_artifact_entries_format_size() {
        let path = create_test_context();
        let result = get_artifact_entries(&path, &ArtifactOptions { show_hidden: true });
        assert!(result.is_ok());
        let artifact = result.unwrap();
        let matched_artifact = artifact.artifacts.iter().find(|a| a.name == "test.txt");
        assert!(matched_artifact.is_some());
        assert!(matched_artifact.unwrap().format_size().contains(" B"));
    }

    #[test]
    fn test_get_artifact_entries_partial() {
        let path = create_test_context_with_hidden_files();
        let result = get_artifact_entries(&path, &ArtifactOptions { show_hidden: true });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.partial);
        assert!(result.artifacts.len() == MAX_ENTRIES);
    }

    #[test]
    fn test_get_artifact_entries_sort() {
        let path = create_test_context();
        let result = get_artifact_entries(&path, &ArtifactOptions { show_hidden: true });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.artifacts.is_empty());
        let matched_artifact = result.artifacts.iter().find(|a| a.name.starts_with('.'));
        assert!(matched_artifact.is_some());
        assert!(matched_artifact.unwrap().name.starts_with('.'));
    }

    #[test]
    fn test_get_artifact_entries_hidden_files() {
        let path = create_test_context();
        let options = ArtifactOptions { show_hidden: true };
        let result = get_artifact_entries(&path, &options);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.artifacts.is_empty());

        let matched_artifact = result.artifacts.iter().find(|a| a.name == ".hidden.txt");
        assert!(matched_artifact.is_some());
    }
}
