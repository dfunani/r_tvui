use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "r_tvui", version, about = "Terminal UI file explorer")]
pub struct Cli {
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}
