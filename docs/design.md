# R-TVUI — Architecture & design

**Version:** 3.1.0 (`Cargo.toml`)

This document describes how R-TVUI is structured, how data flows at runtime, and the conventions contributors should follow when extending the codebase.

For end-user install and keybindings, see the [README](../README.md). For day-to-day development workflow, see [CONTRIBUTING.md](../CONTRIBUTING.md). For testing conventions, see [test.md](./test.md).

---

## Table of contents

1. [Overview](#1-overview)
2. [Goals and non-goals](#2-goals-and-non-goals)
3. [Workspace layout](#3-workspace-layout)
4. [Architecture](#4-architecture)
5. [State model](#5-state-model)
6. [Event loop and async I/O](#6-event-loop-and-async-i-o)
7. [UI layer](#7-ui-layer)
8. [File operations](#8-file-operations)
9. [Preview and side pane](#9-preview-and-side-pane)
10. [Configuration](#10-configuration)
11. [UX contracts](#11-ux-contracts)
12. [Security model](#12-security-model)
13. [Testing and CI](#13-testing-and-ci)
14. [Distribution](#14-distribution)
15. [Known limitations](#15-known-limitations)
16. [Future directions](#16-future-directions)
17. [Glossary](#17-glossary)

---

## 1. Overview

R-TVUI is a keyboard-driven terminal file explorer written in Rust. It presents a split view: a file list (with optional git status and marks) and a side pane for folder listings, text previews, or image metadata.

The application is intentionally small: one binary crate (`r_tvui`) plus one library crate (`crates/core`) for filesystem listing. There is no plugin system today.

---

## 2. Goals and non-goals

### Goals

- **Responsive UI** — directory reads and previews never block the main loop; system open runs in a background thread.
- **Keyboard-first** — every action has a default binding; common settings persist to config.
- **Safe file ops** — destructive actions require confirmation; trash is optional via config.
- **Single deployable binary** — no runtime dependencies beyond the OS (e.g. `open`, `xdg-open`, `$EDITOR`).

### Non-goals (current release)

- Dynamic plugins or previewer extensions
- Remote filesystems (SSH/SFTP)
- Terminal image protocols (kitty/sixel) — image preview is metadata-only
- Path sandboxing / chroot

---

## 3. Workspace layout

```text
r_tvui/
  Cargo.toml              workspace root; version must match release tag (without v)
  src/                    application binary + library
  crates/core/            artifact listing, sort, caps, errors
  docs/                   design, contributing notes, install, test guide, tutorial
  scripts/install.sh      curl-install for macOS/Linux
  .github/workflows/      ci.yml (PR gate), release.yml (tag builds)
```

**Dependencies (application):** ratatui 0.30, crossterm, tokio, clap, serde/toml, dirs, trash, arboard, shellexpand.

---

## 4. Architecture

```text
Terminal (stdin/stdout)
    ↔ ratatui 0.30 + crossterm (ratatui::run in main)
    ↔ App (multi-tab coordinator)
         └── BrowserPane[] (per-tab cwd, list, marks, git, previewer)
    ↔ events/app.rs       poll loop, editor suspend/restore
    ↔ events/keys.rs      mode dispatch (Active, Filter, Rename, …)
    ↔ AsyncEventClient    dedicated Tokio runtime + mpsc channel
    ↔ crates/core         sync/async directory listing
    ↔ config/             ~/.r_tvui/.config.toml
    ↔ os/                 system open, $EDITOR launcher
```

### Layer responsibilities

| Layer | Responsibility | Avoid |
|-------|----------------|--------|
| `main`, `events/app` | Terminal lifecycle, poll interval, editor handoff | Business rules for paths |
| `events/keys`, `events/key` | Key routing by `AppState` | Direct filesystem I/O |
| `models/app`, `models/pane` | Application state, tabs, file ops | ratatui widgets |
| `models/client` | Spawn async listing/preview tasks | Keymap logic |
| `ui/` | Layout and render from `App` snapshot | Blocking I/O |
| `crates/core` | Read directories, sort, cap entries | UI or config |

### Module map

```text
src/
  main.rs                 CLI entry, panic hook, ratatui::run
  lib.rs                  module tree + test wiring
  models/
    app.rs                App, tabs, modes, mutations, async event application
    pane.rs               BrowserPane, git marks, per-tab history
    client.rs             AsyncEventClient, AsyncEvents enum
    previewer.rs          side-pane content (text, folder, image summary)
  events/
    app.rs                main loop (drain async → editor → draw → keys)
    keys.rs               outer dispatcher per AppState
    key.rs                normal-mode keys + mode input handlers
    utils.rs              scroll, enter, navigation helpers
  ui/
    renders.rs            frame composition
    layout.rs             path bar, status bar, split layouts
    views.rs              tables, help overlay, column builders
  config/                 AppConfig, themes, load/save
  os/mod.rs               open_file (spawn), open_with_editor (blocking)
  cli/                    clap, start path resolution
crates/core/
  lib.rs                  Artifact, listing, formatters
  errors.rs               RTVUIError
  utils.rs                get_artifact_entries[_async]
```

---

## 5. State model

### App

`App` (`models/app.rs`) owns:

- `tabs: Vec<BrowserPane>` and `active_tab`
- `split_tab: Option<usize>` — when set, UI shows two panes side by side
- `AppState` — which key handler runs (Active, Filter, Rename, GoTo, Confirm, Help, Quit)
- `AsyncEventClient`, directory `cache`, config
- `pending_editor: Option<PathBuf>` — queued path for `$EDITOR`; the event loop suspends the TUI before launch
- `next_generation` / `next_previewer_generation` — **global** counters assigned to async requests

Global generation IDs ensure async completions always match the pane that issued the request, even across tabs.

### BrowserPane

Each tab is a `BrowserPane` (`models/pane.rs`):

- `current_working_directory`, listing buffers (`entries_cache`, `entries_filtered`)
- `scroll_state`, `filter_input`, navigation `history`
- `marked: HashSet<PathBuf>` for visual selection
- `git_marks` — basename → porcelain status char from `git status --porcelain`
- `generation` / `previewer_generation` — last assigned request IDs for this pane
- `previewer: Previewer`

### AppState modes

| State | Purpose |
|-------|---------|
| `Active` | Normal browsing |
| `Filter` | Live substring filter on listing |
| `Rename` | Inline rename buffer |
| `GoTo` | Path jump input (`~` expanded) |
| `Confirm` | Delete confirmation |
| `Help` | Help overlay |
| `Quit` | Exit main loop |

---

## 6. Event loop and async I/O

### Main loop (`events/app.rs`)

Each iteration:

1. Drain all pending `AsyncEvents` from the channel and apply via `App::apply_async_event`.
2. If `pending_editor` is set, suspend terminal (`ratatui::restore`), run editor, re-init terminal, refresh listing.
3. Draw frame.
4. Poll keyboard (250 ms timeout); dispatch through `events/keys::dispatch_key`.

### AsyncEventClient (`models/client.rs`)

- One Tokio `Runtime` and an `mpsc` channel (capacity 64).
- **`BrowserDone`** — async directory listing result + optional error string.
- **`FolderPreviewDone` / `PreviewerDone`** — side-pane content.

Listing uses `get_artifact_entries_async` from `crates/core`. File text/image preview uses `spawn_blocking` for disk reads.

### Stale-result handling

Each listing or preview request gets a monotonically increasing generation ID. When a completion arrives, it is applied only if the ID matches the pane’s current generation. In-flight tasks are **not cancelled**; late results are ignored.

### Directory cache

`App.cache: HashMap<PathBuf, ArtifactListResult>` stores recent listings. Cleared on manual refresh, sort change, or hidden toggle. A cache hit is shown immediately while a fresh async listing runs.

### Core listing (`crates/core`)

- Sync: `get_artifact_entries`
- Async: `get_artifact_entries_async`
- Cap: 50,000 entries per directory; `partial: true` when truncated
- Options: `show_hidden`, `sort` (Name / Size / Modified)

---

## 7. UI layer

### Layout

Default: path bar → 60/40 horizontal split (list | preview) → status bar.

Split mode (`\`): two equal columns, one per visible tab; preview pane hidden.

Help overlay: centered modal over the body area when `AppState::Help`.

### Rendering rules

- `ui/renders.rs` reads `App` immutably except for `scroll_state` on the active (or focused) pane.
- Git status and marks are applied in `ui/views.rs` when building table rows.
- Status bar text is mode-aware (`ui/layout.rs::get_status_bar`).

Key reference strings live in `ui/views.rs` (`HELP_LINES`) and should stay aligned with [README](../README.md) keybindings.

---

## 8. File operations

Implemented on `App` unless noted:

| Operation | Entry point | Notes |
|-----------|-------------|-------|
| Rename | `commit_rename` | Validates separators, `..`, collisions |
| Delete | `commit_delete` | Confirm mode; trash or permanent; supports marked set |
| Copy path | `copy_selected_path` | Single path or newline-separated marks via `arboard` |
| Bookmarks | `bookmark_cwd`, `jump_to_bookmark` | Max 9, persisted in config |
| History | `history_back`, `history_forward` | Per-tab stack with canonical paths |
| Pane copy/move | `copy_to_other_pane`, `move_to_other_pane` | Requires split; skips existing dest names |
| System open | `events/utils::handle_key_event_enter_mode` | Spawns platform opener |
| Editor | `open_in_editor` → `pending_editor` | Event loop owns terminal suspend |

Move across filesystems uses rename with copy+delete fallback (`move_path` in `app.rs`).

---

## 9. Preview and side pane

Preview mode (`settings.preview`): `OnMove`, `Always`, `Never` — cycled with `P`, persisted.

| Selection | Side pane |
|-----------|-----------|
| Directory | Async listing of children |
| Text file | First ~64 KiB as UTF-8 lossy string |
| Image (png/jpg/gif/…) | Metadata summary + dimensions from magic bytes |

`Previewer` enum: `Empty`, `Folder`, `Preview`. Generation discard applies the same way as listings.

---

## 10. Configuration

**Path:** `~/.r_tvui/.config.toml` (created on first run).

```toml
theme = "Forest"   # Forest | Midnight | Solar | Mono

[settings]
sort = "Name"           # Name | Size | Modified
enable_trash = true
preview = "OnMove"        # OnMove | Always | Never
show_hidden = false

[cache]
bookmarks = []          # up to 9 paths
```

Load/save: `config/utils.rs`. Theme and sort changes from the UI call `save_config` immediately.

---

## 11. UX contracts

These behaviors are intentional and covered by tests — preserve them when changing keymaps:

- **Esc cancels** transient modes (Filter, GoTo, Rename, Help, Confirm); it does not quit except from normal mode handling.
- **`q` in Filter / GoTo** is literal input, not quit.
- **`d` / `l`** enter directories; **`x` / Delete** delete (not `d`).
- **`h` / Home** jumps to filesystem root (not vim-left parent).
- **Confirm delete** requires explicit `y`; `n`, Esc, and `q` cancel.

Full keymap: [README § Keybindings](../README.md#keybindings).

---

## 12. Security model

- Local filesystem only; no network service.
- Files open only on explicit Enter or `e`.
- Rename rejects path separators and `..` in new names.
- No enforced path jail — the process runs with the user’s OS permissions.

---

## 13. Testing and CI

Quality gate (also `make fmt`, `make lint`, `make test`):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

CI (`.github/workflows/ci.yml`) runs on `master` and `release/**` branches. Release builds (`.github/workflows/release.yml`) trigger on `v*` tags and produce platform tarballs/zip with stable “latest” alias names.

See [test.md](./test.md) for module-level test layout and coverage exclusions.

---

## 14. Distribution

| Channel | Location |
|---------|----------|
| Download page | https://r-tvui-web.vercel.app/download |
| GitHub Releases | Versioned assets + short-name aliases |
| install.sh | `~/.local/bin` on macOS/Linux |

Release version in root `Cargo.toml` and `crates/core/Cargo.toml` must match the git tag without the `v` prefix (e.g. tag `v3.1.0` → version `3.1.0`). See [CHANGELOG.md](../CHANGELOG.md) for release notes.

---

## 15. Known limitations

1. Async tasks are ignored when stale, not aborted (`JoinHandle` cancel is future work).
2. One Tokio runtime per process.
3. Image preview is summary-only (no in-terminal rendering).
4. `OnMove` and `Always` preview modes behave similarly; only `Never` fully disables the pane.
5. Git column runs `git status` synchronously when a listing is applied (acceptable for typical repos).

---

## 16. Future directions

Plausible extension areas (not implemented):

- **Plugin / spotter API** — cheap metadata hooks before full preview
- **Terminal graphics** — kitty/sixel behind a feature flag
- **Remote VFS** — SSH/SFTP as optional backend
- **Richer preview modes** — clearer distinction between OnMove and Always
- **Configurable keymap** — today bindings are compile-time constants

When adding features, prefer extending `BrowserPane` / `App` methods and wiring through `events/keys.rs` rather than introducing new global singletons.

---

## 17. Glossary

| Term | Meaning |
|------|---------|
| **Artifact** | File or directory entry from `crates/core` |
| **BrowserPane** | Per-tab browser state (cwd, list, marks, history) |
| **Generation** | Unique ID for an async listing/preview request |
| **AsyncEventClient** | Tokio runtime + channel delivering completion events |
| **Previewer** | Side-pane model: empty, folder listing, or text/image summary |
| **Split** | Dual-pane UI showing two tabs without the preview column |
