use super::utils::display_artifact_name;
use crate::config::app::Palette;
use ratatui::layout::Constraint;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use rtvui_core::Artifact;

const TABLE_COLUMNS: [&str; 3] = ["NAME", "SIZE", "MODIFIED"];

pub fn get_preview_text<'a>(body: &str, title: String, palette: Palette) -> Paragraph<'a> {
    Paragraph::new(body.to_string())
        .style(Style::new().fg(palette.text).bg(palette.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(palette.accent))
                .title(title),
        )
}

pub fn get_artifact_display<'a>(
    artifacts: &[Artifact],
    title: String,
    palette: Palette,
) -> Table<'a> {
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
    .style(Style::new().fg(palette.text).bg(palette.background))
    .header(
        Row::new(TABLE_COLUMNS).style(Style::new().add_modifier(Modifier::BOLD).fg(palette.header)),
    )
    .row_highlight_style(
        Style::new()
            .add_modifier(Modifier::REVERSED)
            .fg(palette.highlight),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(palette.accent))
            .title(title),
    )
}
