use ratatui::widgets::TableState;
use rtvui_core::utils::get_artifact_entries;
use rtvui_core::{Artifact, ArtifactOptions};
use std::io::Result;
use std::path::PathBuf;

use crate::config::app::{AppConfig, Themes};

pub struct App {
    pub current_working_directory: PathBuf,
    pub artifacts: Vec<Artifact>,
    pub scroll_state: TableState,

    pub status_message: String,
    pub state: AppState,
    pub filter_input: String,
    pub entries_cache: Vec<Artifact>,
    pub entries_filtered: Vec<Artifact>,

    pub config: AppConfig,
}

pub enum AppState {
    Active,
    Filter,
    GoTo,
    Confirm,
    Help,
    Quit,
}

impl App {
    pub fn new(path: PathBuf, config: AppConfig) -> Result<Self> {
        let current_working_directory = path;
        let result = get_artifact_entries(&current_working_directory, &ArtifactOptions::default())?;
        let mut scroll_state = TableState::default();
        let mut scroll_index = None;
        let artifacts = result.artifacts;

        if !artifacts.is_empty() {
            scroll_index = Some(0);
        }
        scroll_state.select(scroll_index);

        Ok(Self {
            current_working_directory,
            artifacts,
            scroll_state,
            status_message: String::new(),
            state: AppState::Active,
            filter_input: String::new(),
            entries_cache: Vec::new(),
            entries_filtered: Vec::new(),
            config,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let artifacts =
            get_artifact_entries(&self.current_working_directory, &ArtifactOptions::default())?;
        self.artifacts = artifacts.artifacts;

        let Some(selection) = self.scroll_state.selected() else {
            return Ok(());
        };

        if self.artifacts.is_empty() {
            self.scroll_state.select(None);
        } else {
            let min_selection = selection.min(self.artifacts.len() - 1);
            self.scroll_state.select(Some(min_selection));
        };
        Ok(())
    }

    pub fn filter(&mut self) -> Result<()> {
        self.entries_filtered = self
            .entries_cache
            .iter()
            .filter(|artifact| {
                artifact
                    .name
                    .to_lowercase()
                    .contains(&self.filter_input.to_lowercase())
            })
            .cloned()
            .collect();

        self.scroll_state.select(Some(0));
        Ok(())
    }

    pub fn update_theme(&mut self) {
        match self.config.theme {
            Themes::Forest => self.config.theme = Themes::Midnight,
            Themes::Midnight => self.config.theme = Themes::Solar,
            Themes::Solar => self.config.theme = Themes::Mono,
            Themes::Mono => self.config.theme = Themes::Forest,
        }
    }
}
