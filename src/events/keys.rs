use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::models::app::{App, AppState};
use crate::utils::browser::update_side_pane;
use crate::utils::navigation::{
    activate_selected, go_home, go_parent, move_selection, navigate_into_selected,
    refresh_listing, toggle_hidden,
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
                if !self.entries.is_empty() {
                    self.selected = 0;
                    update_side_pane(self);
                }
            }
            KeyCode::End => {
                if !self.entries.is_empty() {
                    self.selected = self.entries.len() - 1;
                    update_side_pane(self);
                }
            }
            KeyCode::Enter => activate_selected(self),
            KeyCode::Right | KeyCode::Char('l') => navigate_into_selected(self),
            KeyCode::Left | KeyCode::Char('h') => go_parent(self),
            KeyCode::Char('G') => go_home(self),
            KeyCode::Char('r') => refresh_listing(self),
            KeyCode::Char('.') => toggle_hidden(self),
            KeyCode::Char('t') | KeyCode::Char('T') => self.cycle_theme(),
            _ => (),
        }
    }
}
