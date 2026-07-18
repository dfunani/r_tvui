use std::path::PathBuf;

use rtvui_core::utils::get_artifact_entries_async;
use rtvui_core::{ArtifactListResult, ArtifactOptions};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;

use crate::models::previewer::{FolderPreviewer, PreviewPreviewer, Previewer, read_preview_body};

const CHANNEL_CAPACITY: usize = 64;

pub enum AsyncEvents {
    BrowserDone {
        generation: u64,
        artifacts: ArtifactListResult,
        path: PathBuf,
        error: Option<String>,
    },
    FolderPreviewDone {
        generation: u64,
        previewer: Previewer,
    },
    PreviewerDone {
        generation: u64,
        previewer: Previewer,
    },
}

pub struct AsyncEventClient {
    pub runtime: Runtime,
    pub sender: mpsc::Sender<AsyncEvents>,
    pub receiver: mpsc::Receiver<AsyncEvents>,
}

impl AsyncEventClient {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        Self {
            runtime: Runtime::new().unwrap(),
            sender,
            receiver,
        }
    }

    pub fn send(&self, path: PathBuf, generation: u64, options: ArtifactOptions) {
        let sender = self.sender.clone();
        self.runtime.spawn(async move {
            let result = get_artifact_entries_async(path.clone(), options).await;
            let (artifacts, error) = match result {
                Ok(artifacts) => (artifacts, None),
                Err(error) => (ArtifactListResult::default(), Some(format!("{error:?}"))),
            };
            let payload = AsyncEvents::BrowserDone {
                generation,
                artifacts,
                path,
                error,
            };
            let _ = sender.send(payload).await;
        });
    }

    pub fn send_folder_preview(
        &self,
        path: PathBuf,
        title: String,
        generation: u64,
        options: ArtifactOptions,
    ) {
        let sender = self.sender.clone();
        self.runtime.spawn(async move {
            let result = get_artifact_entries_async(path, options).await;
            let artifacts = result.unwrap_or_default().artifacts;
            let previewer = Previewer::Folder(FolderPreviewer { title, artifacts });
            let payload = AsyncEvents::FolderPreviewDone {
                generation,
                previewer,
            };
            let _ = sender.send(payload).await;
        });
    }

    pub fn send_previewer(&self, path: PathBuf, title: String, generation: u64) {
        let sender = self.sender.clone();
        self.runtime.spawn(async move {
            let body = spawn_blocking(move || read_preview_body(&path))
                .await
                .ok()
                .and_then(|result| result.ok())
                .unwrap_or_else(|| "Content Preview Unavailable".to_string());
            let previewer = Previewer::Preview(PreviewPreviewer { title, body });
            let payload = AsyncEvents::PreviewerDone {
                generation,
                previewer,
            };
            let _ = sender.send(payload).await;
        });
    }

    pub fn drain(&mut self) -> impl Iterator<Item = AsyncEvents> + '_ {
        std::iter::from_fn(|| self.receiver.try_recv().ok())
    }
}

impl Default for AsyncEventClient {
    fn default() -> Self {
        Self::new()
    }
}
