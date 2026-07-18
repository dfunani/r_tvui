use crate::models::app::{App, AppState};
use ratatui::layout::{Constraint, Direction, Layout};
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
    let mut status_message = String::from(" w/s ↑↓ · a/d ⇆ · d enter· q quit ");

    if app.state == AppState::Rename {
        status_message = format!(
            " rename: {}▏ · Enter confirm · Esc cancel ",
            app.rename_input
        );
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

pub fn get_layout() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(50),
            Constraint::Length(1),
        ])
}

pub fn get_preview_layout() -> Layout {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
}
