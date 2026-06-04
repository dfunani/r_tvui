use super::key::{
    handle_key_event_confirm_mode, handle_key_event_filter_mode, handle_key_event_go_to_mode,
    handle_key_event_help_mode, handle_key_event_normal_mode,
};
use crate::models::app::App;
use crate::models::app::AppState;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::io::Result;

pub fn handle_key_events_normal_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        KeyCode::Char('/') => Ok(AppState::Filter),
        KeyCode::Char('g') => Ok(AppState::GoTo),
        KeyCode::Char('?') => Ok(AppState::Help),
        _ => handle_key_event_normal_mode(app, event_key),
    }
}

pub fn handle_key_events_filter_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        _ => handle_key_event_filter_mode(app, event_key),
    }
}

pub fn handle_key_events_go_to_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        _ => handle_key_event_go_to_mode(app, event_key),
    }
}

pub fn handle_key_events_confirm_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        _ => handle_key_event_confirm_mode(app, event_key),
    }
}

pub fn handle_key_events_help_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        _ => handle_key_event_help_mode(app, event_key),
    }
}
