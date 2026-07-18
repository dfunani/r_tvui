use super::utils::display_artifact_name;
use crate::config::app::Palette;
use crate::models::pane::BrowserPane;
use ratatui::layout::Constraint;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use rtvui_core::Artifact;

const TABLE_COLUMNS: [&str; 4] = ["", "NAME", "SIZE", "MODIFIED"];

const HELP_LINES: [&str; 27] = [
    "Navigation",
    "  w/s or j/k or ↑↓   move selection",
    "  a/d or l or ←→     parent / enter directory",
    "  Enter              open file (or enter directory)",
    "  h / Home           jump to filesystem root",
    "  G                  jump to $HOME",
    "  u / i              history back / forward",
    "  g                  go to path  ·  / filter  ·  r refresh",
    "",
    "Tabs & split",
    "  N / W              new tab / close tab",
    "  [ / ]              previous / next tab",
    "  \\                  toggle dual-cwd split",
    "  Tab                focus other split pane",
    "  c / m              copy / move selection to other pane",
    "",
    "Selection & files",
    "  Space / U          mark / clear marks",
    "  t theme · o sort · . hidden · P preview · F2 rename",
    "  x/Del delete · y copy path(s) · b bookmark · 1-9 jump",
    "  e                  open in $EDITOR",
    "",
    "Modes",
    "  Esc                cancel mode / quit from normal",
    "  q                  quit (normal) · close help",
    "  ?                  this help",
    "  y / n              confirm / cancel delete",
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

pub fn get_simple_table<'a>(artifacts: &[Artifact], title: String, palette: Palette) -> Table<'a> {
    build_table(artifacts, title, palette, None, None)
}

pub fn get_pane_table<'a>(pane: &BrowserPane, title: String, palette: Palette) -> Table<'a> {
    build_table(
        &pane.entries_filtered,
        title,
        palette,
        Some(&pane.marked),
        Some(&pane.git_marks),
    )
}

fn build_table<'a>(
    artifacts: &[Artifact],
    title: String,
    palette: Palette,
    marked: Option<&std::collections::HashSet<std::path::PathBuf>>,
    git_marks: Option<&std::collections::HashMap<String, char>>,
) -> Table<'a> {
    let rows: Vec<Row<'a>> = artifacts
        .iter()
        .map(|artifact| {
            let mark = if marked.is_some_and(|set| set.contains(&artifact.path)) {
                "*"
            } else {
                " "
            };
            let git = git_marks
                .and_then(|map| map.get(&artifact.name).copied())
                .map(|c| c.to_string())
                .unwrap_or_default();
            let base = display_artifact_name(artifact);
            let name = if git.is_empty() {
                base
            } else {
                format!("[{git}] {base}")
            };
            Row::new(vec![
                Cell::from(mark),
                Cell::from(name),
                Cell::from(artifact.format_size()),
                Cell::from(artifact.format_modified()),
            ])
        })
        .collect();

    Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Percentage(50),
            Constraint::Length(10),
            Constraint::Length(18),
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
