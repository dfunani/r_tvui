use ratatui::crossterm::event;
use ratatui::{DefaultTerminal, Frame};

use crate::models::app::App;
use crate::models::app::AppState;
use std::io::Result;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.show_cursor()?;
        while self.state != AppState::Exit {
            terminal.draw(|frame| self.render(frame))?;
            let key_event = event::read()?;
            self.handle_key_event(key_event);
        }
        Ok(())
    }
}
