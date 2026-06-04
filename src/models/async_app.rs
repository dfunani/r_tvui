use std::path::PathBuf;

use rtvui_core::ArtifactListResult;
use rtvui_core::utils::get_artifact_entries_async;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

const CHANNEL_CAPACITY: usize = 64;

pub enum AsyncAppEvent {
    BrowserDone {
        artifacts: ArtifactListResult,
        path: PathBuf,
    },
}

pub struct AsyncApp {
    pub runtime: Runtime,
    pub sender: mpsc::Sender<AsyncAppEvent>,
    pub receiver: mpsc::Receiver<AsyncAppEvent>,
}

impl AsyncApp {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        Self {
            runtime: Runtime::new().unwrap(),
            sender,
            receiver,
        }
    }

    pub fn send(&self, path: PathBuf) {
        let sender = self.sender.clone();
        self.runtime.spawn(async move {
            let result = get_artifact_entries_async(&path, &Default::default()).await;
            let payload = AsyncAppEvent::BrowserDone {
                artifacts: result.unwrap(),
                path,
            };
            let _ = sender.send(payload).await;
        });
    }

    pub fn drain(&mut self) -> impl Iterator<Item = AsyncAppEvent> + '_ {
        std::iter::from_fn(|| self.receiver.try_recv().ok())
    }
}

impl Default for AsyncApp {
    fn default() -> Self {
        Self::new()
    }
}
