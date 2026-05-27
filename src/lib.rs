#[cfg(test)]
mod tests;

pub mod events;
pub mod models;
pub mod theme;
pub mod utils;

pub mod ui;

pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}
