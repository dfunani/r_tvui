use crate::events::keys::{
    handle_key_events_confirm_mode, handle_key_events_filter_mode, handle_key_events_go_to_mode,
    handle_key_events_help_mode, handle_key_events_normal_mode,
};
use crate::models::app::App;
use crate::models::app::AppState;
use crate::ui::renders::render;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use std::io::Result;
use std::time::Duration;

pub fn app_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        let Some(key) = handle_event_loop()? else {
            continue;
        };

        match app.state {
            AppState::Active => {
                if let AppState::Quit = handle_key_events_normal_mode(app, key)? {
                    break;
                }
            }
            AppState::Filter => {
                if let AppState::Active = handle_key_events_filter_mode(app, key)? {
                    app.state = AppState::Active;
                }
            }
            AppState::GoTo => {
                if let AppState::Active = handle_key_events_go_to_mode(app, key)? {
                    app.state = AppState::Active;
                }
            }
            AppState::Confirm => {
                if let AppState::Active = handle_key_events_confirm_mode(app, key)? {
                    app.state = AppState::Active;
                }
            }
            AppState::Help => {
                if let AppState::Active = handle_key_events_help_mode(app, key)? {
                    app.state = AppState::Active;
                }
            }
            AppState::Quit => {
                break;
            }
        }
    }
    Ok(())
}

fn handle_event_loop() -> Result<Option<KeyEvent>> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(None);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(None);
    };

    if key.kind != KeyEventKind::Press {
        return Ok(None);
    }

    Ok(Some(key))
}
