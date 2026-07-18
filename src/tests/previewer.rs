#[cfg(test)]
mod test_previewer {
    use std::fs;
    use std::path::PathBuf;

    use crate::models::previewer::read_text_prefix;

    #[test]
    fn reads_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.txt");
        fs::write(&path, "hello world").unwrap();
        let body = read_text_prefix(&path).unwrap();
        assert_eq!(body, "hello world");
    }

    #[test]
    fn truncates_large_file_to_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.txt");
        // 100 KiB of ASCII, larger than the 64 KiB preview window.
        fs::write(&path, vec![b'a'; 100 * 1024]).unwrap();
        let body = read_text_prefix(&path).unwrap();
        assert_eq!(body.len(), 64 * 1024);
    }

    #[test]
    fn errors_on_missing_file() {
        let result = read_text_prefix(&PathBuf::from("/no/such/rtvui/preview.txt"));
        assert!(result.is_err());
    }
}
