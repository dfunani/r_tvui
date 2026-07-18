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
use crate::models::pane::BrowserPane;
use crate::models::previewer::Previewer;

const MAX_TABS: usize = 9;

pub struct App {
    pub tabs: Vec<BrowserPane>,
    pub active_tab: usize,
    /// When set, UI shows `active_tab` | `split_tab` side by side.
    pub split_tab: Option<usize>,
    pub status_message: String,
    pub state: AppState,
    pub rename_input: String,
    pub goto_input: String,
    pub config: AppConfig,
    pub async_client: AsyncEventClient,
    pub cache: HashMap<PathBuf, ArtifactListResult>,
    /// File queued for `$EDITOR`; the event loop suspends the terminal,
    /// runs the editor, and restores the TUI.
    pub pending_editor: Option<PathBuf>,
    /// Globally unique listing-request counter (never reused across tabs, so
    /// async results can only ever match the pane that issued them).
    next_generation: u64,
    /// Globally unique previewer-request counter (same uniqueness contract).
    next_previewer_generation: u64,
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
        let async_client = AsyncEventClient::new();
        let generation = 1;
        let start = path.canonicalize().unwrap_or(path);
        async_client.send(
            start.clone(),
            generation,
            Self::get_artifact_options(&config),
        );

        let mut pane = BrowserPane::from_listing(start, artifact_list, generation);
        pane.refresh_git_marks();

