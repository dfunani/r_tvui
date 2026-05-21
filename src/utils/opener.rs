use std::path::Path;
use std::process::{Command, Stdio};

/// Try to open a path with the OS default application.
/// Child stdout/stderr are discarded so macOS Launch Services errors cannot
/// corrupt the TUI. Returns `true` only when the open command reports success.
pub fn try_open_with_system_default(path: &Path) -> bool {
    open_with_system_default(path).is_ok()
}

fn open_with_system_default(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Wait for `open` so we know if Launch Services found an app (spawn alone lies).
        let status = Command::new("open")
            .arg(path.as_os_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no application for this file type",
            ))
        }
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path.as_os_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy();
        Command::new("cmd")
            .args(["/C", "start", "", path_str.as_ref()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "system open is not supported on this platform",
        ))
    }
}
