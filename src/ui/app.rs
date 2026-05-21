use ratatui::{Frame, buffer::Buffer, layout::Rect, widgets::Widget};

use crate::models::app::App;
use crate::ui::render::render_main_view;

impl App {
    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_main_view(area, buf, self);
    }
}
