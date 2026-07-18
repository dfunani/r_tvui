#[cfg(test)]
mod test_events {
    use std::fs;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::config::app::AppConfig;
    use crate::events::key::{
        handle_key_event_confirm_mode, handle_key_event_go_to_mode, handle_key_event_help_mode,
    };
    use crate::events::keys::{
        dispatch_key, handle_key_events_confirm_mode, handle_key_events_filter_mode,
        handle_key_events_go_to_mode, handle_key_events_help_mode, handle_key_events_normal_mode,
        handle_key_events_rename_mode,
    };
    use crate::events::utils::handle_key_event_enter_mode;
    use crate::models::app::{App, AppState};
    use crate::tests::support::with_temp_home;

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
        assert_eq!(app.scroll_state().selected(), Some(0));
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(1));
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(2));
        // already at the bottom -> stays
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(2));
    }

    #[test]
    fn scroll_up_retreats_and_clamps() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b")]);
        app.scroll_state_mut().select(Some(1));
        handle_key_events_normal_mode(&mut app, ch('w')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(0));
        // already at the top -> stays
        handle_key_events_normal_mode(&mut app, ch('w')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(0));
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
        assert!(app.current_working_directory().ends_with("sub"));
        handle_key_events_normal_mode(&mut app, ch('a')).unwrap();
        assert!(!app.current_working_directory().ends_with("sub"));
    }

    #[test]
    fn enter_on_directory_navigates_in() {
        let (_dir, mut app) = app_with_subdir();
        handle_key_events_normal_mode(&mut app, code(KeyCode::Enter)).unwrap();
        assert!(app.current_working_directory().ends_with("sub"));
    }

    #[test]
    fn home_key_walks_toward_root() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, ch('h')).unwrap();
        assert!(app.current_working_directory().components().count() <= 1);
    }

    #[test]
    fn capital_g_jumps_to_home() {
        with_temp_home(|home| {
            let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
            handle_key_events_normal_mode(&mut app, ch('G')).unwrap();
            assert_eq!(
                *app.current_working_directory(),
                home.canonicalize().unwrap()
            );
        });
    }

    #[test]
    fn history_keys_navigate() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let first_path = first.path().canonicalize().unwrap();
        let second_path = second.path().canonicalize().unwrap();
        let mut app = App::new(first_path.clone(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        *app.current_working_directory_mut() = second_path.clone();
        app.record_history();

        handle_key_events_normal_mode(&mut app, ch('u')).unwrap();
        assert_eq!(*app.current_working_directory(), first_path);

        handle_key_events_normal_mode(&mut app, ch('i')).unwrap();
        assert_eq!(*app.current_working_directory(), second_path);
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
        assert_eq!(app.filter_input(), "alp");
        assert_eq!(app.entries_filtered().len(), 1);
        assert_eq!(app.entries_filtered()[0].name, "alpha.txt");

        handle_key_events_filter_mode(&mut app, code(KeyCode::Backspace)).unwrap();
        assert_eq!(app.filter_input(), "al");

        // 'q' is a literal character in filter mode, not quit.
        handle_key_events_filter_mode(&mut app, ch('q')).unwrap();
        assert_eq!(app.filter_input(), "alq");

        let state = handle_key_events_filter_mode(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(state, AppState::Active);
        assert_eq!(app.filter_input(), "");
        assert_eq!(app.entries_filtered().len(), app.pane().artifacts.len());
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

    // ---- dispatch table ----------------------------------------------------

    #[test]
    fn dispatch_key_routes_by_state() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);

        app.state = AppState::Active;
        dispatch_key(&mut app, ch('/')).unwrap();
        assert_eq!(app.state, AppState::Filter);

        dispatch_key(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(app.state, AppState::Active);

        app.state = AppState::Rename;
        dispatch_key(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(app.state, AppState::Active);

        app.state = AppState::GoTo;
        dispatch_key(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(app.state, AppState::Active);

        app.state = AppState::Confirm;
        dispatch_key(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(app.state, AppState::Active);

        app.state = AppState::Help;
        dispatch_key(&mut app, code(KeyCode::Esc)).unwrap();
        assert_eq!(app.state, AppState::Active);

        // `q` cancels Help (not GoTo — GoTo treats `q` as path input).
        app.state = AppState::Help;
        dispatch_key(&mut app, ch('q')).unwrap();
        assert_eq!(app.state, AppState::Active);

        // The Quit arm is a no-op terminal state.
        app.state = AppState::Quit;
        dispatch_key(&mut app, ch('x')).unwrap();
        assert_eq!(app.state, AppState::Quit);
    }

    // ---- go-to / confirm / help dispatchers --------------------------------

    #[test]
    fn mode_dispatchers_exit_or_delegate() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);

        assert_eq!(
            handle_key_events_go_to_mode(&mut app, code(KeyCode::Esc)).unwrap(),
            AppState::Active
        );
        assert_eq!(
            handle_key_events_confirm_mode(&mut app, ch('q')).unwrap(),
            AppState::Active
        );
        assert_eq!(
            handle_key_events_help_mode(&mut app, code(KeyCode::Esc)).unwrap(),
            AppState::Active
        );

        assert_eq!(
            handle_key_events_go_to_mode(&mut app, ch('x')).unwrap(),
            AppState::GoTo
        );
        assert_eq!(app.goto_input, "x");
        assert_eq!(
            handle_key_events_confirm_mode(&mut app, ch('x')).unwrap(),
            AppState::Confirm
        );
        assert_eq!(
            handle_key_events_help_mode(&mut app, ch('x')).unwrap(),
            AppState::Help
        );
    }

    #[test]
    fn inner_mode_handlers_cover_all_arms() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);

        assert_eq!(
            handle_key_event_go_to_mode(&mut app, ch('q')).unwrap(),
            AppState::GoTo
        );
        assert_eq!(app.goto_input, "q");
        assert_eq!(
            handle_key_event_go_to_mode(&mut app, ch('x')).unwrap(),
            AppState::GoTo
        );
        assert_eq!(app.goto_input, "qx");

        assert_eq!(
            handle_key_event_confirm_mode(&mut app, ch('x')).unwrap(),
            AppState::Confirm
        );
        assert_eq!(
            handle_key_event_confirm_mode(&mut app, ch('x')).unwrap(),
            AppState::Confirm
        );

        assert_eq!(
            handle_key_event_help_mode(&mut app, code(KeyCode::Esc)).unwrap(),
            AppState::Help
        );
        assert_eq!(
            handle_key_event_help_mode(&mut app, ch('x')).unwrap(),
            AppState::Help
        );
    }

    #[test]
    fn goto_mode_types_jumps_and_rejects() {
        let target = tempfile::tempdir().unwrap();
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);

        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('g')).unwrap(),
            AppState::GoTo
        );
        assert!(app.goto_input.is_empty());

        for c in target.path().display().to_string().chars() {
            assert_eq!(
                handle_key_events_go_to_mode(&mut app, ch(c)).unwrap(),
                AppState::GoTo
            );
        }
        assert_eq!(
            handle_key_events_go_to_mode(&mut app, code(KeyCode::Enter)).unwrap(),
            AppState::Active
        );
        assert_eq!(
            *app.current_working_directory(),
            target.path().canonicalize().unwrap()
        );

        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('g')).unwrap(),
            AppState::GoTo
        );
        for c in "/no/such/rtvui/path".chars() {
            handle_key_events_go_to_mode(&mut app, ch(c)).unwrap();
        }
        assert_eq!(
            handle_key_events_go_to_mode(&mut app, code(KeyCode::Enter)).unwrap(),
            AppState::GoTo
        );
        assert!(app.status_message.contains("not found"));

        assert_eq!(
            handle_key_events_go_to_mode(&mut app, code(KeyCode::Esc)).unwrap(),
            AppState::Active
        );
        assert!(app.goto_input.is_empty());
    }

    #[test]
    fn confirm_delete_mode_yes_and_cancel() {
        let (dir, mut app) = app_with_files(&[("doomed.txt", "a"), ("keep.txt", "b")]);
        app.config.settings.enable_trash = false;

        let index = app
            .entries_filtered()
            .iter()
            .position(|a| a.name == "doomed.txt")
            .unwrap();
        app.scroll_state_mut().select(Some(index));

        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('x')).unwrap(),
            AppState::Confirm
        );
        assert_eq!(
            handle_key_events_confirm_mode(&mut app, ch('n')).unwrap(),
            AppState::Active
        );
        assert!(dir.path().join("doomed.txt").exists());

        assert_eq!(
            handle_key_events_normal_mode(&mut app, code(KeyCode::Delete)).unwrap(),
            AppState::Confirm
        );
        assert_eq!(
            handle_key_events_confirm_mode(&mut app, ch('y')).unwrap(),
            AppState::Active
        );
        assert!(!dir.path().join("doomed.txt").exists());
        assert!(dir.path().join("keep.txt").exists());
        assert!(app.status_message.contains("Deleted"));
    }

    #[test]
    fn copy_path_key_sets_status() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        assert_eq!(
            handle_key_events_normal_mode(&mut app, ch('y')).unwrap(),
            AppState::Active
        );
        assert!(
            app.status_message.contains("Copied") || app.status_message.starts_with("Copy failed"),
            "unexpected status: {}",
            app.status_message
        );
    }

    #[test]
    fn bookmark_keys_save_and_jump() {
        with_temp_home(|_| {
            let first = tempfile::tempdir().unwrap();
            let second = tempfile::tempdir().unwrap();
            let mut app = App::new(first.path().to_path_buf(), AppConfig::default()).unwrap();
            app.reload().unwrap();

            assert_eq!(
                handle_key_events_normal_mode(&mut app, ch('b')).unwrap(),
                AppState::Active
            );
            assert_eq!(app.config.cache.bookmarks.len(), 1);

            *app.current_working_directory_mut() = second.path().to_path_buf();
            handle_key_events_normal_mode(&mut app, ch('b')).unwrap();

            assert_eq!(
                handle_key_events_normal_mode(&mut app, ch('1')).unwrap(),
                AppState::Active
            );
            assert_eq!(
                *app.current_working_directory(),
                first.path().canonicalize().unwrap()
            );
        });
    }

    // ---- option keys (persist to a sandboxed HOME) -------------------------

    #[test]
    fn option_keys_update_config_and_persist() {
        with_temp_home(|home| {
            let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);

            let theme_before = app.config.theme.clone();
            handle_key_events_normal_mode(&mut app, ch('t')).unwrap();
            assert_ne!(app.config.theme, theme_before);
            assert!(home.join(".r_tvui/.config.toml").exists());

            let sort_before = app.config.settings.sort.clone();
            handle_key_events_normal_mode(&mut app, ch('o')).unwrap();
            assert_ne!(app.config.settings.sort, sort_before);

            let hidden_before = app.config.settings.show_hidden;
            handle_key_events_normal_mode(&mut app, ch('.')).unwrap();
            assert_ne!(app.config.settings.show_hidden, hidden_before);

            // refresh + scroll-home keys take their branches without panicking.
            handle_key_events_normal_mode(&mut app, ch('r')).unwrap();
            handle_key_events_normal_mode(&mut app, ch('h')).unwrap();
        });
    }

    // ---- enter-mode guards (no file is opened) -----------------------------

    #[test]
    fn enter_mode_is_noop_without_a_valid_selection() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.scroll_state_mut().select(None);
        handle_key_event_enter_mode(&mut app).unwrap();

        app.scroll_state_mut().select(Some(99));
        handle_key_event_enter_mode(&mut app).unwrap();
    }

    // ---- rename input edge cases -------------------------------------------

    #[test]
    fn rename_mode_enter_commits() {
        let (dir, mut app) = app_with_files(&[("old.txt", "a")]);
        handle_key_events_normal_mode(&mut app, code(KeyCode::F(2))).unwrap();
        app.rename_input = "new.txt".to_string();
        let state = handle_key_events_rename_mode(&mut app, code(KeyCode::Enter)).unwrap();
        assert_eq!(state, AppState::Active);
        assert!(dir.path().join("new.txt").exists());
    }

    #[test]
    fn rename_mode_ignores_unhandled_keys() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, code(KeyCode::F(2))).unwrap();
        let before = app.rename_input.clone();
        let state = handle_key_events_rename_mode(&mut app, code(KeyCode::Left)).unwrap();
        assert_eq!(state, AppState::Rename);
        assert_eq!(app.rename_input, before);
    }

    // ---- navigation guards -------------------------------------------------

    #[test]
    fn scroll_and_navigate_are_safe_without_selection() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.scroll_state_mut().select(None);
        handle_key_events_normal_mode(&mut app, ch('w')).unwrap();
        handle_key_events_normal_mode(&mut app, ch('s')).unwrap();
        handle_key_events_normal_mode(&mut app, ch('d')).unwrap();
    }

    #[test]
    fn forward_on_file_or_out_of_range_is_noop() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        let cwd = app.current_working_directory().clone();
        // a.txt is a file, so scroll_forward takes the non-directory path.
        handle_key_events_normal_mode(&mut app, ch('d')).unwrap();
        assert_eq!(*app.current_working_directory(), cwd);
        // Out-of-range selection: scroll_forward finds no artifact.
        app.scroll_state_mut().select(Some(99));
        handle_key_events_normal_mode(&mut app, ch('d')).unwrap();
        assert_eq!(*app.current_working_directory(), cwd);
    }

    #[test]
    fn vim_jk_move_selection() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a"), ("b.txt", "b"), ("c.txt", "c")]);
        app.reload().unwrap();
        assert_eq!(app.scroll_state().selected(), Some(0));
        handle_key_events_normal_mode(&mut app, ch('j')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(1));
        handle_key_events_normal_mode(&mut app, ch('k')).unwrap();
        assert_eq!(app.scroll_state().selected(), Some(0));
    }

    #[test]
    fn space_toggles_mark_and_u_clears() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.reload().unwrap();
        handle_key_events_normal_mode(&mut app, code(KeyCode::Char(' '))).unwrap();
        assert_eq!(app.pane().marked.len(), 1);
        handle_key_events_normal_mode(&mut app, ch('U')).unwrap();
        assert!(app.pane().marked.is_empty());
    }

    #[test]
    fn new_tab_and_close_tab() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        assert_eq!(app.tabs.len(), 1);
        handle_key_events_normal_mode(&mut app, ch('N')).unwrap();
        assert_eq!(app.tabs.len(), 2);
        handle_key_events_normal_mode(&mut app, ch('W')).unwrap();
        assert_eq!(app.tabs.len(), 1);
    }

    #[test]
    fn split_toggle_creates_peer_tab() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, ch('\\')).unwrap();
        assert!(app.split_tab.is_some());
        assert!(app.tabs.len() >= 2);
        handle_key_events_normal_mode(&mut app, ch('\\')).unwrap();
        assert!(app.split_tab.is_none());
    }

    #[test]
    fn tab_generations_are_globally_unique() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, ch('N')).unwrap();
        handle_key_events_normal_mode(&mut app, ch('N')).unwrap();
        assert_eq!(app.tabs.len(), 3);

        // Reload every tab; no two panes may share a listing generation, so
        // async results can never route to the wrong tab.
        for index in 0..app.tabs.len() {
            app.active_tab = index;
            app.async_reload().unwrap();
        }
        let mut generations: Vec<u64> = app.tabs.iter().map(|pane| pane.generation).collect();
        generations.sort_unstable();
        generations.dedup();
        assert_eq!(generations.len(), app.tabs.len());
    }

    #[test]
    fn closing_tab_clears_split_when_one_tab_remains() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        handle_key_events_normal_mode(&mut app, ch('\\')).unwrap();
        assert!(app.split_tab.is_some());
        handle_key_events_normal_mode(&mut app, ch('W')).unwrap();
        assert_eq!(app.tabs.len(), 1);
        assert!(app.split_tab.is_none());
    }

    #[test]
    fn editor_key_queues_pending_editor_for_files() {
        let (_dir, mut app) = app_with_files(&[("a.txt", "a")]);
        app.reload().unwrap();
        assert!(app.pending_editor.is_none());
        handle_key_events_normal_mode(&mut app, ch('e')).unwrap();
        assert!(app.pending_editor.is_some());
    }

    #[test]
    fn pane_copy_skips_existing_destination() {
        let src_dir = tempfile::tempdir().unwrap();
        std::fs::write(src_dir.path().join("dup.txt"), "source").unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        std::fs::write(dest_dir.path().join("dup.txt"), "already-here").unwrap();

        let mut app = App::new(src_dir.path().to_path_buf(), AppConfig::default()).unwrap();
        app.reload().unwrap();
        app.new_tab().unwrap();
        // Point the new tab (peer) at the destination, then focus the source
        // tab with the peer as the split partner.
        *app.current_working_directory_mut() = dest_dir.path().to_path_buf();
        app.active_tab = 0;
        app.split_tab = Some(1);
        app.copy_to_other_pane().unwrap();

        assert!(app.status_message.contains("skipped"));
        let contents = std::fs::read_to_string(dest_dir.path().join("dup.txt")).unwrap();
        assert_eq!(contents, "already-here");
    }
}
