use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;

use crate::models::app::App;
use crate::models::app::AppState;
use crate::utils::counter::decrement;
use crate::utils::counter::increment;

#[test]
fn test_increment() {
    let mut app = App::default();
    increment(&mut app.counter, 1);
    assert_eq!(app.counter, 1);
}

#[test]
fn test_decrement() {
    let mut app = App::default();
    decrement(&mut app.counter, 1);
    assert_eq!(app.counter, 0);
}

#[test]
fn test_app_state() {
    let app = App::default();
    assert_eq!(app.state, AppState::Start);
}

#[test]
fn test_overflow() {
    let mut app = App::default();
    increment(&mut app.counter, 1);
    increment(&mut app.counter, 1);
    decrement(&mut app.counter, 1);
    assert_eq!(app.counter, 1);

    decrement(&mut app.counter, 1);
    assert_eq!(app.counter, 0);

    decrement(&mut app.counter, 1);
    assert_eq!(app.counter, 0);
}

#[test]
fn test_handle_key_event() {
    let mut app = App::default();
    app.handle_key_event(Event::Key(KeyCode::Up.into()));
    assert_eq!(app.counter, 1);
}

#[test]
fn test_handle_key_event_down() {
    let mut app = App::default();
    app.handle_key_event(Event::Key(KeyCode::Down.into()));
    assert_eq!(app.counter, 0);
}

#[test]
fn test_handle_key_event_esc() {
    let mut app = App::default();
    app.handle_key_event(Event::Key(KeyCode::Esc.into()));
    assert_eq!(app.state, AppState::Exit);
}
