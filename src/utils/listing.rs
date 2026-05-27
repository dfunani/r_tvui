use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use filesystem::{
    CacheConfig, DirectoriesCache, DirectoryListOptions, DirectoryListResult, FileSystemError,
};
use rtvui_core::paths::{AbsolutePath, FileType};
use tokio::runtime::Runtime;

use crate::utils::previewer::preview_path;

#[derive(Debug)]
pub enum ListingEvent {
    Browser {
        generation: u64,
        result: Result<Arc<DirectoryListResult>, FileSystemError>,
    },
    SideFolder {
        generation: u64,
        path: AbsolutePath,
        result: Result<Arc<DirectoryListResult>, FileSystemError>,
    },
    SidePreview {
        generation: u64,
        title: String,
        body: String,
    },
}

pub struct ListingService {
    cache: Arc<Mutex<DirectoriesCache>>,
    updates: mpsc::Receiver<ListingEvent>,
    _sender: mpsc::Sender<ListingEvent>,
    runtime: Runtime,
}

impl Default for ListingService {
    fn default() -> Self {
        Self::new()
    }
}

impl ListingService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let cache = Arc::new(Mutex::new(DirectoriesCache::new(&CacheConfig::default())));
        let runtime = Runtime::new().expect("tokio runtime");
        Self {
            cache,
            updates: rx,
            _sender: tx.clone(),
            runtime,
        }
    }

    pub fn drain(&self) -> impl Iterator<Item = ListingEvent> + '_ {
        self.updates.try_iter()
    }

    pub fn invalidate(&self, path: &AbsolutePath) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.invalidate(path);
        }
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    pub fn request_browser(&self, generation: u64, path: AbsolutePath, opts: DirectoryListOptions) {
        self.spawn_directory_listing(generation, path, opts, ListingKind::Browser);
    }

    pub fn request_side_folder(
        &self,
        generation: u64,
        path: AbsolutePath,
        opts: DirectoryListOptions,
    ) {
        self.spawn_directory_listing(generation, path, opts, ListingKind::SideFolder);
    }

    pub fn request_side_preview(
        &self,
        generation: u64,
        title: String,
        path: PathBuf,
        kind: FileType,
    ) {
        let tx = self._sender.clone();
        thread::spawn(move || {
            let body = preview_path(&path, kind);
            let _ = tx.send(ListingEvent::SidePreview {
                generation,
                title,
                body,
            });
        });
    }

    fn spawn_directory_listing(
        &self,
        generation: u64,
        path: AbsolutePath,
        opts: DirectoryListOptions,
        kind: ListingKind,
    ) {
        let cache = Arc::clone(&self.cache);
        let tx = self._sender.clone();
        let path_for_cache = path.clone();

        self.runtime.spawn(async move {
            let result = {
                let cached = cache.lock().ok().and_then(|c| c.get(&path_for_cache));
                if let Some(hit) = cached {
                    Ok(hit)
                } else {
                    match filesystem::list_directories_async(path.clone(), opts).await {
                        Ok(listing) => {
                            let arc = Arc::new(listing);
                            if let Ok(mut c) = cache.lock() {
                                c.insert(Arc::clone(&arc));
                            }
                            Ok(arc)
                        }
                        Err(err) => Err(err),
                    }
                }
            };

            let event = match kind {
                ListingKind::Browser => ListingEvent::Browser { generation, result },
                ListingKind::SideFolder => ListingEvent::SideFolder {
                    generation,
                    path,
                    result,
                },
            };
            let _ = tx.send(event);
        });
    }
}

enum ListingKind {
    Browser,
    SideFolder,
}

impl std::fmt::Debug for ListingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListingService").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn cached_listing_returns_on_second_request() {
        let service = ListingService::new();
        let dir = std::env::temp_dir();
        let path = AbsolutePath(dir.clone());
        let opts = DirectoryListOptions::default();

        service.request_browser(1, path.clone(), opts.clone());
        let first = wait_for_browser(&service, 1);
        assert!(first.is_ok());

        service.request_browser(2, path, opts);
        let second = wait_for_browser(&service, 2);
        assert!(second.is_ok());
    }

    fn wait_for_browser(
        service: &ListingService,
        generation: u64,
    ) -> Result<Arc<DirectoryListResult>, FileSystemError> {
        for _ in 0..200 {
            for event in service.drain() {
                if let ListingEvent::Browser {
                    generation: event_gen,
                    result,
                } = event
                    && event_gen == generation
                {
                    return result;
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
        Err(FileSystemError::InvalidPath("timeout".into()))
    }
}
