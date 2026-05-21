use core::paths::AbsolutePath;
use std::time::Duration;
use std::sync::Arc;

use crate::directories::DirectoryListResult;
pub struct DirectoriesCache {
    max_entries: usize,          // default 32 directories
    max_age: Duration,             // default 2s unless mtime changed
}

impl DirectoriesCache {
    pub fn new(config: &CacheConfig) -> Self{
        return DirectoriesCache {
            max_entries: config.max_dirs,
            max_age: config.ttl,
        };
    }
    pub fn get(&self, path: &AbsolutePath) -> Option<Arc<DirectoryListResult>>{

    }
    pub fn insert(&mut self, result: Arc<ListResult>);
    pub fn invalidate(&mut self, path: &CanonicalPath);
    pub fn invalidate_tree_under(&mut self, path: &CanonicalPath);
}

pub struct CacheConfig {
    pub max_dirs: usize,
    pub ttl: Duration,
}