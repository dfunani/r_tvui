#[cfg(test)]
mod test_ui {
    use std::fs;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use rtvui_core::{Artifact, ArtifactType};

    use crate::config::app::{AppConfig, Themes};
    use crate::models::app::{App, AppState};
    use crate::models::previewer::{FolderPreviewer, PreviewPreviewer, Previewer};
    use crate::ui::renders::render;

    fn app_with_files(files: &[(&str, &str)]) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        let mut app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        (dir, app)
    }

    fn draw(app: &mut App) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();

        let buffer = terminal.backend().buffer().clone();
        let area = buffer.area;
        let mut text = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                if let Some(cell) = buffer.cell((x, y)) {
                    text.push_str(cell.symbol());
                }
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn renders_file_list_and_status() {
        let (_dir, mut app) = app_with_files(&[("visible.txt", "hi")]);
        let text = draw(&mut app);
        assert!(text.contains("visible.txt"));
        assert!(text.contains("status"));
        assert!(text.contains("Files"));
    }

    #[test]
    fn renders_every_theme_without_panicking() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        for theme in [
            Themes::Forest,
            Themes::Midnight,
            Themes::Solar,
            Themes::Mono,
        ] {
            app.config.theme = theme;
            let text = draw(&mut app);
            assert!(text.contains("a.txt"));
        }
    }

    #[test]
    fn renders_folder_preview() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.previewer = Previewer::Folder(FolderPreviewer {
            title: "child-folder".to_string(),
            artifacts: vec![Artifact {
                name: "inner.txt".to_string(),
                path: std::path::PathBuf::from("inner.txt"),
                artifact_type: ArtifactType::File,
                size: 1,
                modified: None,
            }],
        });
        let text = draw(&mut app);
        // The folder title renders in the pane border; the table NAME column is
        // squeezed in the narrow preview pane so we assert on the title instead.
        assert!(text.contains("child-folder"));
    }

    #[test]
    fn renders_text_preview() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.previewer = Previewer::Preview(PreviewPreviewer {
            title: "readme".to_string(),
            body: "preview-body-content".to_string(),
        });
        let text = draw(&mut app);
        assert!(text.contains("preview-body-content"));
    }

    #[test]
    fn renders_directory_with_trailing_slash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        let mut app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        let text = draw(&mut app);
        assert!(text.contains("subdir/"));
    }

    #[test]
    fn renders_rename_prompt() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.state = AppState::Rename;
        app.rename_input = "a.txt".to_string();
        let text = draw(&mut app);
        assert!(text.contains("rename"));
    }

    #[test]
    fn renders_goto_prompt() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.state = AppState::GoTo;
        app.goto_input = "/tmp".to_string();
        let text = draw(&mut app);
        assert!(text.contains("go to"));
        assert!(text.contains("/tmp"));
    }

    #[test]
    fn renders_help_overlay() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.state = AppState::Help;
        let text = draw(&mut app);
        assert!(text.contains("Help"));
        assert!(text.contains("Navigation"));
        assert!(text.contains("Options"));
    }

    #[test]
    fn renders_status_message_in_bar() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.status_message = "Renamed to b.txt".to_string();
        let text = draw(&mut app);
        assert!(text.contains("Renamed to b.txt"));
    }
}
