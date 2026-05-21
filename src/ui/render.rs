use core::formatters::format_size;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::models::app::{App, SidePane};
use crate::theme::ThemePalette;
use crate::utils::previewer::{cwd_display, entry_label};

pub fn render_main_view(area: Rect, buf: &mut Buffer, app: &App) {
    let theme = app.theme.palette();
    Block::default().style(theme.base()).render(area, buf);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(area);

    render_path_bar(layout[0], buf, app, &theme);
    render_main_panes(layout[1], buf, app, &theme);
    render_status_bar(layout[2], buf, app, &theme);
}

fn render_path_bar(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    let path = cwd_display(&app.cwd.0);
    let hidden = if app.show_hidden {
        Span::styled(" hidden ", theme.kicker_style())
    } else {
        Span::styled(" hidden off ", theme.muted_style())
    };

    let block = theme.path_bar_block().border_set(border::ROUNDED);
    let inner = block.inner(area);
    block.render(area, buf);

    let line = Line::from(vec![
        Span::styled(" R-TVUI ", theme.brand_style()),
        Span::styled(" │ ", theme.hint_style()),
        Span::styled(path, theme.path_style()),
        Span::raw("  "),
        Span::styled(app.theme.name(), theme.theme_badge_style()),
        Span::raw("  "),
        hidden,
    ]);

    Paragraph::new(line)
        .style(Style::default().bg(theme.bg_2))
        .render(inner, buf);
}

fn render_main_panes(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    if !app.side_pane.is_active() {
        render_explorer_list(area, buf, app, theme, "Browser");
        return;
    }

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(area);

    render_explorer_list(panes[0], buf, app, theme, "Browser");
    render_side_pane(panes[1], buf, app, theme);
}

fn render_explorer_list(
    area: Rect,
    buf: &mut Buffer,
    app: &App,
    theme: &ThemePalette,
    title: &str,
) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            let glyph = ThemePalette::entry_glyph(entry);
            let label = entry_label(entry);
            let line = Line::from(vec![
                Span::styled(format!("{glyph} "), theme.entry_style(entry)),
                Span::styled(label, theme.entry_style(entry)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = theme.panel_block(title).border_set(border::ROUNDED);

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selection_style())
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    if !app.entries.is_empty() {
        state.select(Some(app.selected));
    }

    StatefulWidget::render(list, area, buf, &mut state);
}

fn render_side_pane(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    match &app.side_pane {
        SidePane::Hidden => {}
        SidePane::Folder { path, entries } => {
            let title = path
                .0
                .file_name()
                .map(|n| format!("{} ", n.to_string_lossy()))
                .unwrap_or_else(|| "Folder ".to_string());
            render_folder_pane(area, buf, entries, &title, theme);
        }
        SidePane::Preview { title, body } => {
            render_preview_pane(area, buf, title, body, theme);
        }
    }
}

fn render_folder_pane(
    area: Rect,
    buf: &mut Buffer,
    entries: &[core::paths::File],
    title: &str,
    theme: &ThemePalette,
) {
    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let glyph = ThemePalette::entry_glyph(entry);
            let label = entry_label(entry);
            let line = Line::from(vec![
                Span::styled(format!("{glyph} "), theme.entry_style(entry)),
                Span::styled(label, theme.entry_style(entry)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = theme.panel_block(title).border_set(border::ROUNDED);
    let list = List::new(items).block(block);

    Widget::render(list, area, buf);
}

fn render_preview_pane(
    area: Rect,
    buf: &mut Buffer,
    title: &str,
    body: &str,
    theme: &ThemePalette,
) {
    let block = theme
        .panel_block(&format!("Preview · {title}"))
        .border_set(border::ROUNDED);
    let inner = block.inner(area);
    block.render(area, buf);

    Paragraph::new(body)
        .style(theme.preview_body_style())
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

fn render_status_bar(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    let selected_info = app.selected_entry().map(|e| {
        let size = e.size.map(format_size).unwrap_or_else(|| "—".to_string());
        format!("{} · {size}", e.name)
    });

    let block = theme.status_block();
    let inner = block.inner(area);
    block.render(area, buf);

    let line = Line::from(vec![
        Span::styled(app.status.clone(), theme.muted_style()),
        Span::styled(" │ ", theme.hint_style()),
        Span::styled(get_key_hints(), theme.hint_style()),
        Span::styled(" │ ", theme.hint_style()),
        Span::styled(
            selected_info.unwrap_or_else(|| "—".to_string()),
            theme.status_value_style(),
        ),
    ]);

    Paragraph::new(line)
        .style(Style::default().bg(theme.bg))
        .render(inner, buf);
}

fn get_key_hints() -> &'static str {
    "j/k · Enter open · l folder · h .. · t theme · . hidden · G home · q quit"
}
