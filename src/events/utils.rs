use crate::models::app::App;
use crate::os::open_file;
use rtvui_core::ArtifactType;
use std::io::Result;

pub fn scroll_up(app: &mut App) -> Result<()> {
    let Some(mut selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    selection = selection.saturating_sub(1);
    app.scroll_state.select(Some(selection));
    app.request_previewer();
    Ok(())
}

pub fn scroll_down(app: &mut App) -> Result<()> {
    let Some(mut selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    if selection + 1 < app.entries_filtered.len() {
        selection = selection.saturating_add(1);
    }
    app.scroll_state.select(Some(selection));
    app.request_previewer();
    Ok(())
}

pub fn scroll_back(app: &mut App) -> Result<()> {
    app.scroll_state.select(Some(0));
    app.record_history();
    app.async_reload()?;
    Ok(())
}

/// Walk to the filesystem root (`/`).
pub fn scroll_home(app: &mut App) -> Result<()> {
    for _ in 0..app.current_working_directory.components().count() {
        app.current_working_directory.pop();
    }
    app.scroll_state.select(Some(0));
    app.record_history();
    app.async_reload()?;
    Ok(())
}

pub fn scroll_forward(app: &mut App) -> Result<()> {
    let Some(selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    let Some(artifact) = app.entries_filtered.get(selection) else {
        return Ok(());
    };
    if artifact.artifact_type == ArtifactType::Directory {
        app.current_working_directory.push(&artifact.name);
        app.scroll_state.select(Some(0));
        app.record_history();
        app.async_reload()?;
    }
    Ok(())
}

pub fn handle_key_event_enter_mode(app: &mut App) -> Result<()> {
    let Some(selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    let Some(artifact) = app.entries_filtered.get(selection) else {
        return Ok(());
    };
    let path = artifact.path.clone();
    let name = artifact.name.clone();
    open_file(path)?;
    app.status_message = format!("Opening {name}");
    Ok(())
}
