#[cfg(test)]
mod tests;

pub mod events;
pub mod models;
pub mod theme;
pub mod utils;

pub mod ui;

/// Restore the terminal if the process panics while the alternate screen is active.
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = ratatui::restore();
        original(info);
    }));
}
