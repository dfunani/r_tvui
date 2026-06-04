use super::utils::{display_artifact_name, read_artifact_content};
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use rtvui_core::Artifact;

const TABLE_COLUMNS: [&str; 3] = ["NAME", "SIZE", "MODIFIED"];

pub fn get_preview_display<'a>(artifact: &'a Artifact, title: String) -> Paragraph<'a> {
    let content = read_artifact_content(artifact)
        .unwrap_or_else(|_| "Content Preview Unavailable".to_string());
    Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(title))
}

pub fn get_artifact_display<'a>(artifacts: &[Artifact], title: String) -> Table<'a> {
    Table::new(
        artifacts
            .iter()
            .map(|artifact| {
                Row::new(vec![
                    Cell::from(display_artifact_name(artifact)),
                    Cell::from(artifact.format_size()),
                    Cell::from("-"),
                ])
            })
            .collect::<Vec<Row>>(),
        [
            Constraint::Percentage(50),
            Constraint::Length(10),
            Constraint::Length(20),
        ],
    )
    .header(Row::new(TABLE_COLUMNS).style(Style::new().add_modifier(Modifier::BOLD)))
    .row_highlight_style(
        Style::new()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Cyan),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
}
