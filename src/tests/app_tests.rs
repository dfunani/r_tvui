use ratatui::crossterm::event::{Event, KeyCode};

use crate::models::app::{App, AppState};
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
    app.handle_key_event(Event::Key(KeyCode::Esc.into()));
    assert_eq!(app.state, AppState::Exit);
}

#[test]
fn test_toggle_hidden() {
    let mut app = App::default();
    let before = app.show_hidden;
    toggle_hidden(&mut app);
    assert_ne!(app.show_hidden, before);
}

#[test]
fn test_refresh_listing_keeps_valid_selection() {
    let mut app = App::default();
    app.selected = 0;
    refresh_listing(&mut app);
    assert!(app.selected < app.entries.len().max(1));
}
