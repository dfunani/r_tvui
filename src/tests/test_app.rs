mod test_app {
    use std::path::PathBuf;
    use std::{fs, time::SystemTime};

    use rtvui_core::ArtifactType;

    use crate::config::app::Themes;
    use crate::{
        config::app::AppConfig,
        models::app::{App, AppState},
    };
    fn create_test_context() -> PathBuf {
        let path = tempfile::tempdir().unwrap().path().to_path_buf();
        let file_path = path.join("test.txt");
        let hidden_file_path = path.join(".hidden.txt");
        fs::create_dir_all(&path).unwrap();
        fs::write(file_path, "test").unwrap();
        fs::write(hidden_file_path, "hidden").unwrap();
        path
    }

    #[test]
    fn test_app_new() {
        let app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        assert_eq!(app.current_working_directory, PathBuf::from("."));
        assert!(!app.artifacts.is_empty());
        assert_eq!(app.scroll_state.selected(), Some(0));
        assert_eq!(app.status_message, "");
        assert_eq!(app.state, AppState::Active);
        assert_eq!(app.filter_input, "");
        assert_eq!(app.entries_cache, app.artifacts);
        assert_eq!(app.entries_filtered, Vec::new());
        assert_eq!(app.config, AppConfig::default());
    }

    #[test]
    fn test_app_reload() {
        let path = create_test_context();
        let mut app = App::new(path.clone(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        assert!(!app.artifacts.is_empty());
        assert_eq!(app.scroll_state.selected(), Some(0));
        assert_eq!(app.artifacts.len(), 1);
        assert_eq!(app.artifacts[0].name, "test.txt");
        assert_eq!(app.artifacts[0].path, path.join("test.txt"));
        assert_eq!(app.artifacts[0].artifact_type, ArtifactType::File);
        assert_eq!(app.artifacts[0].size, 4);
    }

    #[test]
    fn test_app_filter() {
        let path = create_test_context();
        let mut app = App::new(path.clone(), AppConfig::default()).unwrap();
        app.filter_input = "test".to_string();
        app.filter().unwrap();
        assert!(!app.entries_filtered.is_empty());
        assert_eq!(app.entries_filtered.len(), 1);
        let filtered_artifact = app.entries_filtered[0].clone();
        assert_eq!(filtered_artifact.name, "test.txt");
        assert_eq!(filtered_artifact.path, path.join("test.txt"));
        assert_eq!(filtered_artifact.artifact_type, ArtifactType::File);
        assert_eq!(filtered_artifact.size, 4);
    }

    #[test]
    fn test_app_update_theme() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        app.update_theme();
        assert_eq!(app.config.theme, Themes::Midnight);

        app.update_theme();
        assert_eq!(app.config.theme, Themes::Solar);

        app.update_theme();
        assert_eq!(app.config.theme, Themes::Mono);

        app.update_theme();
        assert_eq!(app.config.theme, Themes::Forest);

        app.update_theme();
        assert_eq!(app.config.theme, Themes::Midnight);
    }
}
