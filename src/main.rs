use clap::Parser;
use r_tvui::{
    cli::model::Cli,
    cli::utils::{get_start_path, global_exception_handler},
    config::app::AppConfig,
    config::utils::load_config,
    events::app::app_loop,
    models::app::App,
};
use std::io::Result;

fn main() -> Result<()> {
    global_exception_handler();
    let cli = Cli::parse();
    let path = get_start_path(cli.path)?;
    let config = load_config().unwrap_or_else(|_| AppConfig::default());
    let mut app = App::new(path, config)?;
    ratatui::run(|terminal| app_loop(terminal, &mut app))?;

    Ok(())
}
