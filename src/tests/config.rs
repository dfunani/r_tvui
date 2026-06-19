#[cfg(test)]
mod test_config {
    use ratatui::style::Color;

    use crate::config::app::{AppConfig, Preview, Sort, Themes};
    use crate::config::utils::{get_config_path, save_config};

    // ---- defaults ----------------------------------------------------------

    #[test]
    fn defaults_match_design() {
        let config = AppConfig::default();
        assert_eq!(config.theme, Themes::Forest);
        assert_eq!(config.settings.sort, Sort::Name);
        assert_eq!(config.settings.preview, Preview::OnMove);
        assert!(config.settings.enable_trash);
        assert!(!config.settings.show_hidden);
        assert!(config.cache.bookmarks.is_empty());
    }

    #[test]
    fn enum_defaults() {
        assert_eq!(Themes::default(), Themes::Forest);
        assert_eq!(Sort::default(), Sort::Name);
        assert_eq!(Preview::default(), Preview::OnMove);
    }

    // ---- palette -----------------------------------------------------------

    #[test]
    fn each_theme_has_a_distinct_highlight() {
        assert_eq!(Themes::Forest.palette().highlight, Color::Green);
        assert_eq!(Themes::Midnight.palette().highlight, Color::Cyan);
        assert_eq!(Themes::Solar.palette().highlight, Color::Yellow);
        assert_eq!(Themes::Mono.palette().highlight, Color::White);
    }

    #[test]
    fn palette_fields_are_populated() {
        for theme in [
            Themes::Forest,
            Themes::Midnight,
            Themes::Solar,
            Themes::Mono,
        ] {
            let palette = theme.palette();
            // background uses Rgb; just confirm the accessor returns the struct.
            assert!(matches!(palette.background, Color::Rgb(_, _, _)));
            assert!(matches!(palette.text, Color::Rgb(_, _, _)));
        }
    }

    // ---- persistence round-trip -------------------------------------------

    #[test]
    fn save_then_reload_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut config = AppConfig {
            theme: Themes::Solar,
            ..Default::default()
        };
        config.settings.sort = Sort::Size;
        config.settings.show_hidden = true;
        config.cache.bookmarks = vec!["/tmp".to_string(), "/home".to_string()];

        save_config(&config, &path).unwrap();
        assert!(path.exists());

        let raw = std::fs::read_to_string(&path).unwrap();
        let loaded: AppConfig = toml::from_str(&raw).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn save_config_creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c/config.toml");
        save_config(&AppConfig::default(), &nested).unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn config_path_points_at_dotfile() {
        let path = get_config_path();
        assert!(path.to_string_lossy().ends_with(".config.toml"));
    }
}
