use rtvui_core::ArtifactType;

use crate::models::app::App;
use std::io::Result;

pub fn scroll_up(app: &mut App) -> Result<()> {
    let Some(mut selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    if selection > 0 {
        selection = selection.saturating_sub(1);
    }
    app.scroll_state.select(Some(selection));
    Ok(())
}

pub fn scroll_down(app: &mut App) -> Result<()> {
    let Some(mut selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    if selection + 1 < app.artifacts.len() {
        selection = selection.saturating_add(1);
    }
    app.scroll_state.select(Some(selection));
    Ok(())
}

pub fn scroll_back(app: &mut App) -> Result<()> {
    app.scroll_state.select(Some(0));
    app.reload()?;
    Ok(())
}

pub fn scroll_home(app: &mut App) -> Result<()> {
    for _ in 0..app.current_working_directory.components().count() {
        app.current_working_directory.pop();
    }
    app.scroll_state.select(Some(0));
    app.reload()?;
    Ok(())
}

pub fn scroll_forward(app: &mut App) -> Result<()> {
    let Some(selection) = app.scroll_state.selected() else {
        return Ok(());
    };
    let Some(artifact) = app.artifacts.get(selection) else {
        return Ok(());
    };
    if artifact.artifact_type == ArtifactType::Directory {
        app.current_working_directory.push(&artifact.name);
        app.scroll_state.select(Some(0));
        app.reload()?;
    }
    Ok(())
}
