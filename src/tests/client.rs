#[cfg(test)]
mod test_client {
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    use rtvui_core::{ArtifactOptions, ArtifactSort};

    use crate::models::client::{AsyncEventClient, AsyncEvents};
    use crate::models::previewer::Previewer;

    fn options() -> ArtifactOptions {
        ArtifactOptions { show_hidden: true, sort: ArtifactSort::Name }
    }

    /// Drain the client until an event arrives or the timeout elapses.
    fn wait_for_event(client: &mut AsyncEventClient) -> AsyncEvents {
        for _ in 0..200 {
            if let Some(event) = client.drain().next() {
                return event;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no async event was received within the timeout");
    }

    #[test]
    fn default_constructs_a_client() {
        let mut client = AsyncEventClient::default();
        assert!(client.drain().next().is_none());
    }

    #[test]
    fn send_emits_browser_done() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        let mut client = AsyncEventClient::new();

        client.send(dir.path().to_path_buf(), 7, options());

        match wait_for_event(&mut client) {
            AsyncEvents::BrowserDone { generation, artifacts, path } => {
                assert_eq!(generation, 7);
                assert_eq!(path, dir.path());
                assert_eq!(artifacts.artifacts.len(), 1);
            }
            _ => panic!("expected BrowserDone"),
        }
    }

    #[test]
    fn send_on_missing_path_yields_empty_listing() {
        let mut client = AsyncEventClient::new();
        client.send(PathBuf::from("/no/such/rtvui/dir"), 1, options());
        match wait_for_event(&mut client) {
            AsyncEvents::BrowserDone { artifacts, .. } => {
                assert!(artifacts.artifacts.is_empty());
            }
            _ => panic!("expected BrowserDone"),
        }
    }

    #[test]
    fn send_folder_preview_emits_folder() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("child.txt"), "x").unwrap();
        let mut client = AsyncEventClient::new();

        client.send_folder_preview(dir.path().to_path_buf(), "title".to_string(), 3, options());

        match wait_for_event(&mut client) {
            AsyncEvents::FolderPreviewDone { generation, previewer } => {
                assert_eq!(generation, 3);
                match previewer {
                    Previewer::Folder(folder) => {
                        assert_eq!(folder.title, "title");
                        assert_eq!(folder.artifacts.len(), 1);
                    }
                    _ => panic!("expected a folder previewer"),
                }
            }
            _ => panic!("expected FolderPreviewDone"),
        }
    }

    #[test]
    fn send_previewer_reads_file_body() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("note.txt");
        fs::write(&file, "preview body").unwrap();
        let mut client = AsyncEventClient::new();

        client.send_previewer(file, "note".to_string(), 5);

        match wait_for_event(&mut client) {
            AsyncEvents::PreviewerDone { generation, previewer } => {
                assert_eq!(generation, 5);
                match previewer {
                    Previewer::Preview(preview) => assert_eq!(preview.body, "preview body"),
                    _ => panic!("expected a text previewer"),
                }
            }
            _ => panic!("expected PreviewerDone"),
        }
    }

    #[test]
    fn send_previewer_falls_back_on_unreadable_file() {
        let mut client = AsyncEventClient::new();
        client.send_previewer(PathBuf::from("/no/such/rtvui/file.txt"), "missing".to_string(), 9);
        match wait_for_event(&mut client) {
            AsyncEvents::PreviewerDone { previewer, .. } => match previewer {
                Previewer::Preview(preview) => {
                    assert_eq!(preview.body, "Content Preview Unavailable");
                }
                _ => panic!("expected a text previewer"),
            },
            _ => panic!("expected PreviewerDone"),
        }
    }
}
