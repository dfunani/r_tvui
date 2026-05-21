use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::models::app::{App, AppState};
use crate::utils::navigation::{
    enter_selected, go_home, go_parent, move_selection, refresh_listing, toggle_hidden,
};

impl App {
    pub fn handle_key_event(&mut self, key_event: Event) {
        let Event::Key(key) = key_event else {
            return;
        };
        if key.kind == KeyEventKind::Release {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.state = AppState::Exit;
            }
            KeyCode::Up | KeyCode::Char('k') => move_selection(self, -1),
            KeyCode::Down | KeyCode::Char('j') => move_selection(self, 1),
            KeyCode::Home => {
                self.selected = 0;
                crate::utils::previewer::refresh_preview(self);
            }
            KeyCode::End => {
                if !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                    crate::utils::previewer::refresh_preview(self);
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => enter_selected(self),
            KeyCode::Left | KeyCode::Char('h') => go_parent(self),
            KeyCode::Char('G') => go_home(self),
            KeyCode::Char('r') => refresh_listing(self),
            KeyCode::Char('.') => toggle_hidden(self),
            _ => (),
        }
    }
}
