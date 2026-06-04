use clap::Parser;
use config::{Config, ConfigError, File};
use r_tvui::{config::app::AppConfig, events::app::app_loop, models::app::App};
use std::{
    env::current_dir,
    error::Error,
    fs::{self, OpenOptions},
    io::{Result as IoResult, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(name = "r_tvui", version, about = "Terminal UI file explorer")]
struct Cli {
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() -> IoResult<()> {
    global_exception_handler();
    let cli = Cli::parse();
    let path = get_start_path(cli.path)?;
    let config = load_config().unwrap_or_else(|_| AppConfig::default());
    let mut app = App::new(path, config)?;
    ratatui::run(|terminal| app_loop(terminal, &mut app))?;

    Ok(())
}

fn global_exception_handler() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}

fn get_start_path(path: Option<PathBuf>) -> IoResult<PathBuf> {
    let start_path = path.unwrap_or_else(|| current_dir().unwrap());

    if start_path.is_dir() {
        Ok(start_path.canonicalize().unwrap_or(start_path))
    } else {
        std::env::current_dir()
    }
}

fn load_config() -> Result<AppConfig, ConfigError> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".r_tvui/.config.toml"))
        .unwrap_or_else(|| PathBuf::from(".config.toml"));

    if !config_path.exists() {
        let config = AppConfig::default();
        save_config(&config, &config_path).unwrap();
        return Ok(config);
    }

    let builder = Config::builder()
        .add_source(File::from(config_path))
        .build()?;
    let app_config = builder.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}

fn save_config(app_config: &AppConfig, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string_pretty(app_config)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut toml_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    toml_file.write_all(content.as_bytes())?;
    Ok(())
}
