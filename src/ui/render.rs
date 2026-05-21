use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::models::app::App;

pub fn render_main_view(area: Rect, buf: &mut Buffer, app: &App) {
    let title = Line::from(Span::styled("R-TVUI", Style::default().bold()));
    let instructions = Line::from(get_instructions());

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let counter_text_lines = Line::from(get_counter_text(app.counter));
    let counter_text = Text::from(vec![counter_text_lines]);

    Paragraph::new(counter_text)
        .centered()
        .block(block)
        .render(area, buf)
}

pub fn render_main_view_text(area: Rect, buf: &mut Buffer, app: &App) {
    Paragraph::new(app.counter.to_string())
        .centered()
        .render(area, buf)
}

fn get_instructions<'a>() -> Vec<Span<'a>> {
    vec![
        Span::styled(" Decrement ", Style::default().bold()),
        Span::styled("<Up>", Style::default().blue().bold()),
        Span::styled(" Increment ", Style::default().bold()),
        Span::styled("<Down>", Style::default().blue().bold()),
        Span::styled(" Quit ", Style::default().bold()),
        Span::styled("<Q> ", Style::default().blue().bold()),
    ]
}

fn get_counter_text<'a>(counter: u32) -> Vec<Span<'a>> {
    vec![
        Span::styled("Counter: ", Style::default().bold()),
        Span::styled(counter.to_string(), Style::default().bold().yellow()),
    ]
}
