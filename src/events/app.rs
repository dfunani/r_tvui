use ratatui::crossterm::event;
use ratatui::DefaultTerminal;

use crate::models::app::App;
use crate::models::app::AppState;
use std::io::Result;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != AppState::Exit {
            if self.mode.is_input() {
                terminal.show_cursor()?;
            } else {
                terminal.hide_cursor()?;
            }
            terminal.draw(|frame| self.render(frame))?;
            let key_event = event::read()?;
            self.handle_key_event(key_event);
        }
        terminal.hide_cursor()?;
        self.persist_config();
        Ok(())
    }
}
