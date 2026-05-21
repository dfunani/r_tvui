use rtvui_core::paths::FileType;
use ratatui::crossterm::event::{Event, KeyCode};

use crate::models::app::{App, AppState, SidePane};
use crate::utils::navigation::{move_selection, refresh_listing, toggle_hidden};

#[test]
fn test_app_starts_running() {
    let app = App::default();
    assert_eq!(app.state, AppState::Running);
}

#[test]
fn test_app_lists_current_directory() {
    let app = App::default();
    assert!(!app.entries.is_empty());
    assert!(app.entries[0].is_parent_link || app.cwd.0.exists());
}

#[test]
fn test_parent_selection_hides_side_pane() {
    let app = App::default();
    if app.entries.first().map(|e| e.is_parent_link) == Some(true) {
        assert!(!app.side_pane.is_active());
    }
}

#[test]
fn test_directory_selection_shows_folder_pane() {
    let mut app = App::default();
    if let Some(index) = app
        .entries
        .iter()
        .position(|e| e.kind == FileType::Directory && !e.is_parent_link)
    {
        app.selected = index;
        crate::utils::browser::update_side_pane(&mut app);
        assert!(matches!(app.side_pane, SidePane::Folder { .. }));
    }
}

#[test]
fn test_move_selection_wraps() {
    let mut app = App::default();
    let len = app.entries.len();
    if len < 2 {
        return;
    }
    app.selected = len - 1;
    move_selection(&mut app, 1);
    assert_eq!(app.selected, 0);
}

#[test]
fn test_handle_key_event_quit() {
    let mut app = App::default();
    app.handle_key_event(Event::Key(KeyCode::Char('q').into()));
    assert_eq!(app.state, AppState::Exit);
}

#[test]
fn test_esc_quits_when_no_subquery() {
    let mut app = App::default();
    app.handle_key_event(Event::Key(KeyCode::Esc.into()));
    assert_eq!(app.state, AppState::Exit);
}

#[test]
fn test_esc_clears_filter_before_quit() {
    let mut app = App::default();
    app.filter_query = "foo".to_string();
    crate::utils::navigation::apply_filter_to_app(&mut app);
    app.handle_key_event(Event::Key(KeyCode::Esc.into()));
    assert_eq!(app.state, AppState::Running);
    assert!(app.filter_query.is_empty());
}

#[test]
fn test_toggle_hidden() {
    let mut app = App::default();
    let before = app.show_hidden;
    toggle_hidden(&mut app);
    assert_ne!(app.show_hidden, before);
}

#[test]
fn test_cycle_theme() {
    let mut app = App::default();
    let start = app.theme;
    app.cycle_theme();
    assert_eq!(app.theme, start.next());
}

#[test]
fn test_refresh_listing_keeps_valid_selection() {
    let mut app = App::default();
    app.selected = 0;
    refresh_listing(&mut app);
    assert!(app.selected < app.entries.len().max(1));
}
