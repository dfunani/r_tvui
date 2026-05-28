# R-TVUI — Design & implementation

**Version:** 2.0.0 (shipping)  
**Status:** Living document — describes what ships today and what is planned next.

This document consolidates the former design spec, implementation plan, implementation blueprint, architecture review, install notes, and distribution guide into a single reference for **how R-TVUI is designed and how it is built through phases**.

For a hands-on rebuild from scratch, see **[tutorial.md](./tutorial.md)**.

---

## Table of contents

1. [Problem & goals](#1-problem--goals)
2. [Product principles](#2-product-principles)
3. [Personas](#3-personas)
4. [Phased delivery](#4-phased-delivery)
5. [Architecture (as built)](#5-architecture-as-built)
6. [Runtime & data flow](#6-runtime--data-flow)
7. [Workspace layout](#7-workspace-layout)
8. [Core domains](#8-core-domains)
9. [UX specification](#9-ux-specification)
10. [Preview & side pane](#10-preview--side-pane)
11. [Configuration](#11-configuration)
12. [Quality attributes](#12-quality-attributes)
13. [Security](#13-security)
14. [Testing & CI](#14-testing--ci)
15. [Distribution & releases](#15-distribution--releases)
16. [Known limitations & roadmap](#16-known-limitations--roadmap)
17. [Risks](#17-risks)
18. [Glossary](#18-glossary)

---

## 1. Problem & goals

Power users live in the terminal but often fall back to `ls`, `cd`, and ad-hoc scripts. **R-TVUI** is a dedicated **TUI file explorer** in Rust that reduces friction: browse with keyboard, preview before open, filter and sort, bookmarks, trash delete, and open files with the system default app — without leaving the shell.

**North star:** Responsiveness (non-blocking I/O, predictable ops) with a **small, shippable core**.

---

## 2. Product principles

1. **Never block the UI thread** — directory listing, side-pane reads, and file open run off the main loop (async Tokio + background threads).
2. **Keyboard-first** — every action has a default binding; config persists theme, sort, trash, bookmarks.
3. **Predictable file operations** — confirm destructive actions; optional trash via the `trash` crate.
4. **Terminal realism** — true-color themes; degrade gracefully on limited terminals (`mono` theme).
5. **Single binary** — workspace crates `core` and `filesystem` only; no dynamic plugins in v2.

---

## 3. Personas

| Persona | Needs | v2 support |
|--------|--------|------------|
| **Dev on large repos** | Fast navigation, filter, sort, path copy | Yes — async listing, 50k cap, cache |
| **Ops / SRE** | Clear errors, trash, open with system app | Yes — partial listing flag, `xdg-open` / `open` |
| **Minimalist** | Works out of the box, saved config | Yes — defaults + `~/.config/rtvui/config.toml` |

---

## 4. Phased delivery

The project was planned in milestones **M0–M4**. The shipping app implements **M0 + most M1/M2 UX** in a **simpler crate layout** than the original multi-crate blueprint.

| Phase | Scope | Exit criteria | Status (v2.0.0) |
|-------|--------|---------------|-----------------|
| **M0 — Spike** | TUI frame, list one directory, j/k/h/l/q | Opens, lists cwd, quits cleanly | Done |
| **M1 — MVP browser** | Navigate, filter, sort, text preview, config, themes | Daily-usable local browser | Done (no multi-tab) |
| **M2 — File ops** | Delete (trash), rename, clipboard path, bookmarks, history | Destructive ops with confirm | Done |
| **M2.5 — Async v2** | Async listing, directory cache, generation guards, background open | Large dirs stay responsive | Done |
| **M3 — Power** | Split dual-cwd, image preview, git column, external tools | Power-user parity | Planned |
| **M4 — Plugins** | Previewer/spotter API, third-party extensions | Extensibility | Planned |

### 4.1 Original vs actual structure

Early planning proposed many crates (`r-tvui_app`, `r-tvui_fs`, `r-tvui_preview`, …). The shipped design uses:

- **Binary crate** `r_tvui` — `src/` with `models`, `events`, `ui`, `utils`
- **`crates/core`** — paths, formatters, ID newtypes reserved for later
- **`crates/filesystem`** — sync + async directory listing, sort, cache, 50k entry cap

This keeps compile times and mental overhead low while preserving clear boundaries.

### 4.2 Features (phased)

| Feature | Target phase | v2.0.0 |
|---------|--------------|--------|
| Single-pane browser + status line | M0 | Yes |
| Multi-tab | M1 | No |
| Async listing + text preview | M1 / v2 | Yes |
| Visual selection + bulk copy/move | M2 | No |
| Image preview (terminal-dependent) | M3 | No |
| Split panes (dual cwd) | M3 | No (side pane is preview/folder, not second cwd) |
| Plugin API | M4 | No |
| Remote VFS (SSH/SFTP) | Later | No |

---

## 5. Architecture (as built)

```text
Terminal (stdin/stdout)
    ↔ ratatui 0.30 + crossterm (via ratatui::run)
    ↔ App state (models/app.rs — cwd, selection, modes, side pane)
    ↔ Event loop (events/app.rs — poll + key dispatch)
    ↔ ListingService (utils/listing.rs — Tokio runtime, mpsc events)
    ↔ DirectoriesCache (crates/filesystem — TTL + LRU)
    ↔ list_directories_async (crates/filesystem)
    ↔ Config (utils/config.rs — TOML)
    ↔ Opener (utils/opener.rs — background system open)
```

### 5.1 Module map

```text
src/
  main.rs              CLI (clap), panic hook, ratatui::run
  lib.rs               modules + install_panic_hook
  models/
    app.rs             App, SidePane, ListingService hooks
    mode.rs            AppMode (Normal, Filter, GoToPath, ConfirmDelete, …)
  events/
    app.rs             Main loop: poll keys, drain listing events, draw
    keys.rs            Key → action dispatch
  ui/
    render.rs          Layout: file list + side pane + status
    app.rs             Widget helpers
  utils/
    listing.rs         Tokio runtime, cache, ListingEvent channel
    navigation.rs      refresh cwd, generation tokens (stale discard)
    browser.rs         Side-pane folder/preview requests
    previewer.rs       Text preview for files
    opener.rs          open_in_background (macOS/Linux/Windows)
    config.rs          Load/save ~/.config/rtvui/config.toml
    ops.rs             delete, rename
    filter.rs, history.rs, clipboard.rs, startup.rs, …
crates/
  core/                File, AbsolutePath, formatters, ids (reserved)
  filesystem/          list_directories, cache, sort, errors
```

### 5.2 Layer responsibilities

| Layer | Owns | Must not |
|-------|------|----------|
| `main` / `events` | argv, terminal lifecycle, poll interval | Business rules for paths |
| `models` | App state, modes, side pane enum | Direct filesystem I/O |
| `ui` | ratatui layout from App snapshot | Blocking I/O |
| `utils/listing` | Spawn async reads, cache access | Keymap logic |
| `filesystem` | Directory reads, sort, cap, cache | UI or keybindings |

---

## 6. Runtime & data flow

### 6.1 Main loop

1. `App::new(optional path)` loads config, resolves start path, requests initial browser listing.
2. `App::run(terminal)` loops:
   - Drain `ListingEvent`s from `ListingService` (browser list, side folder, side preview).
   - Apply results only if `generation` matches `browser_listing_gen` / `side_pane_gen`.
   - Poll keyboard (~16 ms); dispatch keys → navigation, filter, ops, etc.
   - `terminal.draw` → `ui::render`.

### 6.2 Async listing

- **`ListingService`** holds a dedicated **Tokio runtime** and an **`mpsc`** channel for completion events.
- **`list_directories_async`** runs on the runtime; results are wrapped in **`Arc<DirectoryListResult>`** and stored in **`DirectoriesCache`** (shared mutex).
- **Browser** and **side pane** share the cache — revisiting a recently listed folder avoids redundant disk reads.

### 6.3 Stale-result guard

Fast navigation increments **generation counters** before each request. Late completions are **discarded** if their generation no longer matches. This avoids showing the wrong directory after rapid `j`/`k` or `h`/`l`.

*Note:* In-flight tasks are not aborted; they still complete but are ignored. Acceptable for v2; optional `JoinHandle` cancel is a future improvement.

### 6.4 Background file open

Opening files with the OS default app uses **`open_in_background`** (`thread::spawn`) so macOS `open`, Linux `xdg-open`, and Windows `start` never block the TUI loop.

### 6.5 Cache invalidation

| Action | Cache behavior |
|--------|----------------|
| Manual refresh (`r`) | Invalidate entry for cwd, reload |
| Sort / hidden toggle | Clear full cache |
| Delete / rename | Clear cache, async refresh |

---

## 7. Workspace layout

```text
r_tvui/
  Cargo.toml           workspace: r_tvui, crates/core, crates/filesystem
  src/                 application binary + library
  crates/
    core/              rtvui-core — paths, formatters
    filesystem/        directory listing, cache, errors
  docs/
    design.md          this file
    tutorial.md        rebuild guide
  scripts/
    install.sh         curl-install for macOS/Linux
  .github/workflows/
    ci.yml             fmt, clippy, test
    release.yml        matrix build on v* tags
```

**Stack:** Rust 2024 edition, ratatui 0.30, tokio, clap, serde/toml, trash, arboard (clipboard).

---

## 8. Core domains

| Domain | Responsibility | Primary location |
|--------|----------------|------------------|
| **App state** | cwd, selection, filter, sort, modes, bookmarks, history | `models/app.rs` |
| **Directory cache** | Per-path listing, TTL + LRU, invalidation | `filesystem::DirectoriesCache` |
| **Listing tasks** | Browser vs side-pane requests, events | `utils/listing.rs` |
| **Navigation** | Parent, enter dir, go to path, home, history | `utils/navigation.rs` |
| **Side pane** | Hidden / folder listing / file preview | `models/app.rs` (`SidePane`) |
| **File ops** | Delete (confirm, trash), rename | `utils/ops.rs` |
| **Config** | Theme, sort, trash, bookmarks, preview_on_move | `utils/config.rs` |
| **Themes** | gotyme, midnight, forest, solar, mono | `theme.rs` |

### 8.1 Filesystem crate

- **`list_directories`** / **`list_directories_async`** with **`DirectoryListOptions`** (hidden, sort).
- **Cap:** 50,000 entries per directory; **`partial`** flag when truncated.
- **`AbsolutePath`** — canonical paths for operations.
- Errors as **`FileSystemError`** (not found, permission denied, I/O).

### 8.2 Core crate

- **`File`**, **`FileType`**, path helpers, size/mtime formatters.
- **`ids`** — newtypes (`TabId`, etc.) reserved for future tabs/git UI.

---

## 9. UX specification

### 9.1 Layout (default)

```text
┌─ path: ~/proj/src                    [filter] ─────────────┐
├──────────────────────────────┬─────────────────────────────┤
│  NAME          SIZE    MODIFIED │  SIDE PANE                 │
│  > src/        -       ...     │  (folder listing or        │
│    main.rs     4.2K    ...     │   text preview)            │
├──────────────────────────────┴─────────────────────────────┤
│ status message · key hints                                  │
└─────────────────────────────────────────────────────────────┘
```

### 9.2 Keybindings (shipping)

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection down / up |
| `l` / Enter | Enter directory or open file with default app |
| `h` | Parent directory |
| `g` | Go to path (prompt) |
| `G` | Home directory |
| `u` / `i` | History back / forward |
| `/` | Filter listing by name |
| `s` | Cycle sort (name / size / modified) |
| `.` | Toggle hidden files |
| `p` / `P` | Refresh preview / toggle preview-on-move |
| `y` | Copy path to clipboard |
| `b` | Bookmark folder · `1`–`9` jump to bookmark |
| `F2` | Rename |
| `d` | Delete (confirm; trash if enabled) |
| `r` | Refresh listing (invalidates cache for cwd) |
| `t` | Cycle theme (saved to config) |
| `?` | Help overlay |
| `q` / `Esc` | Quit (`Esc` clears filter first) |

### 9.3 Modes

- **Normal** — navigation and commands.
- **Filter** — `/` then type; `Esc` clears.
- **GoToPath** — `g` then path; Enter confirms.
- **ConfirmDelete** — `d` then `y`/`n`.

### 9.4 File operations

- **Delete:** confirm overlay; **`use_trash`** in config uses OS trash when available.
- **Rename:** `F2`, inline buffer, validate before apply.
- **Open file:** system handler (silent no-op if no associated app).

---

## 10. Preview & side pane

### 10.1 Pipeline (v2)

1. User selects entry → optional **preview on move** (`preview_on_move` config).
2. **Directory:** side pane shows folder listing (async, cached).
3. **File:** **`previewer::preview_path`** reads text prefix into side pane.
4. New selection bumps **side_pane_gen**; stale preview/folder results dropped.

### 10.2 Planned (M3+)

- Spotter (fast mime/binary sniff) before full read.
- Pluggable previewers; external `bat` / `chafa` hooks.
- Image protocols (kitty / iTerm2) behind feature flags.

---

## 11. Configuration

- **Path:** `~/.config/rtvui/config.toml` (override: `RTVUI_CONFIG`).
- **Auto-created** on first run with defaults.

Example:

```toml
theme = "forest"
sort = "name"          # name | size | modified
use_trash = true
preview_on_move = true
bookmarks = ["/Users/you/Projects", "/Users/you/Downloads"]
```

**Themes:** `gotyme`, `midnight`, `forest`, `solar`, `mono`.

---

## 12. Quality attributes

| Attribute | Target | v2 approach |
|-----------|--------|-------------|
| Cold start | Fast first frame | Single binary, minimal init |
| Large directories | Responsive UI | Async list + 50k cap + cache |
| Memory | Bounded | Cache LRU/TTL; drop on clear |
| Accessibility | Readable | High-contrast `mono` theme |

---

## 13. Security

- Operates only on paths the **user can read**; opens files only on **explicit** Enter/`l`.
- **No network service** — local filesystem only.
- Path handling uses canonical absolute paths; future “jail” mode would restrict roots via config (planned in original spec, not enforced in v2).
- Release binaries should be built from **tagged CI** sources.

---

## 14. Testing & CI

### 14.1 Local

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

### 14.2 CI (`.github/workflows/ci.yml`)

On push/PR to `main`/`master`: fmt, clippy `-D warnings`, workspace tests.

### 14.3 Tests in tree

- Unit/integration tests under `src/tests/` (app state, listing, config).
- Prefer `tempfile` fixtures for future filesystem integration tests.

---

## 15. Distribution & releases

### 15.1 End-user install

| Method | Audience |
|--------|----------|
| `curl … install.sh` | macOS / Linux → `~/.local/bin` |
| [GitHub Releases](https://github.com/dfunani/r_tvui/releases) | All platforms — tar.gz / zip |

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `r_tvui-<ver>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `r_tvui-<ver>-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `r_tvui-<ver>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `r_tvui-<ver>-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | `r_tvui-<ver>-x86_64-pc-windows-msvc.zip` |

### 15.2 Maintainer release flow

1. Bump `version` in root `Cargo.toml`.
2. `cargo test --workspace` and `cargo clippy` green.
3. Tag: `git tag v2.0.0 && git push origin v2.0.0`
4. **Release workflow** builds matrix artifacts and publishes to GitHub Releases.
5. Update release notes; optional: project website `RELEASE_VERSION`.

### 15.3 Requirements for users

- Modern terminal (true-color recommended).
- macOS: `open` built in.
- Linux: `xdg-open` (`xdg-utils` package).
- Windows: `cmd start`.

---

## 16. Known limitations & roadmap

### 16.1 Current limitations (v2.0.0)

1. No explicit task cancel — generation discard only.
2. One Tokio runtime per app instance.
3. No tabs, visual multi-select, or copy/move queue UI.
4. Core ID/git types exist but UI does not use git column yet.
5. Very large single directories still heavy at read time (capped, async avoids freeze).

### 16.2 Recommended next steps

| Priority | Task |
|----------|------|
| Medium | Abort in-flight listing on navigation (`JoinHandle`) |
| Low | Integration tests for rename/delete with `tempfile` |
| Low | Multi-tab support |
| Low | Git status column (debounced) |

---

## 17. Risks

| Risk | Mitigation |
|------|------------|
| Terminal preview fragmentation | Capability probe before image preview (M3) |
| Feature creep vs Yazi | Phased roadmap; ship small binary |
| Async race bugs | Generation tokens; tests for navigation |

---

## 18. Glossary

| Term | Meaning |
|------|---------|
| **Browser** | Main file list for current `cwd` |
| **Side pane** | Right column: folder contents or file preview |
| **Generation** | Monotonic counter to ignore stale async results |
| **ListingService** | Tokio + channel + cache coordinator |
| **Spotter** | (Planned) Cheap metadata before full preview |
| **Previewer** | (Planned) Renders preview content by file type |

---

*This document supersedes `DESIGN_SPEC.md`, `REVIEW.md`, `INSTALL.md`, `DISTRIBUTION.md`, and `planning/*`.*
