use super::key::{
    handle_key_event_confirm_mode, handle_key_event_filter_input, handle_key_event_help_mode,
    handle_key_event_normal_mode, handle_key_event_rename_input,
};
use crate::events::key::handle_key_event_go_to_mode;
use crate::models::app::App;
use crate::models::app::AppState;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::io::Result;

/// Route a key press to the handler for the current state and store the
/// resulting state on the app. Extracted from the blocking run loop so the
/// dispatch table can be exercised in tests.
pub fn dispatch_key(app: &mut App, event_key: KeyEvent) -> Result<()> {
    app.state = match app.state {
        AppState::Active => handle_key_events_normal_mode(app, event_key)?,
        AppState::Filter => handle_key_events_filter_mode(app, event_key)?,
        AppState::Rename => handle_key_events_rename_mode(app, event_key)?,
        AppState::GoTo => handle_key_events_go_to_mode(app, event_key)?,
        AppState::Confirm => handle_key_events_confirm_mode(app, event_key)?,
        AppState::Help => handle_key_events_help_mode(app, event_key)?,
        AppState::Quit => AppState::Quit,
    };
    Ok(())
}

pub fn handle_key_events_normal_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        KeyCode::Char('/') => {
            app.filter_input.clear();
            app.filter()?;
            Ok(AppState::Filter)
        }
        KeyCode::Char('g') => Ok(app.begin_goto()),
        KeyCode::Char('?') => Ok(AppState::Help),
        KeyCode::F(2) => Ok(app.begin_rename()),
        // `d` is enter-dir on WASD; use `x` / Delete for destructive delete.
        KeyCode::Char('x') | KeyCode::Delete => Ok(app.begin_delete()),
        KeyCode::Char('y') => {
            app.copy_selected_path();
            Ok(AppState::Active)
        }
        KeyCode::Char('b') => {
            app.bookmark_cwd();
            Ok(AppState::Active)
        }
        KeyCode::Char('u') => {
            app.history_back()?;
            Ok(AppState::Active)
        }
        KeyCode::Char('i') => {
            app.history_forward()?;
            Ok(AppState::Active)
        }
        KeyCode::Char('P') => {
            app.cycle_preview();
            Ok(AppState::Active)
        }
        KeyCode::Char('G') => {
            app.jump_home()?;
            Ok(AppState::Active)
        }
        KeyCode::Char(digit) if digit.is_ascii_digit() && digit != '0' => {
            let slot = digit.to_digit(10).unwrap() as usize;
            app.jump_to_bookmark(slot)?;
            Ok(AppState::Active)
        }
        _ => handle_key_event_normal_mode(app, event_key),
    }
}

pub fn handle_key_events_rename_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc => Ok(AppState::Active),
        _ => handle_key_event_rename_input(app, event_key),
    }
}

pub fn handle_key_events_filter_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc => {
            app.filter_input.clear();
            app.filter()?;
            Ok(AppState::Active)
        }
        _ => handle_key_event_filter_input(app, event_key),
    }
}

pub fn handle_key_events_go_to_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        // Esc cancels; `q` is a literal path character (same contract as Filter).
        KeyCode::Esc => {
            app.goto_input.clear();
            Ok(AppState::Active)
        }
        _ => handle_key_event_go_to_mode(app, event_key),
    }
}

pub fn handle_key_events_confirm_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') => Ok(AppState::Active),
        _ => handle_key_event_confirm_mode(app, event_key),
    }
}

pub fn handle_key_events_help_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Active),
        _ => handle_key_event_help_mode(app, event_key),
    }
}
