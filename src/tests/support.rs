//! Shared helpers for the test suite.
#![cfg(test)]

use std::path::Path;
use std::sync::Mutex;

/// Serializes tests that mutate the process-global `HOME` environment variable
/// so they cannot race each other (or anything reading `get_config_path`).
static HOME_LOCK: Mutex<()> = Mutex::new(());

/// Run `body` with `HOME` pointed at a fresh temp directory, then restore the
/// previous value. This keeps config-persistence tests from ever touching the
/// developer's real `~/.r_tvui` directory.
pub fn with_temp_home<T>(body: impl FnOnce(&Path) -> T) -> T {
    let guard = HOME_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let previous = std::env::var_os("HOME");

    // SAFETY: access to HOME is serialized by HOME_LOCK for the duration of the
    // closure, and the original value is restored before the lock is released.
    unsafe { std::env::set_var("HOME", dir.path()) };

    let result = body(dir.path());

    unsafe {
        match previous {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }
    drop(guard);
    result
}
