use crate::events::keys::dispatch_key;
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

        dispatch_key(app, key)?;
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
