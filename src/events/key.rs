use super::utils::{scroll_back, scroll_down, scroll_forward, scroll_home, scroll_up};
use crate::models::app::App;
use crate::models::app::AppState;
use crate::os::open_file;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::io::Result;

pub fn handle_key_event_normal_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Up | KeyCode::Char('w') => scroll_up(app)?,
        KeyCode::Down | KeyCode::Char('s') => scroll_down(app)?,
        KeyCode::Left | KeyCode::Char('a') if app.current_working_directory.pop() => {
            scroll_back(app)?
        }
        KeyCode::Right | KeyCode::Char('d') => scroll_forward(app)?,
        KeyCode::Enter => open_file(
            app.artifacts[app.scroll_state.selected().unwrap()]
                .path
                .clone(),
        )?,
        KeyCode::Char('h') | KeyCode::Home => scroll_home(app)?,
        _ => {}
    }
    Ok(AppState::Active)
}

pub fn handle_key_event_filter_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(AppState::Active);
        }
        _ => {}
    }
    Ok(AppState::Active)
}

pub fn handle_key_event_go_to_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(AppState::Quit);
        }
        _ => {}
    }
    Ok(AppState::Active)
}

pub fn handle_key_event_confirm_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(AppState::Quit);
        }
        _ => {}
    }
    Ok(AppState::Active)
}

pub fn handle_key_event_help_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            return Ok(AppState::Quit);
        }
        _ => {}
    }
    Ok(AppState::Active)
}
