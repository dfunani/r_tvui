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
            .args(["/C", "start", "", path.display().to_string()])
            .spawn();
    });
    Ok(())
}
