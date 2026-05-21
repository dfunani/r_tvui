use std::path::PathBuf;

use clap::Parser;
use r_tvui::models::app::App;

#[derive(Parser, Debug)]
#[command(name = "r_tvui", about = "Terminal file explorer", version)]
struct Cli {
    /// Directory (or file) to open; ~ expanded; invalid paths use current directory
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    r_tvui::install_panic_hook();
    let cli = Cli::parse();
    ratatui::run(|terminal| App::new(cli.path).run(terminal))?;
    Ok(())
}
