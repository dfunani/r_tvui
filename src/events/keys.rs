use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;

use crate::models::app::App;
use crate::models::app::AppState;
use crate::utils::counter::decrement;
use crate::utils::counter::increment;

impl App {
    pub fn handle_key_event(&mut self, key_event: Event) {
        match key_event {
            Event::Key(key_event) if get_increment_keys().contains(&key_event.code) => {
                increment(&mut self.counter, 1)
            }
            Event::Key(key_event) if get_decrement_keys().contains(&key_event.code) => {
                decrement(&mut self.counter, 1)
            }
            Event::Key(key_event) if get_exit_keys().contains(&key_event.code) => {
                self.state = AppState::Exit
            }
            _ => (),
        }
    }
}

fn get_exit_keys() -> Vec<KeyCode> {
    vec![KeyCode::Char('Q'), KeyCode::Esc]
}

fn get_increment_keys() -> Vec<KeyCode> {
    vec![
        KeyCode::Up,
        KeyCode::Char('+'),
        KeyCode::Char('='),
        KeyCode::Right,
    ]
}

fn get_decrement_keys() -> Vec<KeyCode> {
    vec![
        KeyCode::Down,
        KeyCode::Char('-'),
        KeyCode::Char('_'),
        KeyCode::Left,
    ]
}
