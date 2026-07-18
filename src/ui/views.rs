use super::utils::display_artifact_name;
use crate::config::app::Palette;
use ratatui::layout::Constraint;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use rtvui_core::Artifact;

const TABLE_COLUMNS: [&str; 3] = ["NAME", "SIZE", "MODIFIED"];

const HELP_LINES: [&str; 16] = [
    "Navigation",
    "  w/s or ↑↓     move selection",
    "  a/d or ←→     parent / enter directory",
    "  Enter         open file (or enter directory)",
    "  h / Home      jump to filesystem root",
    "  g             go to path  ·  / filter  ·  r refresh",
    "",
    "Options",
    "  t theme  ·  o sort  ·  . hidden  ·  F2 rename  ·  x/Del delete  ·  y copy",
    "  b bookmark cwd  ·  1-9 jump bookmark",
    "",
    "Modes",
    "  Esc           cancel filter / go-to / rename / help / delete",
    "  q             quit (normal) · close help",
    "  ?             this help",
    "  y / n         confirm / cancel delete (in confirm mode)",
];

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

pub fn get_help_overlay<'a>(palette: Palette) -> Paragraph<'a> {
    let lines: Vec<Line<'a>> = HELP_LINES.iter().map(|line| Line::from(*line)).collect();
    Paragraph::new(lines)
        .style(Style::new().fg(palette.text).bg(palette.background))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(palette.accent))
                .title(" Help "),
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
