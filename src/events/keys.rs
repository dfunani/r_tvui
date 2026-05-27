use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

use crate::models::app::{App, AppState};
use crate::models::mode::{AppMode, InputKind};
use crate::utils::clipboard;
use crate::utils::navigation::{
    activate_selected, bookmark_cwd, confirm_delete, confirm_filter, confirm_goto_path,
    confirm_rename, force_preview, go_home, go_parent, goto_bookmark, history_back,
    history_forward, maybe_update_side_pane, move_selection, navigate_into_selected,
    refresh_listing_force, start_delete_confirm, toggle_hidden,
};

impl App {
    pub fn handle_key_event(&mut self, key_event: Event) {
        let Event::Key(key) = key_event else {
            return;
        };
        if key.kind == KeyEventKind::Release {
            return;
        }

        match self.mode {
            AppMode::Help => self.handle_help_keys(key.code, key.modifiers),
            AppMode::Input(kind) => self.handle_input_keys(kind, key.code, key.modifiers),
            AppMode::ConfirmDelete => self.handle_confirm_delete_keys(key.code),
            AppMode::Normal => self.handle_normal_keys(key.code, key.modifiers),
        }
    }

    fn handle_help_keys(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if matches!(
            code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Char('Q')
        ) && modifiers.is_empty()
        {
            self.cancel_mode();
        }
    }

    fn handle_confirm_delete_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => confirm_delete(self),
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.cancel_mode();
                self.status = "Delete cancelled".to_string();
            }
            _ => (),
        }
    }

    fn handle_input_keys(&mut self, kind: InputKind, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Esc => {
                self.cancel_mode();
                self.status = "Cancelled".to_string();
            }
            KeyCode::Enter => {
                let value = self.input_buffer.clone();
                self.cancel_mode();
                match kind {
                    InputKind::Filter => confirm_filter(self, &value),
                    InputKind::GoToPath => confirm_goto_path(self, &value),
                    InputKind::Rename => confirm_rename(self, &value),
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT => {
                self.input_buffer.push(c);
            }
            _ => (),
        }
    }

    fn handle_normal_keys(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') if modifiers.is_empty() => {
                self.state = AppState::Exit;
            }
            KeyCode::Esc if modifiers.is_empty() => {
                if self.has_active_subquery() {
                    self.clear_filter();
                } else {
                    self.state = AppState::Exit;
                }
            }
            KeyCode::Char('?') if modifiers.is_empty() => {
                self.mode = AppMode::Help;
            }
            KeyCode::Up | KeyCode::Char('k') => move_selection(self, -1),
            KeyCode::Down | KeyCode::Char('j') => move_selection(self, 1),
            KeyCode::Home if !self.entries.is_empty() => {
                self.selected = 0;
                maybe_update_side_pane(self);
            }
            KeyCode::End if !self.entries.is_empty() => {
                self.selected = self.entries.len() - 1;
                maybe_update_side_pane(self);
            }
            KeyCode::Enter => activate_selected(self),
            KeyCode::Right | KeyCode::Char('l') => navigate_into_selected(self),
            KeyCode::Left | KeyCode::Char('h') => go_parent(self),
            KeyCode::Char('G') => go_home(self),
            KeyCode::Char('r') if modifiers.is_empty() => refresh_listing_force(self),
            KeyCode::Char('.') if modifiers.is_empty() => toggle_hidden(self),
            KeyCode::Char('t') | KeyCode::Char('T') if modifiers.is_empty() => self.cycle_theme(),
            KeyCode::Char('s') if modifiers.is_empty() => self.cycle_sort(),
            KeyCode::Char('g') if modifiers.is_empty() => {
                self.enter_input(InputKind::GoToPath, self.cwd.0.display().to_string());
            }
            KeyCode::Char('/') if modifiers.is_empty() => {
                self.enter_input(InputKind::Filter, self.filter_query.clone());
            }
            KeyCode::Char('p') if modifiers.is_empty() => force_preview(self),
            KeyCode::Char('P') if modifiers == KeyModifiers::SHIFT => self.toggle_preview_on_move(),
            KeyCode::Char('y') if modifiers.is_empty() => self.copy_selected_path(),
            KeyCode::Char('u') if modifiers.is_empty() => history_back(self),
            KeyCode::Char('i') if modifiers.is_empty() => history_forward(self),
            KeyCode::Char('d') if modifiers.is_empty() => start_delete_confirm(self),
            KeyCode::F(2) => self.start_rename(),
            KeyCode::Char('b') if modifiers.is_empty() => bookmark_cwd(self),
            KeyCode::Char(c @ '1'..='9') if modifiers.is_empty() => {
                goto_bookmark(self, (c as u8 - b'1') as usize);
            }
            _ => (),
        }
    }

    fn start_rename(&mut self) {
        let seed = self
            .selected_entry()
            .map(|e| e.name.clone())
            .unwrap_or_default();
        self.enter_input(InputKind::Rename, seed);
    }

    fn copy_selected_path(&mut self) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        if entry.is_parent_link {
            self.status = "Nothing to copy".to_string();
            return;
        }
        match clipboard::copy_path(&entry.path.0) {
            Ok(()) => self.status = format!("Copied · {}", entry.path.0.display()),
            Err(err) => self.status = format!("Clipboard: {err}"),
        }
    }
}
