use std::path::Path;
use std::process::Command;

/// Open a path with the OS default application (non-blocking).
pub fn open_with_system_default(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path.as_os_str()).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path.as_os_str()).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy();
        Command::new("cmd")
            .args(["/C", "start", "", path_str.as_ref()])
            .spawn()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "system open is not supported on this platform",
        ));
    }

    Ok(())
}
