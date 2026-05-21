use core::formatters::format_size;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::models::app::App;
use crate::utils::previewer::{cwd_display, entry_label};

pub fn render_main_view(area: Rect, buf: &mut Buffer, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    render_path_bar(layout[0], buf, app);
    render_main_panes(layout[1], buf, app);
    render_status_bar(layout[2], buf, app);
}

fn render_path_bar(area: Rect, buf: &mut Buffer, app: &App) {
    let path = cwd_display(&app.cwd.0);
    let hidden = if app.show_hidden { "hidden: on" } else { "hidden: off" };
    let title = Line::from(vec![
        Span::styled(" R-TVUI ", Style::default().bold()),
        Span::raw(" "),
        Span::styled(path, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(hidden, Style::default().fg(Color::DarkGray)),
    ]);

    Block::bordered()
        .title(title)
        .border_set(border::ROUNDED)
        .render(area, buf);
}

fn render_main_panes(area: Rect, buf: &mut Buffer, app: &App) {
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    render_file_list(panes[0], buf, app);
    render_preview_pane(panes[1], buf, app);
}

fn render_file_list(area: Rect, buf: &mut Buffer, app: &App) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let style = if entry.hidden {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(entry_label(entry)).style(style)
        })
        .collect();

    let block = Block::bordered()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    if !app.entries.is_empty() {
        state.select(Some(app.selected));
    }

    StatefulWidget::render(list, area, buf, &mut state);
}

fn render_preview_pane(area: Rect, buf: &mut Buffer, app: &App) {
    let title = app
        .selected_entry()
        .map(|e| format!(" Preview: {} ", e.name))
        .unwrap_or_else(|| " Preview ".to_string());

    let block = Block::bordered()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    Paragraph::new(app.preview.as_str())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((0, 0))
        .render(area, buf);
}

fn render_status_bar(area: Rect, buf: &mut Buffer, app: &App) {
    let selected_info = app.selected_entry().map(|e| {
        let size = e.size.map(format_size).unwrap_or_else(|| "-".to_string());
        format!("{} ({size})", e.name)
    });

    let line = Line::from(vec![
        Span::styled(app.status.clone(), Style::default()),
        Span::raw("  |  "),
        Span::styled(
            get_key_hints(),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  |  "),
        Span::styled(
            selected_info.unwrap_or_else(|| "—".to_string()),
            Style::default().fg(Color::Yellow),
        ),
    ]);

    Block::bordered()
        .border_set(border::PLAIN)
        .render(area, buf);

    Paragraph::new(line).render(
        Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width.saturating_sub(2),
            height: area.height,
        },
        buf,
    );
}

fn get_key_hints() -> &'static str {
    "j/k move  l/Enter open  h/..  . hidden  r refresh  G home  q quit"
}
