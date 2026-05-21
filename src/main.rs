use r_tvui::models::app::App;
use std::io::Result;

fn main() -> Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}
