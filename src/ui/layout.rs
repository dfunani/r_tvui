use crate::models::app::{App, AppState};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn get_path_bar<'a>(app: &'a App) -> Line<'a> {
    let accent = app.palette().accent;
    Line::from(vec![
        Span::raw(" path: "),
        Span::styled(
            app.current_working_directory.display().to_string(),
            Style::new().add_modifier(Modifier::BOLD).fg(accent),
        ),
    ])
}

pub fn get_status_bar<'a>(app: &'a App) -> Paragraph<'a> {
    let accent = app.palette().accent;
    let mut status_message =
        String::from(
            " w/s ↑↓ · a/d ⇆ · Enter open · / filter · g go · y copy · x del · ? help · q quit ",
        );

    if app.state == AppState::Rename {
        status_message = format!(
            " rename: {}▏ · Enter confirm · Esc cancel ",
            app.rename_input
        );
    } else if app.state == AppState::GoTo {
        status_message = format!(
            " go to: {}▏ · Enter jump · Esc cancel ",
            app.goto_input
        );
    } else if app.state == AppState::Filter {
        status_message = format!(
            " filter: {}▏ · Esc clear ",
            app.filter_input
        );
    } else if app.state == AppState::Help {
        status_message = String::from(" help · Esc/q close ");
    } else if app.state == AppState::Confirm {
        let name = app
            .selected_artifact()
            .map(|artifact| artifact.name.as_str())
            .unwrap_or("?");
        let via = if app.config.settings.enable_trash {
            "trash"
        } else {
            "permanently"
        };
        status_message = format!(" delete {name} ({via})? · y confirm · n/Esc cancel ");
    } else if !app.status_message.is_empty() {
        status_message = format!(" {} · {} ", app.status_message, status_message);
    }

    let bar = Line::from(vec![
        Span::raw(" status: "),
        Span::styled(
            status_message.to_string(),
            Style::new().add_modifier(Modifier::BOLD).fg(accent),
        ),
    ]);
    Paragraph::new(bar)
}

/// Centered rectangle covering `percent_x` × `percent_y` of `area`.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn get_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
}

pub fn get_preview_layout() -> Layout {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
}