        Ok(Self {
            tabs: vec![pane],
            active_tab: 0,
            split_tab: None,
            status_message: String::new(),
            state: AppState::Active,
            rename_input: String::new(),
            goto_input: String::new(),
            config,
            async_client,
            cache: HashMap::new(),
            pending_editor: None,
            next_generation: generation,
            next_previewer_generation: 0,
        })
    }

    pub fn pane(&self) -> &BrowserPane {
        &self.tabs[self.active_tab]
    }

    pub fn pane_mut(&mut self) -> &mut BrowserPane {
        &mut self.tabs[self.active_tab]
    }

    // --- Compatibility accessors used across UI / events / tests ---

    pub fn current_working_directory(&self) -> &PathBuf {
        &self.pane().current_working_directory
    }

    pub fn current_working_directory_mut(&mut self) -> &mut PathBuf {
        &mut self.pane_mut().current_working_directory
    }

    pub fn scroll_state(&self) -> &TableState {
        &self.pane().scroll_state
    }

    pub fn scroll_state_mut(&mut self) -> &mut TableState {
        &mut self.pane_mut().scroll_state
    }

    pub fn entries_filtered(&self) -> &[Artifact] {
        &self.pane().entries_filtered
    }

    pub fn entries_cache(&self) -> &[Artifact] {
        &self.pane().entries_cache
    }

    pub fn filter_input(&self) -> &str {
        &self.pane().filter_input
    }

    pub fn filter_input_mut(&mut self) -> &mut String {
        &mut self.pane_mut().filter_input
    }

    pub fn listing_partial(&self) -> bool {
        self.pane().listing_partial
    }

    pub fn previewer(&self) -> &Previewer {
        &self.pane().previewer
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
        let path = self.pane().current_working_directory.clone();
        let options = Self::get_artifact_options(&self.config);
        match get_artifact_entries(&path, &options) {
            Ok(artifacts) => self.handle_reload(artifacts)?,
            Err(e) => self.handle_reload_error(e)?,
        }
        Ok(())
    }

    pub fn async_reload(&mut self) -> Result<()> {
        let path = self.pane().current_working_directory.clone();
        self.next_generation += 1;
        let generation = self.next_generation;
        self.pane_mut().generation = generation;
        if let Some(cached) = self.cache.get(&path).cloned() {
            self.handle_reload(cached).unwrap_or_default();
            self.request_previewer();
        }
        let options = Self::get_artifact_options(&self.config);
        self.async_client.send(path, generation, options);
        Ok(())
    }

    pub fn request_previewer(&mut self) {
        let preview = self.config.settings.preview;
        let options = Self::get_artifact_options(&self.config);
        self.next_previewer_generation += 1;
        let generation = self.next_previewer_generation;
        let pane = self.pane_mut();
        pane.previewer_generation = generation;

        let Some(artifact) = pane.selected_artifact().cloned() else {
            pane.previewer = Previewer::Empty;
            return;
        };
        if preview == Preview::Never {
            pane.previewer = Previewer::Empty;
            return;
        }

        let path = artifact.path.clone();
        let title = artifact.name.clone();
        let is_file = artifact.artifact_type == ArtifactType::File;
        if is_file {
            self.async_client.send_previewer(path, title, generation);
        } else {
            self.async_client
                .send_folder_preview(path, title, generation, options);
        }
    }

    fn handle_reload(&mut self, artifacts: ArtifactListResult) -> Result<()> {
        self.pane_mut().apply_listing(artifacts);
        self.pane_mut().refresh_git_marks();
        Ok(())
    }

    fn handle_reload_error(&mut self, error: RTVUIError) -> Result<()> {
        let pane = self.pane_mut();
        pane.artifacts.clear();
        pane.entries_cache.clear();
        pane.entries_filtered.clear();
        self.status_message = format!("Error: {error:?}");
        Ok(())
    }

    pub fn filter(&mut self) -> Result<()> {
        self.pane_mut().apply_filter();
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

    pub fn record_history(&mut self) {
        self.pane_mut().record_history();
    }

    pub fn history_back(&mut self) -> Result<()> {
        let pane = self.pane_mut();
        if pane.history_index == 0 {
            self.status_message = "History: at oldest".to_string();
            return Ok(());
        }
        pane.history_index -= 1;
        pane.current_working_directory = pane.history[pane.history_index].clone();
        pane.scroll_state.select(Some(0));
        self.status_message = "History: back".to_string();
        self.async_reload()
    }

    pub fn history_forward(&mut self) -> Result<()> {
        let pane = self.pane_mut();
        if pane.history_index + 1 >= pane.history.len() {
            self.status_message = "History: at newest".to_string();
            return Ok(());
        }
        pane.history_index += 1;
        pane.current_working_directory = pane.history[pane.history_index].clone();
        pane.scroll_state.select(Some(0));
        self.status_message = "History: forward".to_string();
        self.async_reload()
    }

    pub fn jump_home(&mut self) -> Result<()> {
        let Some(home) = dirs::home_dir() else {
            self.status_message = "Home directory unavailable".to_string();
            return Ok(());
        };
        let pane = self.pane_mut();
        pane.current_working_directory = home.canonicalize().unwrap_or(home);
        pane.scroll_state.select(Some(0));
        pane.record_history();
        self.async_reload()
    }

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
                let Some(index) = self
                    .tabs
                    .iter()
                    .position(|pane| pane.generation == generation)
                else {
                    return;
                };
                let pane = &mut self.tabs[index];
                pane.current_working_directory = path.clone();
                pane.entries_cache = artifacts.artifacts.clone();
                pane.listing_partial = artifacts.partial;
                self.cache.insert(path, artifacts.clone());
                if let Some(message) = error {
                    self.status_message = format!("Error: {message}");
                } else if artifacts.partial {
                    self.status_message = "Listing truncated at 50,000 entries".to_string();
                }
                pane.apply_listing(artifacts);
                pane.refresh_git_marks();
                if index == self.active_tab && self.config.settings.preview != Preview::Never {
                    self.request_previewer();
                }
            }
            AsyncEvents::FolderPreviewDone {
                generation,
                previewer,
            }
            | AsyncEvents::PreviewerDone {
                generation,
                previewer,
            } => {
                if let Some(pane) = self
                    .tabs
                    .iter_mut()
                    .find(|pane| pane.previewer_generation == generation)
                {
                    pane.previewer = previewer;
                }
            }
        }
    }

    pub fn is_file_artifact(&self) -> bool {
        self.pane().is_file_artifact()
    }

    pub fn selected_artifact(&self) -> Option<&Artifact> {
        self.pane().selected_artifact()
    }

    pub fn begin_rename(&mut self) -> AppState {
        match self.selected_artifact() {
            Some(artifact) => {
                self.rename_input = artifact.name.clone();
                AppState::Rename
            }
            None => AppState::Active,
        }
    }

    pub fn begin_goto(&mut self) -> AppState {
        self.goto_input.clear();
        AppState::GoTo
    }

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
            self.pane().current_working_directory.join(candidate)
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

        let pane = self.pane_mut();
        pane.current_working_directory = path;
        pane.scroll_state.select(Some(0));
        pane.record_history();
        self.goto_input.clear();
        self.status_message.clear();
        self.async_reload()?;
        Ok(AppState::Active)
    }

    pub fn bookmark_cwd(&mut self) {
        let path = self
            .pane()
            .current_working_directory
            .canonicalize()
            .unwrap_or_else(|_| self.pane().current_working_directory.clone())
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

        let pane = self.pane_mut();
        pane.current_working_directory = path;
        pane.scroll_state.select(Some(0));
        pane.record_history();
        self.status_message = format!("Jumped to bookmark {slot}");
        self.async_reload()
    }

    pub fn begin_delete(&mut self) -> AppState {
        if !self.pane().marked.is_empty() || self.selected_artifact().is_some() {
            AppState::Confirm
        } else {
            AppState::Active
        }
    }

    pub fn commit_delete(&mut self) -> Result<()> {
        let use_trash = self.config.settings.enable_trash;
        let targets: Vec<(PathBuf, String, bool)> = {
            let pane = self.pane();
            if !pane.marked.is_empty() {
                pane.marked
                    .iter()
                    .map(|path| {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string());
                        let is_dir = path.is_dir();
                        (path.clone(), name, is_dir)
                    })
                    .collect()
            } else if let Some(artifact) = pane.selected_artifact() {
                vec![(
                    artifact.path.clone(),
                    artifact.name.clone(),
                    artifact.artifact_type == ArtifactType::Directory,
                )]
            } else {
                Vec::new()
            }
        };

        if targets.is_empty() {
            return Ok(());
        }

        let mut ok = 0usize;
        let mut fail = 0usize;
        for (path, _name, is_dir) in &targets {
            let result = if use_trash {
                trash::delete(path).map_err(|error| std::io::Error::other(error.to_string()))
            } else if *is_dir {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            };
            match result {
                Ok(()) => ok += 1,
                Err(_) => fail += 1,
            }
        }

        self.pane_mut().clear_marks();
        self.status_message = if fail == 0 {
            if use_trash {
                format!("Moved {ok} item(s) to trash")
            } else {
                format!("Deleted {ok} item(s)")
            }
        } else {
            format!("Deleted {ok}, failed {fail}")
        };
        self.refresh()
    }

    pub fn copy_selected_path(&mut self) {
        let pane = self.pane();
        let text = if !pane.marked.is_empty() {
            let mut paths: Vec<_> = pane
                .marked
                .iter()
                .map(|path| path.display().to_string())
                .collect();
            paths.sort();
            paths.join("\n")
        } else if let Some(artifact) = pane.selected_artifact() {
            artifact.path.display().to_string()
        } else {
            self.status_message = "Copy failed: nothing selected".to_string();
            return;
        };
        let preview = text.lines().next().unwrap_or("").to_string();
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text.clone())) {
            Ok(()) => {
                let count = text.lines().count();
                self.status_message = if count > 1 {
                    format!("Copied {count} paths")
                } else {
                    format!("Copied {preview}")
                };
            }
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
        let cwd = self.pane().current_working_directory.clone();

        if new_name.is_empty() || new_name == old_name {
            return Ok(());
        }

        if new_name.contains('/') || new_name.contains('\\') || new_name == ".." {
            self.status_message = "Rename failed: name cannot contain a path separator".to_string();
            return Ok(());
        }

        let new_path = cwd.join(&new_name);
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

    pub fn toggle_mark(&mut self) {
        self.pane_mut().toggle_mark_selected();
        let count = self.pane().marked.len();
        self.status_message = format!("Marked: {count}");
    }

    pub fn clear_marks(&mut self) {
        self.pane_mut().clear_marks();
        self.status_message = "Marks cleared".to_string();
    }

    pub fn new_tab(&mut self) -> Result<()> {
        if self.tabs.len() >= MAX_TABS {
            self.status_message = format!("Tab limit ({MAX_TABS}) reached");
            return Ok(());
        }
        let path = self.pane().current_working_directory.clone();
        let listing = self
            .cache
            .get(&path)
            .cloned()
            .unwrap_or_else(|| ArtifactListResult {
                artifacts: self.pane().entries_cache.clone(),
                partial: self.pane().listing_partial,
            });
        self.next_generation += 1;
        let mut pane = BrowserPane::from_listing(path.clone(), listing, self.next_generation);
        pane.refresh_git_marks();
        self.tabs.push(pane);
        self.active_tab = self.tabs.len() - 1;
        self.status_message = format!("Tab {} / {}", self.active_tab + 1, self.tabs.len());
        self.async_reload()
    }

    pub fn close_tab(&mut self) -> Result<()> {
        if self.tabs.len() == 1 {
            self.status_message = "Cannot close the last tab".to_string();
            return Ok(());
        }
        let closed = self.active_tab;
        self.tabs.remove(closed);
        if let Some(split) = self.split_tab {
            if split == closed {
                self.split_tab = None;
            } else if split > closed {
                self.split_tab = Some(split - 1);
            }
        }
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        // Split needs two distinct visible tabs; drop it if the peer now
        // aliases the active tab or only one tab remains.
        if self.tabs.len() < 2 || self.split_tab == Some(self.active_tab) {
            self.split_tab = None;
        }
        self.status_message = format!("Tab {} / {}", self.active_tab + 1, self.tabs.len());
        Ok(())
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        self.status_message = format!("Tab {} / {}", self.active_tab + 1, self.tabs.len());
        self.request_previewer();
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() < 2 {
            return;
        }
        self.active_tab = if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
        self.status_message = format!("Tab {} / {}", self.active_tab + 1, self.tabs.len());
        self.request_previewer();
    }

    /// Toggle dual-cwd split: shows the next tab (creating one if needed) beside the active tab.
    pub fn toggle_split(&mut self) -> Result<()> {
        if self.split_tab.is_some() {
            self.split_tab = None;
            self.status_message = "Split off".to_string();
            return Ok(());
        }
        if self.tabs.len() < 2 {
            self.new_tab()?;
        }
        let peer = (self.active_tab + 1) % self.tabs.len();
        if peer == self.active_tab {
            self.status_message = "Need a second tab to split".to_string();
            return Ok(());
        }
        self.split_tab = Some(peer);
        self.status_message = "Split on · Tab focuses panes".to_string();
        Ok(())
    }

    /// When split, swap focus between the two visible tabs; otherwise cycle tabs.
    pub fn focus_other_pane(&mut self) {
        if let Some(peer) = self.split_tab {
            let current = self.active_tab;
            self.active_tab = peer;
            self.split_tab = Some(current);
            self.status_message = format!("Focus tab {}", self.active_tab + 1);
            self.request_previewer();
        } else {
            self.next_tab();
        }
    }

    /// Copy marked (or selected) entries into the other split pane's directory.
    pub fn copy_to_other_pane(&mut self) -> Result<()> {
        let Some(peer) = self.split_tab else {
            self.status_message = "Split required for pane copy (\\)".to_string();
            return Ok(());
        };
        let dest = self.tabs[peer].current_working_directory.clone();
        let sources = self.collect_transfer_sources();
        if sources.is_empty() {
            self.status_message = "Nothing to copy".to_string();
            return Ok(());
        }
        let mut ok = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;
        for src in sources {
            let name = src.file_name().unwrap_or_default();
            let dest_path = dest.join(name);
            if dest_path.exists() {
                skipped += 1;
                continue;
            }
            let result = if src.is_dir() {
                copy_dir_all(&src, &dest_path)
            } else {
                std::fs::copy(&src, &dest_path).map(|_| ())
            };
            match result {
                Ok(()) => ok += 1,
                Err(_) => failed += 1,
            }
        }
        self.status_message = transfer_status("Copied", ok, skipped, failed);
        // Refresh peer listing
        let saved = self.active_tab;
        self.active_tab = peer;
        self.refresh()?;
        self.active_tab = saved;
        Ok(())
    }

    /// Move marked (or selected) entries into the other split pane's directory.
    pub fn move_to_other_pane(&mut self) -> Result<()> {
        let Some(peer) = self.split_tab else {
            self.status_message = "Split required for pane move (\\)".to_string();
            return Ok(());
        };
        let dest = self.tabs[peer].current_working_directory.clone();
        let sources = self.collect_transfer_sources();
        if sources.is_empty() {
            self.status_message = "Nothing to move".to_string();
            return Ok(());
        }
        let mut ok = 0usize;
        let mut skipped = 0usize;
        let mut failed = 0usize;
        for src in sources {
            let name = src.file_name().unwrap_or_default();
            let dest_path = dest.join(name);
            if dest_path.exists() {
                skipped += 1;
                continue;
            }
            match move_path(&src, &dest_path) {
                Ok(()) => ok += 1,
                Err(_) => failed += 1,
            }
        }
        self.pane_mut().clear_marks();
        self.status_message = transfer_status("Moved", ok, skipped, failed);
        self.refresh()?;
        let saved = self.active_tab;
        self.active_tab = peer;
        self.refresh()?;
        self.active_tab = saved;
        Ok(())
    }

    fn collect_transfer_sources(&self) -> Vec<PathBuf> {
        let pane = self.pane();
        if !pane.marked.is_empty() {
            pane.marked.iter().cloned().collect()
        } else if let Some(artifact) = pane.selected_artifact() {
            vec![artifact.path.clone()]
        } else {
            Vec::new()
        }
    }

    /// Queue the selected file for `$EDITOR`. The event loop performs the
    /// actual launch because it owns the terminal (raw mode must be suspended
    /// around a terminal editor).
    pub fn open_in_editor(&mut self) {
        let Some(artifact) = self.selected_artifact() else {
            self.status_message = "Editor: nothing selected".to_string();
            return;
        };
        if artifact.artifact_type != ArtifactType::File {
            self.status_message = "Editor: select a file".to_string();
            return;
        }
        self.pending_editor = Some(artifact.path.clone());
    }
}

fn transfer_status(verb: &str, ok: usize, skipped: usize, failed: usize) -> String {
    let mut message = format!("{verb} {ok} item(s) to other pane");
    if skipped > 0 {
        message.push_str(&format!(" · {skipped} skipped (exists)"));
    }
    if failed > 0 {
        message.push_str(&format!(" · {failed} failed"));
    }
    message
}

/// Rename, falling back to copy + delete when crossing filesystems.
fn move_path(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    if src.is_dir() {
        copy_dir_all(src, dst)?;
        std::fs::remove_dir_all(src)
    } else {
        std::fs::copy(src, dst)?;
        std::fs::remove_file(src)
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}
