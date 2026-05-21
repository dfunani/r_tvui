use std::path::Path;

pub fn copy_path(path: &Path) -> Result<(), String> {
    let text = path.display().to_string();
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(text)
        .map_err(|e| e.to_string())
}
