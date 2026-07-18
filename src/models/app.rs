use ratatui::widgets::TableState;
use rtvui_core::errors::RTVUIError;
use rtvui_core::utils::get_artifact_entries;
use rtvui_core::{Artifact, ArtifactListResult, ArtifactOptions, ArtifactSort, ArtifactType};
use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;

use crate::config::app::{AppConfig, Palette, Preview, Sort, Themes};
use crate::models::client::{AsyncEventClient, AsyncEvents};
use crate::models::previewer::Previewer;

pub struct App {
    pub current_working_directory: PathBuf,
    pub artifacts: Vec<Artifact>,
    pub scroll_state: TableState,

    pub status_message: String,
    pub state: AppState,
    pub filter_input: String,
    pub rename_input: String,
    pub entries_cache: Vec<Artifact>,
    pub entries_filtered: Vec<Artifact>,

    pub config: AppConfig,
    pub async_client: AsyncEventClient,
    pub generation: u64,
    pub previewer: Previewer,
    pub previewer_generation: u64,
    pub cache: HashMap<PathBuf, ArtifactListResult>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppState {
    #[default]
    Active,
    Filter,
    Rename,
    GoTo,
    Confirm,
    Help,
    Quit,
}

impl App {
    pub fn new(path: PathBuf, config: AppConfig) -> Result<Self> {
        let result = get_artifact_entries(&path, &Self::get_artifact_options(&config));
        let artifact_list = result.unwrap_or_else(|_| ArtifactListResult::default());
        let mut scroll_state = TableState::default();
        let mut scroll_index = None;
        let artifacts = artifact_list.artifacts;

        if !artifacts.is_empty() {
            scroll_index = Some(0);
        }
        scroll_state.select(scroll_index);
        let async_client = AsyncEventClient::new();
        let generation = 1;
        async_client.send(
            path.clone(),
            generation,
            Self::get_artifact_options(&config),
        );

        Ok(Self {
            current_working_directory: path,
            artifacts: artifacts.clone(),
            scroll_state,
            status_message: String::new(),
            state: AppState::Active,
            filter_input: String::new(),
            rename_input: String::new(),
            entries_cache: artifacts.clone(),
            entries_filtered: artifacts.clone(),
            config,
            async_client,
            generation,
            previewer: Previewer::Empty,
            previewer_generation: 0,
            cache: HashMap::new(),
        })
    }

