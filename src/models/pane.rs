use crate::models::previewer::Previewer;
use ratatui::widgets::TableState;
use rtvui_core::{Artifact, ArtifactListResult};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Independent browser state for one tab (and optionally a split peer).
pub struct BrowserPane {
    pub current_working_directory: PathBuf,
    pub artifacts: Vec<Artifact>,
    pub scroll_state: TableState,
    pub filter_input: String,
    pub entries_cache: Vec<Artifact>,
    pub entries_filtered: Vec<Artifact>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub generation: u64,
    pub previewer: Previewer,
    pub previewer_generation: u64,
    pub listing_partial: bool,
    pub marked: HashSet<PathBuf>,
    /// Basename → porcelain status char (`M`, `A`, `D`, `?`, …).
    pub git_marks: HashMap<String, char>,
}

impl BrowserPane {
    pub fn from_listing(path: PathBuf, listing: ArtifactListResult, generation: u64) -> Self {
        let artifacts = listing.artifacts;
        let mut scroll_state = TableState::default();
        if !artifacts.is_empty() {
            scroll_state.select(Some(0));
        }
        Self {
            current_working_directory: path.clone(),
            artifacts: artifacts.clone(),
            scroll_state,
            filter_input: String::new(),
            entries_cache: artifacts.clone(),
            entries_filtered: artifacts,
            history: vec![path],
            history_index: 0,
            generation,
            previewer: Previewer::Empty,
            previewer_generation: 0,
            listing_partial: listing.partial,
            marked: HashSet::new(),
            git_marks: HashMap::new(),
        }
    }

    pub fn selected_artifact(&self) -> Option<&Artifact> {
        let selection = self.scroll_state.selected()?;
        self.entries_filtered.get(selection)
    }

    pub fn is_file_artifact(&self) -> bool {
        self.selected_artifact()
            .is_some_and(|artifact| artifact.artifact_type == rtvui_core::ArtifactType::File)
    }

    pub fn apply_filter(&mut self) {
        let needle = self.filter_input.to_lowercase();
        self.entries_filtered = self
            .entries_cache
            .iter()
            .filter(|artifact| artifact.name.to_lowercase().contains(&needle))
            .cloned()
            .collect();
        self.scroll_state
            .select(if self.entries_filtered.is_empty() {
                None
            } else {
                Some(0)
            });
    }

    pub fn apply_listing(&mut self, listing: ArtifactListResult) {
        self.artifacts = listing.artifacts.clone();
        self.entries_cache = listing.artifacts.clone();
        self.listing_partial = listing.partial;
        if self.filter_input.is_empty() {
            self.entries_filtered = listing.artifacts;
        } else {
            self.apply_filter();
            return;
        }
        if self.entries_filtered.is_empty() {
            self.scroll_state.select(None);
            return;
        }
        let selection = self.scroll_state.selected().unwrap_or(0);
        let clamped = selection.min(self.entries_filtered.len() - 1);
        self.scroll_state.select(Some(clamped));
    }

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

    pub fn toggle_mark_selected(&mut self) {
        let Some(artifact) = self.selected_artifact() else {
            return;
        };
        let path = artifact.path.clone();
        if !self.marked.remove(&path) {
            self.marked.insert(path);
        }
    }

    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    pub fn refresh_git_marks(&mut self) {
        self.git_marks = load_git_marks(&self.current_working_directory);
    }
}

/// Run `git status --porcelain` when `cwd` is inside a work tree.
pub fn load_git_marks(cwd: &Path) -> HashMap<String, char> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &cwd.display().to_string(),
            "status",
            "--porcelain",
            "-u",
        ])
        .output();
    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut marks = HashMap::new();
    for line in text.lines() {
        if line.len() < 4 {
            continue;
        }
        let status = line.chars().next().unwrap_or(' ');
        let status = if status == ' ' {
            line.chars().nth(1).unwrap_or('?')
        } else {
            status
        };
        let path_part = line[3..].trim();
        // Handle renames: `R  old -> new`
        let name = path_part
            .rsplit_once(" -> ")
            .map(|(_, new)| new)
            .unwrap_or(path_part);
        let basename = std::path::Path::new(name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_string();
        marks.insert(basename, if status == ' ' { '?' } else { status });
    }
    marks
}
