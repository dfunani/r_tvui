use super::layout::{get_layout, get_path_bar, get_preview_layout, get_status_bar};
use super::views::{get_artifact_display, get_preview_display};
use crate::models::app::App;
use ratatui::Frame;
use ratatui::layout::Rect;
use rtvui_core::utils::get_artifact_entries;
use rtvui_core::{ArtifactOptions, ArtifactType};

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = get_layout();
    let segments = layout.split(frame.area());

    let preview_layout = get_preview_layout();
    let preview_segments = preview_layout.split(segments[1]);

    frame.render_widget(get_path_bar(app), segments[0]);
    frame.render_stateful_widget(
        get_artifact_display(&app.artifacts, " Files ".to_string()),
        preview_segments[0],
        &mut app.scroll_state,
    );
    handle_preview(frame, app, preview_segments[1]);

    frame.render_widget(get_status_bar(app), segments[2]);
}

fn handle_preview(frame: &mut Frame, app: &mut App, segment: Rect) {
    let Some(selection) = app.scroll_state.selected() else {
        return frame.render_widget(
            get_artifact_display(
                &app.artifacts,
                app.current_working_directory.display().to_string(),
            ),
            segment,
        );
    };

    let Some(artifact) = app.artifacts.get(selection) else {
        return frame.render_widget(
            get_artifact_display(
                &app.artifacts,
                app.current_working_directory.display().to_string(),
            ),
            segment,
        );
    };

    if artifact.artifact_type == ArtifactType::File {
        frame.render_widget(
            get_preview_display(artifact, artifact.name.to_string()),
            segment,
        );
    } else {
        let artifacts = get_artifact_entries(&artifact.path, &ArtifactOptions::default()).unwrap();
        frame.render_widget(
            get_artifact_display(&artifacts.artifacts, artifact.name.to_string()),
            segment,
        );
    }
}
