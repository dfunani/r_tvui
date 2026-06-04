use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub theme: Themes,
    pub settings: Settings,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub sort: Sort,
    pub enable_trash: bool,
    pub preview: Preview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    pub bookmarks: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Themes::Forest,
            cache: CacheConfig { bookmarks: vec![] },
            settings: Settings::default(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sort: Sort::Name,
            enable_trash: true,
            preview: Preview::OnMove,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Themes {
    #[default]
    Forest,
    Midnight,
    Solar,
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Sort {
    #[default]
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Preview {
    #[default]
    OnMove,
    Always,
    Never,
}
