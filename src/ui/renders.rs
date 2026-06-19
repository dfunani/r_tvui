use super::layout::{get_layout, get_path_bar, get_preview_layout, get_status_bar};
use super::views::{get_artifact_display, get_preview_text};
use crate::models::app::App;
use crate::models::previewer::Previewer;
use ratatui::Frame;
use ratatui::layout::Rect;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = get_layout();
    let segments = layout.split(frame.area());

    let preview_layout = get_preview_layout();
    let preview_segments = preview_layout.split(segments[1]);

    frame.render_widget(get_path_bar(app), segments[0]);

    frame.render_stateful_widget(
        get_artifact_display(&app.entries_filtered, " Files ".to_string(), app.palette()),
        preview_segments[0],
        &mut app.scroll_state,
    );
    handle_preview(frame, app, preview_segments[1]);

    frame.render_widget(get_status_bar(app), segments[2]);
}

fn handle_preview(frame: &mut Frame, app: &mut App, segment: Rect) {
    match &app.previewer {
        Previewer::Empty => frame.render_widget(
            get_artifact_display(
                &app.entries_filtered,
                app.current_working_directory.display().to_string(),
                app.palette(),
            ),
            segment,
        ),
        Previewer::Folder(folder) => frame.render_widget(
            get_artifact_display(&folder.artifacts, folder.title.clone(), app.palette()),
            segment,
        ),
        Previewer::Preview(preview) => frame.render_widget(
            get_preview_text(&preview.body, preview.title.clone(), app.palette()),
            segment,
        ),
    }
}
