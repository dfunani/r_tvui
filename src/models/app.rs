#[derive(PartialEq, Debug)]
pub struct App {
    pub counter: u32,
    pub state: AppState,
}

#[derive(PartialEq, Debug)]
pub enum AppState {
    Exit,
    Start,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Start
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            counter: 0,
            state: AppState::default(),
        }
    }
}
