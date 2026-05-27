use std::time::Duration;

use ratatui::DefaultTerminal;
use ratatui::crossterm::event;
use std::io::Result;

use crate::models::app::{App, AppState};
use crate::utils::navigation::poll_listing_events;

const EVENT_POLL: Duration = Duration::from_millis(16);

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.state != AppState::Exit {
            poll_listing_events(self);

            if self.mode.is_input() {
                terminal.show_cursor()?;
            } else {
                terminal.hide_cursor()?;
            }
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(EVENT_POLL)? {
                let key_event = event::read()?;
                self.handle_key_event(key_event);
            }
        }
        terminal.hide_cursor()?;
        self.persist_config();
        Ok(())
    }
}
