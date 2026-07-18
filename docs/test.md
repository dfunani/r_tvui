# Testing R-TVUI

This document explains how the test suite is organised, how to run it, how to
add new tests, and how code coverage works. It is the source of truth for
testing conventions in this repository.

## TL;DR

```bash
make test          # fmt + lint + run the whole suite (cargo test --workspace --all-targets)
cargo test --workspace   # just the tests
make coverage      # text coverage summary for the workspace
make coverage-html # HTML coverage report (opens in a browser)
```

## How testing is structured

R-TVUI is a Cargo workspace with two crates:

- `core` (`crates/core`) — pure file-system / artifact logic with no UI.
- `r_tvui` (root) — the application: models, events, config, UI rendering.

Tests live **inside each crate** as `#[cfg(test)]` modules (Rust's standard
unit-test location). They are compiled only for `cargo test`, so they add zero
weight to release builds. There are no separate `tests/` integration
directories; every test exercises the crate's own public API.

### Test module layout

```
crates/core/src/
  lib.rs                 # declares `mod tests { mod core; }` under #[cfg(test)]
  tests/core.rs          # all core-crate tests

src/
  lib.rs                 # declares `mod tests { ... }` under #[cfg(test)]
  tests/
    app.rs               # App state machine: reload, filter, rename, async events, sort, theme
    cli.rs               # start-path resolution
    config.rs            # config defaults, palette, save/load round-trip
    events.rs            # keymap dispatch, scrolling, navigation, filter/rename input
    previewer.rs         # file preview reading + truncation
    ui.rs                # headless rendering of the full TUI via ratatui TestBackend
```

Each file is wired into its crate through the `tests` module in `lib.rs`:

```rust
#[cfg(test)]
pub mod tests {
    pub mod app;
    pub mod cli;
    pub mod config;
    pub mod events;
    pub mod previewer;
    pub mod ui;
}
```

## Running tests

```bash
# Everything (both crates, all targets)
cargo test --workspace

# A single crate
cargo test -p core
cargo test -p r_tvui

# A single module or test by name (substring match)
cargo test --workspace filter
cargo test -p r_tvui commit_rename

# Show output (println!, dbg!) even for passing tests
cargo test --workspace -- --nocapture
```

`make test` runs `fmt` and `lint` (clippy) first, then the suite, so it is the
recommended pre-commit command.

## What the suite covers

The goal is to test **behaviour and intent**, not implementation details. The
suite is organised by responsibility:

| Area | File | Representative cases |
| --- | --- | --- |
| Directory listing | `core/tests/core.rs` | hidden-file filtering, `MAX_ENTRIES` truncation (`partial`), metadata (size/type/path), missing-path errors |
| Sorting | `core/tests/core.rs` | directories before files, name (asc), size & modified (desc), tie-breaking |
| Size formatting | `core/tests/core.rs` | B / KB / MB / GB boundaries |
| Async listing | `core/tests/core.rs` | `get_artifact_entries_async` matches sync result; errors propagate |
| Error mapping | `core/tests/core.rs` | `raise_filesystem_error` variant mapping + fallback |
| App lifecycle | `app.rs` | construction, empty dir, reload + selection clamping |
| Filtering | `app.rs` | substring, case-insensitive, empty matches all, no-match resets selection |
| Rename | `app.rs` | success, rejects path separators, rejects existing target, no-op when unchanged |
| Async events | `app.rs` | generation guards drop stale `BrowserDone`/`PreviewerDone` results |
| Config | `config.rs` | defaults match design, palette per theme, save→reload round-trip |
| Keymap | `events.rs` | mode transitions, scrolling clamps, directory navigation, filter/rename input editing |
| Preview | `previewer.rs` | reads small files, truncates to the 64 KiB window, errors on missing file |
| Rendering | `ui.rs` | full frame renders for every theme and previewer state without panicking |

### A note on side effects

Some code paths are deliberately **not** driven through tests because they have
real side effects on the developer's machine:

- **Persisting config** (`t`, `.`, `o` keys) calls `save_config(get_config_path())`,
  which writes to `~/.r_tvui/.config.toml`. Tests exercise the underlying
  methods (`update_theme`, `cycle_sort`, `show_hidden` toggle) and
  `save_config` against a **temp path** instead, so the real user config is
  never touched.
- **Opening files** (`Enter` on a file) spawns the OS `open`/`xdg-open` process.
  Tests only navigate into directories, never open files.

