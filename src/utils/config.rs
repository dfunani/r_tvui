use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use filesystem::DirectorySortOrder;

use crate::theme::ThemeId;

/// User settings persisted under `~/.config/rtvui/config.toml` (or `$RTVUI_CONFIG`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: ThemeId,
    #[serde(default)]
    pub sort: SortPreference,
    #[serde(default = "default_true")]
    pub use_trash: bool,
    #[serde(default = "default_true")]
    pub preview_on_move: bool,
    #[serde(default)]
    pub bookmarks: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortPreference {
    Name,
    Size,
    Modified,
}

impl Default for SortPreference {
    fn default() -> Self {
        Self::Name
    }
}

impl From<SortPreference> for DirectorySortOrder {
    fn from(value: SortPreference) -> Self {
        match value {
            SortPreference::Name => DirectorySortOrder::Name,
            SortPreference::Size => DirectorySortOrder::Size,
            SortPreference::Modified => DirectorySortOrder::Modified,
        }
    }
}

impl SortPreference {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Size,
            Self::Size => Self::Modified,
            Self::Modified => Self::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Size => "size",
            Self::Modified => "mtime",
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeId::default(),
            sort: SortPreference::default(),
            use_trash: true,
            preview_on_move: true,
            bookmarks: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        config_path()
            .map(|path| Self::load_from(&path))
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "could not resolve config path")
        })?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, contents)
    }
}

pub fn config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("RTVUI_CONFIG") {
        return Some(PathBuf::from(path));
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("rtvui").join("config.toml"));
    }

    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("rtvui")
            .join("config.toml")
    })
}

pub fn save_theme(theme: ThemeId) -> io::Result<()> {
    let mut config = AppConfig::load();
    config.theme = theme;
    config.save()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rtvui-config-test-{nanos}.toml"))
    }

    #[test]
    fn config_roundtrip_theme() {
        let path = temp_config_path();
        let config = AppConfig {
            theme: ThemeId::Forest,
            ..AppConfig::default()
        };
        config.save_to(&path).unwrap();

        let loaded = AppConfig::load_from(&path);
        assert_eq!(loaded.theme, ThemeId::Forest);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn theme_slug_parse() {
        assert_eq!(ThemeId::from_slug("midnight"), Some(ThemeId::Midnight));
        assert_eq!(ThemeId::from_slug("unknown"), None);
    }

    #[test]
    fn sort_preference_cycles() {
        assert_eq!(SortPreference::Name.next(), SortPreference::Size);
    }
}
