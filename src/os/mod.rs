use std::io::Result;
use std::process::Command;
use std::{path::PathBuf, thread};

pub fn open_file(path: PathBuf) -> Result<()> {
    thread::spawn(move || {
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(path).spawn();

        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(path).spawn();

        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn();
    });
    Ok(())
}

/// Open a file with `$EDITOR` / `$VISUAL`, falling back to `vi`.
pub fn open_with_editor(path: PathBuf) -> Result<()> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(&editor).arg(&path).status().map_err(|error| {
        std::io::Error::other(format!("failed to launch editor `{editor}`: {error}"))
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "editor `{editor}` exited with {status}"
        )))
    }
}
