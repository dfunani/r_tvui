use ratatui::widgets::TableState;
use rtvui_core::errors::RTVUIError;
use rtvui_core::utils::get_artifact_entries;
use rtvui_core::{Artifact, ArtifactListResult, ArtifactOptions, ArtifactSort, ArtifactType};
use std::collections::HashMap;
use std::io::Result;
use std::path::PathBuf;

use crate::config::app::{AppConfig, Palette, Preview, Sort, Themes};
use crate::config::utils::{get_config_path, save_config};
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
    pub goto_input: String,
    pub entries_cache: Vec<Artifact>,
    pub entries_filtered: Vec<Artifact>,

    pub config: AppConfig,
    pub async_client: AsyncEventClient,
    pub generation: u64,
    pub previewer: Previewer,
    pub previewer_generation: u64,
    pub cache: HashMap<PathBuf, ArtifactListResult>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub listing_partial: bool,
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
        let start = path.canonicalize().unwrap_or(path);
        async_client.send(
            start.clone(),
            generation,
            Self::get_artifact_options(&config),
        );

        Ok(Self {
            current_working_directory: start.clone(),
            artifacts: artifacts.clone(),
            scroll_state,
            status_message: String::new(),
            state: AppState::Active,
            filter_input: String::new(),
            rename_input: String::new(),
            goto_input: String::new(),
            entries_cache: artifacts.clone(),
            entries_filtered: artifacts.clone(),
            config,
            async_client,
            generation,
            previewer: Previewer::Empty,
            previewer_generation: 0,
            cache: HashMap::new(),
            history: vec![start],
            history_index: 0,
            listing_partial: false,
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

    pub fn cycle_preview(&mut self) {
        self.config.settings.preview = match self.config.settings.preview {
            Preview::OnMove => Preview::Always,
            Preview::Always => Preview::Never,
            Preview::Never => Preview::OnMove,
        };
        save_config(&self.config, &get_config_path()).unwrap_or_default();
        self.status_message = format!("Preview: {:?}", self.config.settings.preview);
        self.request_previewer();
    }

    /// Record cwd in navigation history (truncates any forward entries).
    pub fn record_history(&mut self) {
        let cwd = self
            .current_working_directory
            .canonicalize()
            .unwrap_or_else(|_| self.current_working_directory.clone());
        if self.history.get(self.history_index) == Some(&cwd) {
            return;
        }
        self.history.truncate(self.history_index + 1);
        self.history.push(cwd);
        self.history_index = self.history.len() - 1;
    }

    pub fn history_back(&mut self) -> Result<()> {
        if self.history_index == 0 {
            self.status_message = "History: at oldest".to_string();
            return Ok(());
        }
        self.history_index -= 1;
        self.current_working_directory = self.history[self.history_index].clone();
        self.scroll_state.select(Some(0));
        self.status_message = "History: back".to_string();
        self.async_reload()
    }

    pub fn history_forward(&mut self) -> Result<()> {
        if self.history_index + 1 >= self.history.len() {
            self.status_message = "History: at newest".to_string();
            return Ok(());
        }
        self.history_index += 1;
        self.current_working_directory = self.history[self.history_index].clone();
        self.scroll_state.select(Some(0));
        self.status_message = "History: forward".to_string();
        self.async_reload()
    }

    /// Jump to the user's home directory.
    pub fn jump_home(&mut self) -> Result<()> {
        let Some(home) = dirs::home_dir() else {
            self.status_message = "Home directory unavailable".to_string();
            return Ok(());
        };
        self.current_working_directory = home.canonicalize().unwrap_or(home);
        self.scroll_state.select(Some(0));
        self.record_history();
        self.async_reload()
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
                error,
            } => {
                if generation != self.generation {
                    return;
                }
                self.current_working_directory = path.clone();
                self.entries_cache = artifacts.artifacts.clone();
                self.listing_partial = artifacts.partial;
                self.cache.insert(path, artifacts.clone());
                if let Some(message) = error {
                    self.status_message = format!("Error: {message}");
                } else if artifacts.partial {
                    self.status_message = "Listing truncated at 50,000 entries".to_string();
                }
                self.handle_reload(artifacts).unwrap_or_default();
                if self.config.settings.preview != Preview::Never {
                    self.request_previewer();
                }
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

    /// Clear the go-to buffer and enter path-jump mode.
    pub fn begin_goto(&mut self) -> AppState {
        self.goto_input.clear();
        AppState::GoTo
    }

    /// Resolve `goto_input` (supports `~`) and jump if it is an existing directory.
    pub fn commit_goto(&mut self) -> Result<AppState> {
        let raw = self.goto_input.trim();
        if raw.is_empty() {
            self.status_message = "Go to: enter a path".to_string();
            return Ok(AppState::GoTo);
        }

        let expanded = shellexpand::tilde(raw);
        let candidate = PathBuf::from(expanded.as_ref());
        let candidate = if candidate.is_absolute() {
            candidate
        } else {
            self.current_working_directory.join(candidate)
        };

        let path = match candidate.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                self.status_message = format!("Go to failed: {raw} not found");
                return Ok(AppState::GoTo);
            }
        };

        if !path.is_dir() {
            self.status_message = format!("Go to failed: {raw} is not a directory");
            return Ok(AppState::GoTo);
        }

        self.current_working_directory = path;
        self.goto_input.clear();
        self.scroll_state.select(Some(0));
        self.status_message.clear();
        self.record_history();
        self.async_reload()?;
        Ok(AppState::Active)
    }

    /// Bookmark the current directory into slots 1–9 (persisted).
    pub fn bookmark_cwd(&mut self) {
        let path = self
            .current_working_directory
            .canonicalize()
            .unwrap_or_else(|_| self.current_working_directory.clone())
            .display()
            .to_string();

        if let Some(index) = self
            .config
            .cache
            .bookmarks
            .iter()
            .position(|bookmark| bookmark == &path)
        {
            self.status_message = format!("Already bookmarked as {}", index + 1);
            return;
        }

        if self.config.cache.bookmarks.len() >= 9 {
            self.config.cache.bookmarks.remove(0);
        }
        self.config.cache.bookmarks.push(path);
        let slot = self.config.cache.bookmarks.len();
        save_config(&self.config, &get_config_path()).unwrap_or_default();
        self.status_message = format!("Bookmarked as {slot}");
    }

    /// Jump to bookmark slot `1..=9` if present and still a directory.
    pub fn jump_to_bookmark(&mut self, slot: usize) -> Result<()> {
        if !(1..=9).contains(&slot) {
            return Ok(());
        }
        let Some(path_str) = self.config.cache.bookmarks.get(slot - 1).cloned() else {
            self.status_message = format!("No bookmark in slot {slot}");
            return Ok(());
        };

        let candidate = PathBuf::from(&path_str);
        let path = match candidate.canonicalize() {
            Ok(path) if path.is_dir() => path,
            _ => {
                self.status_message = format!("Bookmark {slot} missing: {path_str}");
                return Ok(());
            }
        };

        self.current_working_directory = path;
        self.scroll_state.select(Some(0));
        self.status_message = format!("Jumped to bookmark {slot}");
        self.record_history();
        self.async_reload()
    }

    /// Enter confirm-delete when an entry is selected.
    pub fn begin_delete(&mut self) -> AppState {
        match self.selected_artifact() {
            Some(_) => AppState::Confirm,
            None => AppState::Active,
        }
    }

    /// Delete the selected entry (trash or permanent) and refresh.
    pub fn commit_delete(&mut self) -> Result<()> {
        let Some(artifact) = self.selected_artifact() else {
            return Ok(());
        };
        let name = artifact.name.clone();
        let path = artifact.path.clone();
        let is_dir = artifact.artifact_type == ArtifactType::Directory;
        let use_trash = self.config.settings.enable_trash;

        let result = if use_trash {
            trash::delete(&path).map_err(|error| std::io::Error::other(error.to_string()))
        } else if is_dir {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        match result {
            Ok(()) => {
                self.status_message = if use_trash {
                    format!("Moved {name} to trash")
                } else {
                    format!("Deleted {name}")
                };
            }
            Err(error) => {
                self.status_message = format!("Delete failed: {error}");
            }
        }
        self.refresh()
    }

    /// Copy the selected entry's absolute path to the system clipboard.
    pub fn copy_selected_path(&mut self) {
        let Some(artifact) = self.selected_artifact() else {
            self.status_message = "Copy failed: nothing selected".to_string();
            return;
        };
        let path = artifact.path.display().to_string();
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(path.clone())) {
            Ok(()) => self.status_message = format!("Copied {path}"),
            Err(error) => self.status_message = format!("Copy failed: {error}"),
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
