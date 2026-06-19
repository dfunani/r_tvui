#[cfg(test)]
mod test_events {
    use std::fs;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::config::app::AppConfig;
    use crate::events::keys::{
        handle_key_events_filter_mode, handle_key_events_normal_mode, handle_key_events_rename_mode,
    };
    use crate::models::app::{App, AppState};

    fn app_with_files(files: &[(&str, &str)]) -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        let mut app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        (dir, app)
    }

    fn ch(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    fn code(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // ---- scrolling ---------------------------------------------------------

    #[test]
    fn scroll_down_advances_and_clamps() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b"), ("c.txt", "c")]);
        assert_eq!(app.scroll_state.selected(), Some(0));
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state.selected(), Some(1));
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state.selected(), Some(2));
        // already at the bottom -> stays
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state.selected(), Some(2));
    }

    #[test]
    fn scroll_up_retreats_and_clamps() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b")]);
        app.scroll_state.select(Some(1));
        handle_key_events_normal_mode(&mut app, ch('w')).unwrap();
        assert_eq!(app.scroll_state.selected(), Some(0));
        // already at the top -> stays
        handle_key_events_normal_mode(&mut app, ch('w')).unwrap();
        assert_eq!(app.scroll_state.selected(), Some(0));
    }

    // ---- directory navigation ---------------------------------------------

    fn app_with_subdir() -> (tempfile::TempDir, App) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("child.txt"), "x").unwrap();
        let mut app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        (dir, app)
    }

    #[test]
    fn d_enters_directory_and_a_leaves_it() {
        let (_dir, mut app) = app_with_subdir();
        handle_key_events_normal_mode(&mut app, ch('d')).unwrap();
        assert!(app.current_working_directory.ends_with("sub"));
        handle_key_events_normal_mode(&mut app, ch('a')).unwrap();
        assert!(!app.current_working_directory.ends_with("sub"));
    }

    #[test]
    fn enter_on_directory_navigates_in() {
        let (_dir, mut app) = app_with_subdir();
        handle_key_events_normal_mode(&mut app, code(KeyCode::Enter)).unwrap();
        assert!(app.current_working_directory.ends_with("sub"));
    }

    #[test]
    fn home_key_walks_toward_root() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, ch('h')).unwrap();
        assert!(app.current_working_directory.components().count() <= 1);
    }

    // ---- normal-mode transitions ------------------------------------------

    #[test]
    fn normal_mode_enters_sub_modes() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('/')).unwrap(),
            AppState::Filter
        );
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('g')).unwrap(),
            AppState::GoTo
        );
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('?')).unwrap(),
            AppState::Help
        );
        assert_eq!(
            handle_key_events_normal_mode(&mut app, code(KeyCode::F(2))).unwrap(),
            AppState::Rename
        );
    }

    #[test]
    fn normal_mode_quits_on_q_and_esc() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('q')).unwrap(),
            AppState::Quit
        );
        assert_eq!(
            handle_key_events_normal_mode(&mut app, code(KeyCode::Esc)).unwrap(),
            AppState::Quit
        );
    }

    // ---- filter input mode -------------------------------------------------

    #[test]
    fn filter_mode_types_narrows_and_clears() {
        let (_dir, mut app) = app_with_files(&[("alpha.txt", "a"), ("beta.txt", "b")]);
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('/')).unwrap(),
            AppState::Filter
        );

        for c in "alp".chars() {
            assert_eq!(
                handle_key_events_filter_mode(&mut app, ch(c)).unwrap(),
                AppState::Filter
            );
        }
        assert_eq!(app.filter_input, "alp");
        assert_eq!(app.entries_filtered.len(), 1);
        assert_eq!(app.entries_filtered[0].name, "alpha.txt");

        handle_key_events_filter_mode(&mut app, code(KeyCode::Backspace)).unwrap();
        assert_eq!(app.filter_input, "al");

        // 'q' is a literal character in filter mode, not quit.
        handle_key_events_filter_mode(&mut app, ch('q')).unwrap();
        assert_eq!(app.filter_input, "alq");

        let state = handle_key_events_filter_mode(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(state, AppState::Active);
        assert_eq!(app.filter_input, "");
        assert_eq!(app.entries_filtered.len(), app.artifacts.len());
    }

    // ---- rename input mode -------------------------------------------------

    #[test]
    fn rename_mode_edits_buffer_and_cancels() {
        let (_dir, mut app) = app_with_files(&[("file.txt", "a")]);
        assert_eq!(
            handle_key_events_normal_mode(&mut app, code(KeyCode::F(2))).unwrap(),
            AppState::Rename
        );
        assert_eq!(app.rename_input, "file.txt");

        assert_eq!(
            handle_key_events_rename_mode(&mut app, ch('X')).unwrap(),
            AppState::Rename
        );
        assert_eq!(app.rename_input, "file.txtX");

        handle_key_events_rename_mode(&mut app, code(KeyCode::Backspace)).unwrap();
        assert_eq!(app.rename_input, "file.txt");

        let state = handle_key_events_rename_mode(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(state, AppState::Active);
    }
}
