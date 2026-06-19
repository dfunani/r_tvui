use crate::events::keys::{
    handle_key_events_confirm_mode, handle_key_events_filter_mode, handle_key_events_go_to_mode,
    handle_key_events_help_mode, handle_key_events_normal_mode, handle_key_events_rename_mode,
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
        let events: Vec<_> = app.async_client.drain().collect();
        for event in events {
            app.apply_async_event(event);
        }
        terminal.draw(|frame| render(frame, app))?;

        let Some(key) = handle_event_loop()? else {
            continue;
        };

        match app.state {
            AppState::Active => {
                app.state = handle_key_events_normal_mode(app, key)?;
            }
            AppState::Filter => {
                app.state = handle_key_events_filter_mode(app, key)?;
            }
            AppState::Rename => {
                app.state = handle_key_events_rename_mode(app, key)?;
            }
            AppState::GoTo => {
                app.state = handle_key_events_go_to_mode(app, key)?;
            }
            AppState::Confirm => {
                app.state = handle_key_events_confirm_mode(app, key)?;
            }
            AppState::Help => {
                app.state = handle_key_events_help_mode(app, key)?;
            }
            AppState::Quit => {
                break;
            }
        }
        if app.state == AppState::Quit {
            break;
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