    pub fn get_artifact_options(config: &AppConfig) -> ArtifactOptions {
        let sort = match config.settings.sort {
            Sort::Name => ArtifactSort::Name,
            Sort::Size => ArtifactSort::Size,
            Sort::Modified => ArtifactSort::Modified,
        };
        ArtifactOptions {
            show_hidden: config.settings.show_hidden,
            sort,
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        let result = get_artifact_entries(
            &self.current_working_directory,
            &Self::get_artifact_options(&self.config),
        );

        match result {
            Ok(artifacts) => {
                self.handle_reload(artifacts)?;
            }
            Err(e) => {
                self.handle_reload_error(e)?;
            }
        }

        Ok(())
    }

    pub fn async_reload(&mut self) -> Result<()> {
        self.generation += 1;
        if let Some(cached) = self.cache.get(&self.current_working_directory).cloned() {
            self.entries_cache = cached.artifacts.clone();
            self.handle_reload(cached).unwrap_or_default();
            self.request_previewer();
        }
        self.async_client.send(
            self.current_working_directory.clone(),
            self.generation,
            Self::get_artifact_options(&self.config),
        );
        Ok(())
    }

    pub fn request_previewer(&mut self) {
        self.previewer_generation += 1;
        let generation = self.previewer_generation;

        let Some(selection) = self.scroll_state.selected() else {
            self.previewer = Previewer::Empty;
            return;
        };
        let Some(artifact) = self.entries_filtered.get(selection) else {
            self.previewer = Previewer::Empty;
            return;
        };
        if self.config.settings.preview == Preview::Never {
            self.previewer = Previewer::Empty;
            return;
        }

        let path = artifact.path.clone();
        let title = artifact.name.clone();
        if artifact.artifact_type == ArtifactType::File {
            self.async_client.send_previewer(path, title, generation);
        } else {
            self.async_client.send_folder_preview(
                path,
                title,
                generation,
                Self::get_artifact_options(&self.config),
            );
        }
    }

    fn handle_reload(&mut self, artifacts: ArtifactListResult) -> Result<()> {
        self.artifacts = artifacts.artifacts;
        self.entries_filtered = self.artifacts.clone();

        if self.artifacts.is_empty() {
            self.scroll_state.select(None);
            return Ok(());
        }

        let selection = self.scroll_state.selected().unwrap_or(0);
        let clamped = selection.min(self.artifacts.len() - 1);
        self.scroll_state.select(Some(clamped));
        Ok(())
    }

    fn handle_reload_error(&mut self, error: RTVUIError) -> Result<()> {
        self.artifacts.clear();
        self.entries_cache.clear();
        self.entries_filtered.clear();
        self.status_message = format!("Error: {error:?}");
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
        self.request_previewer();
        Ok(())
    }

    pub fn cycle_sort(&mut self) {
        self.config.settings.sort = match self.config.settings.sort {
            Sort::Name => Sort::Size,
            Sort::Size => Sort::Modified,
            Sort::Modified => Sort::Name,
        };
    }

    /// Re-list the current directory after a change that invalidates cached
    /// listings (e.g. toggling hidden files or changing the sort order).
    pub fn refresh(&mut self) -> Result<()> {
        self.cache.clear();
        self.async_reload()
    }

    pub fn update_theme(&mut self) {
        match self.config.theme {
            Themes::Forest => self.config.theme = Themes::Midnight,
            Themes::Midnight => self.config.theme = Themes::Solar,
            Themes::Solar => self.config.theme = Themes::Mono,
            Themes::Mono => self.config.theme = Themes::Forest,
        }
    }

    pub fn palette(&self) -> Palette {
        self.config.theme.palette()
    }

    pub fn apply_async_event(&mut self, event: AsyncEvents) {
        match event {
            AsyncEvents::BrowserDone {
                generation,
                artifacts,
                path,
            } => {
                if generation != self.generation {
                    return;
                }
                self.current_working_directory = path.clone();
                self.entries_cache = artifacts.artifacts.clone();
                self.cache.insert(path, artifacts.clone());
                // clamp selection like handle_reload does
                self.handle_reload(artifacts).unwrap_or_default();
                self.request_previewer();
            }
            AsyncEvents::FolderPreviewDone {
                generation,
                previewer,
            } => {
                if generation != self.previewer_generation {
                    return;
                }
                self.previewer = previewer;
            }
            AsyncEvents::PreviewerDone {
                generation,
                previewer,
            } => {
                if generation != self.previewer_generation {
                    return;
                }
                self.previewer = previewer;
            }
        }
    }
    pub fn is_file_artifact(&self) -> bool {
        self.selected_artifact()
            .is_some_and(|artifact| artifact.artifact_type == ArtifactType::File)
    }

    pub fn selected_artifact(&self) -> Option<&Artifact> {
        let selection = self.scroll_state.selected()?;
        self.entries_filtered.get(selection)
    }

    /// Seed the rename buffer with the selected entry's name and enter rename mode.
    pub fn begin_rename(&mut self) -> AppState {
        match self.selected_artifact() {
            Some(artifact) => {
                self.rename_input = artifact.name.clone();
                AppState::Rename
            }
            None => AppState::Active,
        }
    }

    pub fn commit_rename(&mut self) -> Result<()> {
        let new_name = self.rename_input.trim().to_string();
        let Some(artifact) = self.selected_artifact() else {
            return Ok(());
        };
        let old_name = artifact.name.clone();
        let old_path = artifact.path.clone();

        if new_name.is_empty() || new_name == old_name {
            return Ok(());
        }

        if new_name.contains('/') || new_name.contains('\\') || new_name == ".." {
            self.status_message = "Rename failed: name cannot contain a path separator".to_string();
            return Ok(());
        }

        let new_path = self.current_working_directory.join(&new_name);
        if new_path.exists() {
            self.status_message = format!("Rename failed: {new_name} already exists");
            return Ok(());
        }

        match std::fs::rename(&old_path, &new_path) {
            Ok(()) => self.status_message = format!("Renamed to {new_name}"),
            Err(error) => self.status_message = format!("Rename failed: {error}"),
        }
        self.refresh()
    }
}
