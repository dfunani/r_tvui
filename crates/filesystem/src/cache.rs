use crate::directories::DirectoryListResult;
use rtvui_core::paths::AbsolutePath;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct CacheConfig {
    pub max_dirs: usize,
    pub ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_dirs: 32,
            ttl: Duration::from_secs(2),
        }
    }
}

pub struct DirectoriesCache {
    max_entries: usize,
    max_age: Duration,
    entries: HashMap<std::path::PathBuf, (Instant, Arc<DirectoryListResult>)>,
}

impl DirectoriesCache {
    pub fn new(config: &CacheConfig) -> Self {
        Self {
            max_entries: config.max_dirs,
            max_age: config.ttl,
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, path: &AbsolutePath) -> Option<Arc<DirectoryListResult>> {
        let (stored_at, result) = self.entries.get(&path.0)?;
        if stored_at.elapsed() > self.max_age {
            return None;
        }
        Some(Arc::clone(result))
    }

    pub fn insert(&mut self, result: Arc<DirectoryListResult>) {
        if self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, (instant, _))| *instant)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }
        self.entries
            .insert(result.path.0.clone(), (Instant::now(), result));
    }

    pub fn invalidate(&mut self, path: &AbsolutePath) {
        self.entries.remove(&path.0);
    }
}
