#[cfg(test)]
mod test_app {
    use std::fs;
    use std::path::PathBuf;

    use rtvui_core::{Artifact, ArtifactListResult, ArtifactSort, ArtifactType};

    use crate::config::app::{AppConfig, Preview, Sort, Themes};
    use crate::models::app::{App, AppState};
    use crate::models::client::AsyncEvents;
    use crate::models::previewer::{FolderPreviewer, PreviewPreviewer, Previewer};

    // ---- helpers -----------------------------------------------------------

    /// Build an app rooted in a fresh temp dir seeded with `files`.
    /// The returned `TempDir` guard keeps the directory alive for the test.
    fn app_with_files(files: &[(&str, &str)]) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        let app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        (dir, app)
    }

    fn artifact(name: &str) -> Artifact {
        Artifact {
            name: name.to_string(),
            path: PathBuf::from("/tmp").join(name),
            artifact_type: ArtifactType::File,
            size: 1,
            modified: None,
        }
    }

    // ---- construction ------------------------------------------------------

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
        assert_eq!(app.entries_filtered, app.artifacts);
        assert_eq!(app.config, AppConfig::default());
    }

    #[test]
    fn new_empty_directory_has_no_selection() {
        let dir = tempfile::tempdir().unwrap();
        let app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        assert!(app.artifacts.is_empty());
        assert_eq!(app.scroll_state.selected(), None);
    }

    // ---- options mapping ---------------------------------------------------

    #[test]
    fn get_artifact_options_maps_config() {
        let mut config = AppConfig::default();
        config.settings.sort = Sort::Size;
        config.settings.show_hidden = true;
        let options = App::get_artifact_options(&config);
        assert!(options.show_hidden);
        assert_eq!(options.sort, ArtifactSort::Size);

        config.settings.sort = Sort::Modified;
        config.settings.show_hidden = false;
        let options = App::get_artifact_options(&config);
        assert!(!options.show_hidden);
        assert_eq!(options.sort, ArtifactSort::Modified);
    }

    // ---- reload / selection clamping --------------------------------------

    #[test]
    fn reload_lists_visible_files() {
        let (dir, mut app) = app_with_files(&[("test.txt", "test"), (".hidden.txt", "hidden")]);
        app.reload().unwrap();
        assert_eq!(app.artifacts.len(), 1);
        assert_eq!(app.artifacts[0].name, "test.txt");
        assert_eq!(app.artifacts[0].path, dir.path().join("test.txt"));
        assert_eq!(app.artifacts[0].artifact_type, ArtifactType::File);
        assert_eq!(app.artifacts[0].size, 4);
    }

    #[test]
    fn reload_clamps_stale_selection() {
        let (dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b"), ("c.txt", "c")]);
        app.reload().unwrap();
        app.scroll_state.select(Some(2));
        // Remove two files so the directory shrinks to a single entry.
        fs::remove_file(dir.path().join("b.txt")).unwrap();
        fs::remove_file(dir.path().join("c.txt")).unwrap();
        app.reload().unwrap();
        assert_eq!(app.artifacts.len(), 1);
        assert_eq!(app.scroll_state.selected(), Some(0));
    }

    #[test]
    fn reload_empty_directory_clears_selection() {
        let (dir, mut app) = app_with_files(&[("only.txt", "x")]);
        app.reload().unwrap();
        assert_eq!(app.scroll_state.selected(), Some(0));
        fs::remove_file(dir.path().join("only.txt")).unwrap();
        app.reload().unwrap();
        assert!(app.artifacts.is_empty());
        assert_eq!(app.scroll_state.selected(), None);
    }

    // ---- filtering ---------------------------------------------------------

    #[test]
    fn filter_matches_substring() {
        let (_dir, mut app) = app_with_files(&[("alpha.txt", "a"), ("beta.rs", "b")]);
        app.reload().unwrap();
        app.filter_input = "alpha".to_string();
        app.filter().unwrap();
        assert_eq!(app.entries_filtered.len(), 1);
        assert_eq!(app.entries_filtered[0].name, "alpha.txt");
    }

    #[test]
    fn filter_is_case_insensitive() {
        let (_dir, mut app) = app_with_files(&[("README.md", "a")]);
        app.reload().unwrap();
        app.filter_input = "readme".to_string();
        app.filter().unwrap();
        assert_eq!(app.entries_filtered.len(), 1);
    }

    #[test]
    fn filter_empty_matches_everything() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b")]);
        app.reload().unwrap();
        app.filter_input = String::new();
        app.filter().unwrap();
        assert_eq!(app.entries_filtered.len(), app.artifacts.len());
    }

    #[test]
    fn filter_no_match_is_empty_and_resets_selection() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.reload().unwrap();
        app.filter_input = "zzz-nope".to_string();
        app.filter().unwrap();
        assert!(app.entries_filtered.is_empty());
        assert_eq!(app.scroll_state.selected(), Some(0));
    }

    // ---- selection helpers -------------------------------------------------

    #[test]
    fn selected_artifact_tracks_scroll_state() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.reload().unwrap();
        assert_eq!(
            app.selected_artifact().map(|a| a.name.clone()),
            Some("file.txt".to_string())
        );
        assert!(app.is_file_artifact());
        app.scroll_state.select(None);
        assert!(app.selected_artifact().is_none());
        assert!(!app.is_file_artifact());
    }

    // ---- sort / theme cycling ---------------------------------------------

    #[test]
    fn cycle_sort_rotates() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        assert_eq!(app.config.settings.sort, Sort::Name);
        app.cycle_sort();
        assert_eq!(app.config.settings.sort, Sort::Size);
        app.cycle_sort();
        assert_eq!(app.config.settings.sort, Sort::Modified);
        app.cycle_sort();
        assert_eq!(app.config.settings.sort, Sort::Name);
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

    // ---- refresh clears cache ---------------------------------------------

    #[test]
    fn refresh_clears_cache() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.cache
            .insert(PathBuf::from("/somewhere"), ArtifactListResult::default());
        assert!(!app.cache.is_empty());
        app.refresh().unwrap();
        assert!(app.cache.is_empty());
    }

    // ---- async event application ------------------------------------------

    #[test]
    fn apply_browser_done_updates_state_when_generation_matches() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        let target = PathBuf::from("/tmp/rtvui-fake");
        let listing = ArtifactListResult {
            artifacts: vec![artifact("only.txt")],
            partial: false,
        };
        app.apply_async_event(AsyncEvents::BrowserDone {
            generation: app.generation,
            artifacts: listing,
            path: target.clone(),
        });
        assert_eq!(app.current_working_directory, target);
        assert_eq!(app.artifacts.len(), 1);
        assert_eq!(app.artifacts[0].name, "only.txt");
        assert!(app.cache.contains_key(&target));
    }

    #[test]
    fn apply_browser_done_ignores_stale_generation() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        let before = app.artifacts.clone();
        let stale = app.generation + 99;
        app.apply_async_event(AsyncEvents::BrowserDone {
            generation: stale,
            artifacts: ArtifactListResult {
                artifacts: vec![artifact("ghost.txt")],
                partial: false,
            },
            path: PathBuf::from("/tmp/ghost"),
        });
        assert_eq!(app.artifacts, before);
    }

    #[test]
    fn apply_previewer_done_respects_generation() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        let preview = Previewer::Preview(PreviewPreviewer {
            title: "t".to_string(),
            body: "hello".to_string(),
        });
        app.apply_async_event(AsyncEvents::PreviewerDone {
            generation: app.previewer_generation,
            previewer: preview,
        });
        match &app.previewer {
            Previewer::Preview(p) => assert_eq!(p.body, "hello"),
            _ => panic!("expected preview to be applied"),
        }

        // A stale preview event is dropped.
        app.apply_async_event(AsyncEvents::PreviewerDone {
            generation: app.previewer_generation + 50,
            previewer: Previewer::Empty,
        });
        assert!(matches!(app.previewer, Previewer::Preview(_)));
    }

    // ---- rename ------------------------------------------------------------

    #[test]
    fn begin_rename_seeds_buffer() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.reload().unwrap();
        assert_eq!(app.begin_rename(), AppState::Rename);
        assert_eq!(app.rename_input, "file.txt");
    }

    #[test]
    fn begin_rename_without_selection_stays_active() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.reload().unwrap();
        app.scroll_state.select(None);
        assert_eq!(app.begin_rename(), AppState::Active);
    }

    #[test]
    fn begin_goto_clears_buffer() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.goto_input = "stale".to_string();
        assert_eq!(app.begin_goto(), AppState::GoTo);
        assert!(app.goto_input.is_empty());
    }

    #[test]
    fn commit_goto_jumps_to_directory() {
        let target = tempfile::tempdir().unwrap();
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.goto_input = target.path().display().to_string();
        assert_eq!(app.commit_goto().unwrap(), AppState::Active);
        assert_eq!(
            app.current_working_directory,
            target.path().canonicalize().unwrap()
        );
        assert!(app.goto_input.is_empty());
    }

    #[test]
    fn commit_goto_rejects_missing_and_file_paths() {
        let (dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.goto_input = "/no/such/rtvui/dir".to_string();
        assert_eq!(app.commit_goto().unwrap(), AppState::GoTo);
        assert!(app.status_message.contains("not found"));

        app.goto_input = dir.path().join("file.txt").display().to_string();
        assert_eq!(app.commit_goto().unwrap(), AppState::GoTo);
        assert!(app.status_message.contains("not a directory"));
    }

    #[test]
    fn commit_rename_renames_file() {
        let (dir, mut app) = app_with_files(&[("old.txt", "a")]);
        app.reload().unwrap();
        app.rename_input = "new.txt".to_string();
        app.commit_rename().unwrap();
        assert!(dir.path().join("new.txt").exists());
        assert!(!dir.path().join("old.txt").exists());
    }

    #[test]
    fn commit_rename_rejects_path_separator() {
        let (dir, mut app) = app_with_files(&[("old.txt", "a")]);
        app.reload().unwrap();
        app.rename_input = "nested/new.txt".to_string();
        app.commit_rename().unwrap();
        assert!(dir.path().join("old.txt").exists());
        assert!(app.status_message.contains("separator"));
    }

    #[test]
    fn commit_rename_rejects_existing_target() {
        let (dir, mut app) = app_with_files(&[("old.txt", "a"), ("taken.txt", "b")]);
        app.reload().unwrap();
        // Select old.txt explicitly (entries are sorted: old.txt, taken.txt).
        let index = app
            .entries_filtered
            .iter()
            .position(|a| a.name == "old.txt")
            .unwrap();
        app.scroll_state.select(Some(index));
        app.rename_input = "taken.txt".to_string();
        app.commit_rename().unwrap();
        assert!(dir.path().join("old.txt").exists());
        assert!(app.status_message.contains("already exists"));
    }

    #[test]
    fn commit_rename_noop_when_unchanged() {
        let (dir, mut app) = app_with_files(&[("same.txt", "a")]);
        app.reload().unwrap();
        app.rename_input = "same.txt".to_string();
        app.commit_rename().unwrap();
        assert!(dir.path().join("same.txt").exists());
    }

    #[test]
    fn preview_never_disables_side_pane() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.reload().unwrap();
        app.config.settings.preview = Preview::Never;
        app.request_previewer();
        assert!(matches!(app.previewer, Previewer::Empty));
    }

    #[test]
    fn request_previewer_handles_files_and_folders() {
        // File selection takes the text-preview branch.
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        app.reload().unwrap();
        app.request_previewer();

        // Directory selection takes the folder-preview branch.
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let mut dir_app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        dir_app.reload().unwrap();
        assert!(!dir_app.is_file_artifact());
        dir_app.request_previewer();
    }

    #[test]
    fn async_reload_uses_cache_when_present() {
        let (_dir, mut app) = app_with_files(&[("real.txt", "a")]);
        app.reload().unwrap();
        let cwd = app.current_working_directory.clone();
        app.cache.insert(
            cwd,
            ArtifactListResult { artifacts: vec![artifact("cached.txt")], partial: false },
        );
        app.async_reload().unwrap();
        assert_eq!(app.artifacts.len(), 1);
        assert_eq!(app.artifacts[0].name, "cached.txt");
    }

    #[test]
    fn reload_error_clears_state_and_reports() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.reload().unwrap();
        app.current_working_directory = PathBuf::from("/no/such/rtvui/dir");
        app.reload().unwrap();
        assert!(app.artifacts.is_empty());
        assert!(app.entries_filtered.is_empty());
        assert!(app.status_message.starts_with("Error:"));
    }

    #[test]
    fn commit_rename_reports_fs_error() {
        let (dir, mut app) = app_with_files(&[("old.txt", "a")]);
        app.reload().unwrap();
        // Delete the file on disk so the rename syscall fails even though the
        // in-memory selection still points at it.
        fs::remove_file(dir.path().join("old.txt")).unwrap();
        app.rename_input = "new.txt".to_string();
        app.commit_rename().unwrap();
        assert!(app.status_message.contains("Rename failed"));
    }

    #[test]
    fn apply_folder_preview_respects_generation() {
        let mut app = App::new(PathBuf::from("."), AppConfig::default()).unwrap();
        app.apply_async_event(AsyncEvents::FolderPreviewDone {
            generation: app.previewer_generation,
            previewer: Previewer::Folder(FolderPreviewer {
                title: "folder".to_string(),
                artifacts: vec![artifact("inner.txt")],
            }),
        });
        assert!(matches!(app.previewer, Previewer::Folder(_)));

        app.apply_async_event(AsyncEvents::FolderPreviewDone {
            generation: app.previewer_generation + 25,
            previewer: Previewer::Empty,
        });
        assert!(matches!(app.previewer, Previewer::Folder(_)));
    }
}
