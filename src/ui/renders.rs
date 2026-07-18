use super::layout::{
    centered_rect, get_layout, get_path_bar, get_preview_layout, get_split_layout, get_status_bar,
};
use super::views::{get_help_overlay, get_pane_table, get_preview_text, get_simple_table};
use crate::models::app::{App, AppState};
use crate::models::previewer::Previewer;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;

pub fn render(frame: &mut Frame, app: &mut App) {
    let layout = get_layout();
    let segments = layout.split(frame.area());
    let palette = app.palette();

    frame.render_widget(get_path_bar(app), segments[0]);

    if let Some(peer) = app.split_tab {
        let split_layout = get_split_layout();
        let panes = split_layout.split(segments[1]);
        let active = app.active_tab;

        let left = get_pane_table(&app.tabs[active], format!(" Tab {} ", active + 1), palette);
        let right = get_pane_table(&app.tabs[peer], format!(" Tab {} ", peer + 1), palette);

        frame.render_stateful_widget(left, panes[0], &mut app.tabs[active].scroll_state);
        // Peer list is non-stateful highlight when unfocused.
        frame.render_widget(right, panes[1]);
    } else {
        let preview_layout = get_preview_layout();
        let preview_segments = preview_layout.split(segments[1]);
        let active = app.active_tab;
        let title = format!(" Files · tab {} ", active + 1);
        let table = get_pane_table(&app.tabs[active], title, palette);
        frame.render_stateful_widget(
            table,
            preview_segments[0],
            &mut app.tabs[active].scroll_state,
        );
        handle_preview(frame, app, preview_segments[1]);
    }

    if app.state == AppState::Help {
        let area = centered_rect(70, 80, segments[1]);
        frame.render_widget(Clear, area);
        frame.render_widget(get_help_overlay(palette), area);
    }

    frame.render_widget(get_status_bar(app), segments[2]);
}

fn handle_preview(frame: &mut Frame, app: &App, segment: Rect) {
    let palette = app.palette();
    match app.previewer() {
        Previewer::Empty => {
            let title = app.current_working_directory().display().to_string();
            let table = get_simple_table(app.entries_filtered(), title, palette);
            frame.render_widget(table, segment);
        }
        Previewer::Folder(folder) => {
            let table = get_simple_table(&folder.artifacts, folder.title.clone(), palette);
            frame.render_widget(table, segment);
        }
        Previewer::Preview(preview) => {
            frame.render_widget(
                get_preview_text(&preview.body, preview.title.clone(), palette),
                segment,
            );
        }
    }
}