These constraints are why `src/os/mod.rs`, `src/main.rs`, and the blocking
`src/events/app.rs` run-loop show low coverage — they are I/O-/process-/TTY-bound
and are validated manually.

## Writing new tests

### 1. Pick the right home

Add the test to the module matching the behaviour (see the table above). If a
genuinely new area appears, create `src/tests/<area>.rs` and register it in
`src/lib.rs` under the `tests` module.

### 2. Use temp directories, keep the guard alive

`tempfile::tempdir()` returns a `TempDir` guard that **deletes the directory
when dropped**. Always bind it for the lifetime of the test:

```rust
fn app_with_files(files: &[(&str, &str)]) -> (tempfile::TempDir, App) {
    let dir = tempfile::tempdir().unwrap();
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    let mut app = App::new(dir.path().to_path_buf(), AppConfig::default()).unwrap();
    app.reload().unwrap();
    (dir, app) // returning `dir` keeps the directory alive
}
```

Return the `TempDir` (or bind it to `_dir`) so it is not dropped early.

### 3. Drive behaviour through the public API

- App behaviour: call `App` methods (`reload`, `filter`, `commit_rename`,
  `apply_async_event`, `cycle_sort`, …).
- Key handling: build a `crossterm::event::KeyEvent` and call the public
  dispatchers in `crate::events::keys` (`handle_key_events_normal_mode`, etc.).

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
assert_eq!(handle_key_events_normal_mode(&mut app, key).unwrap(), AppState::Filter);
```

### 4. Test async logic deterministically

Prefer feeding constructed events to `App::apply_async_event` over waiting on
real background tasks — this keeps tests fast and non-flaky:

```rust
app.apply_async_event(AsyncEvents::BrowserDone {
    generation: app.generation,        // matching generation => applied
    artifacts: listing,
    path: target,
});
```

For the core async wrapper itself, use `#[tokio::test]` (the `tokio` dev-dependency
in `crates/core/Cargo.toml` enables the runtime + macros):

```rust
#[tokio::test]
async fn async_listing_matches_sync() { /* ... */ }
```

### 5. Test rendering headlessly

UI tests render into an in-memory `ratatui::backend::TestBackend`, then read the
buffer back as text and assert on visible content:

```rust
let backend = TestBackend::new(80, 24);
let mut terminal = Terminal::new(backend).unwrap();
terminal.draw(|frame| render(frame, &mut app)).unwrap();
// iterate buffer cells -> String, then assert text.contains("...")
```

Note: the preview pane is narrow, so the file table's NAME column is squeezed
and long names get truncated. Assert on block **titles** or short names, or on
the wider left pane.

### What to add when you change code

- **New listing/sort/option** → add a `core` case asserting the new ordering or
  filtering, including an edge case (empty, over `MAX_ENTRIES`, missing path).
- **New key binding** → add an `events.rs` case for the resulting state
  transition or mutation.
- **New `App` method / state** → add an `app.rs` case for the happy path **and**
  at least one rejection / no-op path.
- **New rendered widget or mode** → add a `ui.rs` case that renders it and
  asserts a stable string is present.
- **Bug fix** → add a regression test that fails before the fix and passes
  after (e.g. the selection-clamp and generation-guard tests).

## Coverage

Coverage uses [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov),
which is based on LLVM's source-based coverage instrumentation.

### One-time setup

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov
```

### Running coverage

```bash
make coverage        # text summary (cargo llvm-cov --workspace --summary-only)
make coverage-html   # HTML report, opens in the browser

# Useful raw invocations:
cargo llvm-cov --workspace                 # summary + run tests
cargo llvm-cov --workspace --open          # HTML report
cargo llvm-cov --workspace --lcov --output-path lcov.info   # for CI / Codecov
```

### Reading the report

- **Regions / Lines** — percentage of executable regions/lines exercised.
- **Functions** — percentage of functions called at least once.
- A line at 0% was never executed; use the HTML report to see exactly which
  branches are missed.

### Current baseline

As of the latest run the workspace sits at roughly **76% line coverage**, with:

- `core` logic ~95–100% (the heart of the app),
- `models/app` ~86%, `ui/*` ~100%, `config/app` 100%,
- intentionally low: `events/app.rs` (blocking run loop), `main.rs`,
  `os/mod.rs` (process spawning) — see "A note on side effects" above.

Treat coverage as a guide, not a target: a meaningful assertion on real
behaviour is worth more than a line touched only to raise the number.

## Continuous integration

The suite is plain `cargo test`, so any CI runner can execute:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo llvm-cov --workspace --lcov --output-path lcov.info   # optional
```
