use crate::events::keys::dispatch_key;
use crate::models::app::App;
use crate::models::app::AppState;
use crate::os::open_with_editor;
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

        if let Some(path) = app.pending_editor.take() {
            run_editor(terminal, app, path)?;
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

/// Suspend the TUI (leave raw mode / alt screen), run `$EDITOR`, restore.
fn run_editor(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    path: std::path::PathBuf,
) -> Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    ratatui::restore();
    let result = open_with_editor(path);
    *terminal = ratatui::init();
    terminal.clear()?;

    match result {
        Ok(()) => {
            app.status_message = format!("Edited {name}");
            app.refresh()?;
        }
        Err(error) => app.status_message = format!("Editor failed: {error}"),
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
