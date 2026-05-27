use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};
use rtvui_core::formatters::format_size;

use crate::models::app::{App, SidePane};
use crate::models::mode::{AppMode, InputKind};
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

    match app.mode {
        AppMode::Help => render_help_overlay(area, buf, app, &theme),
        AppMode::Input(kind) => render_input_overlay(layout[2], buf, app, kind, &theme),
        AppMode::ConfirmDelete => render_confirm_overlay(layout[2], buf, app, &theme),
        AppMode::Normal => {}
    }
}

fn render_path_bar(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    let path = cwd_display(&app.cwd.0);
    let hidden = if app.show_hidden {
        Span::styled(" hidden ", theme.kicker_style())
    } else {
        Span::styled(" hidden off ", theme.muted_style())
    };
    let sort = Span::styled(
        format!(" sort:{} ", app.sort_pref.label()),
        theme.kicker_style(),
    );
    let filter = if app.filter_query.is_empty() {
        Span::raw("")
    } else {
        Span::styled(
            format!(" filter:{} ", app.filter_query),
            theme.theme_badge_style(),
        )
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
        sort,
        filter,
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
    entries: &[rtvui_core::paths::File],
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

    let hints = match app.mode {
        AppMode::Normal => get_key_hints(),
        AppMode::Help => "Esc/? close help",
        AppMode::Input(_) => "Enter confirm · Esc cancel",
        AppMode::ConfirmDelete => "y delete · n cancel",
    };

    let block = theme.status_block();
    let inner = block.inner(area);
    block.render(area, buf);

    let line = Line::from(vec![
        Span::styled(app.status.clone(), theme.muted_style()),
        Span::styled(" │ ", theme.hint_style()),
        Span::styled(hints, theme.hint_style()),
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

fn render_help_overlay(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    Clear.render(area, buf);
    let help = help_text(app);
    let block = theme
        .panel_block("Help · R-TVUI")
        .border_set(border::ROUNDED)
        .style(Style::default().bg(theme.bg_2).add_modifier(Modifier::BOLD));
    let inner = block.inner(area);
    block.render(area, buf);
    Paragraph::new(help)
        .style(theme.preview_body_style())
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

fn render_input_overlay(
    area: Rect,
    buf: &mut Buffer,
    app: &App,
    kind: InputKind,
    theme: &ThemePalette,
) {
    let title = match kind {
        InputKind::Filter => "Filter",
        InputKind::GoToPath => "Go to path",
        InputKind::Rename => "Rename",
    };
    let block = theme.status_block().title(format!(" {title} "));
    let inner = block.inner(area);
    block.render(area, buf);
    let line = Line::from(vec![
        Span::styled("▸ ", theme.brand_style()),
        Span::styled(&app.input_buffer, theme.path_style()),
        Span::styled("█", theme.kicker_style()),
    ]);
    Paragraph::new(line).render(inner, buf);
}

fn render_confirm_overlay(area: Rect, buf: &mut Buffer, app: &App, theme: &ThemePalette) {
    let target = app
        .delete_target
        .as_ref()
        .map(|p| p.0.display().to_string())
        .unwrap_or_else(|| "?".to_string());
    let block = theme.status_block().title(" Confirm delete ");
    let inner = block.inner(area);
    block.render(area, buf);
    let via = if app.use_trash { "trash" } else { "permanent" };
    Paragraph::new(format!("{target}\n({via})  y confirm · n cancel"))
        .style(theme.preview_body_style())
        .render(inner, buf);
}

fn help_text(app: &App) -> String {
    let mut lines = vec![
        "Navigation".to_string(),
        "  j/k, ↑/↓     move selection".to_string(),
        "  l, Enter     open folder / file".to_string(),
        "  h            parent directory".to_string(),
        "  g            go to path".to_string(),
        "  G            home".to_string(),
        "  u / i        history back / forward".to_string(),
        "  1-9          jump to bookmark".to_string(),
        "  b            bookmark current folder".to_string(),
        "".to_string(),
        "View".to_string(),
        "  /            filter by name".to_string(),
        "  s            cycle sort (name/size/mtime)".to_string(),
        "  .            toggle hidden files".to_string(),
        "  p            refresh preview".to_string(),
        "  P            toggle preview-on-move".to_string(),
        "  t            cycle theme".to_string(),
        "  r            refresh listing".to_string(),
        "  ?            this help".to_string(),
        "".to_string(),
        "Actions".to_string(),
        "  y            copy path to clipboard".to_string(),
        "  F2           rename".to_string(),
        "  d            delete (confirm)".to_string(),
        "  q, Esc       quit (Esc clears filter first)".to_string(),
    ];
    if !app.bookmarks.is_empty() {
        lines.push("".to_string());
        lines.push("Bookmarks".to_string());
        for (i, b) in app.bookmarks.iter().enumerate() {
            lines.push(format!("  {}  {}", i + 1, b));
        }
    }
    lines.join("\n")
}

fn get_key_hints() -> &'static str {
    "? help · / filter · Esc quit · q quit · y copy · d del"
}
