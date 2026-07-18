use crate::config::app::AppConfig;
use config::{Config, ConfigError, File};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

pub fn get_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".r_tvui/.config.toml"))
        .unwrap_or_else(|| PathBuf::from(".config.toml"))
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let config_path = get_config_path();

    if !config_path.exists() {
        let config = AppConfig::default();
        save_config(&config, &config_path).unwrap_or_default();
        return Ok(config);
    }

    let builder = Config::builder()
        .add_source(File::from(config_path))
        .build()?;
    let app_config = builder.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}

pub fn save_config(app_config: &AppConfig, path: &PathBuf) -> Result<(), Box<dyn Error>> {
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
