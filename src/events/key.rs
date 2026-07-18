use super::utils::{scroll_back, scroll_down, scroll_forward, scroll_home, scroll_up};
use crate::config::utils::{get_config_path, save_config};
use crate::events::utils::handle_key_event_enter_mode;
use crate::models::app::App;
use crate::models::app::AppState;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use std::io::Result;

pub fn handle_key_event_normal_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    handle_navigation_keys(app, event_key)?; // w, s, a, d
    handle_options_keys(app, event_key)?; // t, ., o, h
    Ok(AppState::Active)
}

pub fn handle_key_event_filter_input(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Char(c) => {
            app.filter_input.push(c);
        }
        KeyCode::Backspace => {
            app.filter_input.pop();
        }
        _ => {}
    };
    app.filter()?;
    Ok(AppState::Filter)
}

pub fn handle_key_event_rename_input(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Char(c) => {
            app.rename_input.push(c);
            Ok(AppState::Rename)
        }
        KeyCode::Backspace => {
            app.rename_input.pop();
            Ok(AppState::Rename)
        }
        KeyCode::Enter => {
            app.commit_rename()?;
            Ok(AppState::Active)
        }
        _ => Ok(AppState::Rename),
    }
}

pub fn handle_key_event_go_to_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Char(c) => {
            app.goto_input.push(c);
            Ok(AppState::GoTo)
        }
        KeyCode::Backspace => {
            app.goto_input.pop();
            Ok(AppState::GoTo)
        }
        KeyCode::Enter => app.commit_goto(),
        _ => Ok(AppState::GoTo),
    }
}

pub fn handle_key_event_confirm_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    // Esc/`q` are handled by the outer dispatcher (cancel → Active).
    let _ = event_key;
    Ok(AppState::Confirm)
}

pub fn handle_key_event_help_mode(_: &mut App, event_key: KeyEvent) -> Result<AppState> {
    // Esc/`q` are handled by the outer dispatcher (cancel → Active).
    let _ = event_key;
    Ok(AppState::Help)
}

fn handle_navigation_keys(app: &mut App, event_key: KeyEvent) -> Result<()> {
    match event_key.code {
        KeyCode::Up | KeyCode::Char('w') => scroll_up(app)?,
        KeyCode::Down | KeyCode::Char('s') => scroll_down(app)?,
        KeyCode::Left | KeyCode::Char('a') if app.current_working_directory.pop() => {
            scroll_back(app)?
        }
        KeyCode::Right | KeyCode::Char('d') => scroll_forward(app)?,
        KeyCode::Enter if app.is_file_artifact() => handle_key_event_enter_mode(app)?,
        KeyCode::Enter if !app.is_file_artifact() => scroll_forward(app)?,
        _ => {}
    }
    Ok(())
}

fn handle_options_keys(app: &mut App, event_key: KeyEvent) -> Result<()> {
    match event_key.code {
        KeyCode::Char('t') => {
            app.update_theme();
            save_config(&app.config, &get_config_path()).unwrap_or_default();
        }
        KeyCode::Char('r') => {
            app.refresh()?;
        }
        KeyCode::Char('.') => {
            app.config.settings.show_hidden = !app.config.settings.show_hidden;
            save_config(&app.config, &get_config_path()).unwrap_or_default();
            app.refresh()?;
        }
        KeyCode::Char('o') => {
            app.cycle_sort();
            save_config(&app.config, &get_config_path()).unwrap_or_default();
            app.refresh()?;
        }
        KeyCode::Char('h') | KeyCode::Home => scroll_home(app)?,
        _ => {}
    }
    Ok(())
}
