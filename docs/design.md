# R-TVUI — Design & implementation

**Version:** 0.1.0 (`Cargo.toml`) — in progress (not a tagged production cut)  
**Status:** Living document — describes what ships on **master today**, what is stubbed, and what is planned. For a gap audit, see **[review.md](./review.md)**.

This document consolidates the former design spec, implementation plan, install notes, and distribution guide into a single reference for **how R-TVUI is designed and how it is built through phases**.

For a hands-on rebuild from scratch, see **[tutorial.md](./tutorial.md)**. Tutorial phases describe the *target* curriculum and may still mention crates/paths that were folded or renamed on master.

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

Power users live in the terminal but often fall back to `ls`, `cd`, and ad-hoc scripts. **R-TVUI** is a dedicated **TUI file explorer** in Rust that reduces friction: browse with keyboard, preview before open, filter and sort, and open files with the system default app — without leaving the shell.

**North star:** Responsiveness (non-blocking I/O, predictable ops) with a **small, shippable core**.

**Today on master:** async browse + filter + rename + themes/sort/hidden + text/folder preview + go-to path + help overlay. Delete/trash, clipboard, bookmarks, and history are **not finished** (see §4 / §9 / §16).

---

## 2. Product principles

1. **Never block the UI thread** — directory listing, side-pane reads, and file open run off the main loop (async Tokio + background threads).
2. **Keyboard-first** — every shipping action has a default binding; config persists theme, sort, preview, hidden, and (schema-ready) trash/bookmarks.
3. **Predictable file operations** — confirm destructive actions when delete ships; optional trash via the `trash` crate (**planned**).
4. **Terminal realism** — true-color themes; degrade gracefully on limited terminals (`Mono` theme).
5. **Single binary** — workspace members: binary `r_tvui` + `crates/core` only (filesystem listing lives in `core`; no dynamic plugins).

---

## 3. Personas

| Persona | Needs | Support on master |
|--------|--------|-------------------|
| **Dev on large repos** | Fast navigation, filter, sort, path copy | **Partial** — async listing, 50k cap, cache, filter/sort; copy-path not yet |
| **Ops / SRE** | Clear errors, trash, open with system app | **Partial** — open works; trash/delete not yet; listing errors often silent |
| **Minimalist** | Works out of the box, saved config | **Yes** — defaults + `~/.r_tvui/.config.toml` |

---

## 4. Phased delivery

Milestones **M0–M4** below are the **product roadmap**. (Separate IDs in [review.md](./review.md) track implementation gaps.)

| Phase | Scope | Exit criteria | Status (master / 0.1.0) |
|-------|--------|---------------|-------------------------|
| **M0 — Spike** | TUI frame, list one directory, quit | Opens, lists cwd, quits cleanly | **Done** (WASD nav, not vim `j`/`k`) |
| **M1 — MVP browser** | Navigate, filter, sort, text preview, config, themes | Daily-usable local browser | **Done** (no multi-tab; WASD nav) |
| **M2 — File ops** | Delete (trash), rename, clipboard path, bookmarks, history | Destructive ops with confirm | **Partial** — rename only; delete/clipboard/bookmarks/history missing |
| **M2.5 — Async** | Async listing, directory cache, generation guards, background open | Large dirs stay responsive | **Done** |
| **M3 — Power** | Split dual-cwd, image preview, git column, external tools | Power-user parity | **Planned** |
| **M4 — Plugins** | Previewer/spotter API, third-party extensions | Extensibility | **Planned** |

### 4.1 Original vs actual structure

Early planning proposed many crates (`r-tvui_app`, `r-tvui_fs`, `r-tvui_preview`, …) and a separate `crates/filesystem`. **Master uses:**

- **Binary crate** `r_tvui` — `src/` with `models`, `events`, `ui`, `config`, `os`, `cli`
- **`crates/core`** — artifacts, sync/async listing, sort, 50k cap, errors

### 4.2 Features (phased)

| Feature | Target phase | Master |
|---------|--------------|--------|
| Single-pane browser + status line | M0 | **Yes** |
| Multi-tab | M1 | **No** |
| Async listing + text preview | M1 / M2.5 | **Yes** |
| Filter / sort / themes / rename | M1–M2 | **Yes** |
| Delete / trash / clipboard / bookmarks / history | M2 | **No** (schema stubs for trash/bookmarks) |
| GoTo path / Help overlay | M1 | **Yes** |
| Visual selection + bulk copy/move | M2+ | **No** |
| Image preview (terminal-dependent) | M3 | **No** |
| Split panes (dual cwd) | M3 | **No** (side pane is preview/folder, not second cwd) |
| Plugin API | M4 | **No** |
| Remote VFS (SSH/SFTP) | Later | **No** |

---

## 5. Architecture (as built)

```text
Terminal (stdin/stdout)
    ↔ ratatui 0.30 + crossterm (via ratatui::run)
    ↔ App state (models/app.rs — cwd, selection, modes, previewer)
    ↔ Event loop (events/app.rs — poll + key dispatch)
    ↔ AsyncEventClient (models/client.rs — Tokio runtime, mpsc events)
    ↔ list_artifact_entries[_async] (crates/core)
    ↔ Config (config/* — TOML at ~/.r_tvui/.config.toml)
    ↔ Opener (os/mod.rs — background system open)
```

### 5.1 Module map

```text
src/
  main.rs              CLI (clap), panic hook, ratatui::run
  lib.rs               modules + test wiring
  models/
    app.rs             App, AppState, filter/rename/theme/sort/cache
    client.rs          AsyncEventClient (Tokio + mpsc)
    previewer.rs       Text / folder preview types
  events/
    app.rs             Main loop: poll keys, drain async events, draw
    keys.rs            Outer mode dispatch (Esc cancel contract)
    key.rs             Per-mode key handlers
    utils.rs           Scroll / enter / home helpers
  ui/
    renders.rs         Layout: file list + side pane + status
    layout.rs, views.rs, utils.rs
  config/
    app.rs             AppConfig, themes, sort, preview, settings
    utils.rs           Load/save ~/.r_tvui/.config.toml
  os/mod.rs            open_file (macOS/Linux/Windows)
  cli/                 Clap model + start-path + panic restore hook
crates/
  core/                Artifact listing, sort, formatters, errors
```

### 5.2 Layer responsibilities

| Layer | Owns | Must not |
|-------|------|----------|
| `main` / `events` | argv, terminal lifecycle, poll interval | Business rules for paths |
| `models` | App state, modes, previewer, async client | Keymap tables |
| `ui` | ratatui layout from App snapshot | Blocking I/O |
| `models/client` | Spawn async reads | Keymap logic |
| `crates/core` | Directory reads, sort, cap | UI or keybindings |

---

## 6. Runtime & data flow

### 6.1 Main loop

1. `App::new(optional path)` loads config, resolves start path, requests initial listing.
2. Loop (`events/app.rs`):
   - Drain `AsyncEvents` from `AsyncEventClient` (browser list, folder preview, text preview).
   - Apply results only if `generation` matches.
   - Poll keyboard; `dispatch_key` → navigation, filter, ops, etc.
   - Draw via `ui::renders`.

### 6.2 Async listing

- **`AsyncEventClient`** holds a dedicated **Tokio runtime** and an **`mpsc`** channel for completion events.
- Listing runs on the runtime; results may be stored in an in-app **per-path cache** (cleared on refresh/sort/hidden).
- **Browser** and **side pane** both request listings through the client.

### 6.3 Stale-result guard

Fast navigation increments **generation counters** before each request. Late completions are **discarded** if their generation no longer matches.

*Note:* In-flight tasks are not aborted; they still complete but are ignored. Optional `JoinHandle` cancel is a future improvement.

### 6.4 Background file open

Opening files with the OS default app uses a background spawn so macOS `open`, Linux `xdg-open`, and Windows `start` never block the TUI loop. Failures are currently silent.

### 6.5 Cache invalidation

| Action | Cache behavior |
|--------|----------------|
| Manual refresh (`r`) | Clear cache, reload |
| Sort / hidden toggle | Clear full cache |
| Rename (and future delete) | Clear cache, async refresh |

---

## 7. Workspace layout

```text
r_tvui/
  Cargo.toml           workspace: ".", crates/core
  src/                 application binary + library
  crates/
    core/              rtvui-core — listing, artifacts, errors
  docs/
    design.md          this file
    tutorial.md        rebuild guide
    review.md          implementation audit
    test.md            test / coverage conventions
  scripts/
    install.sh         curl-install for macOS/Linux
  .github/workflows/
    .github.yml        intended CI (must be renamed to ci.yml — currently hidden)
    .release.yml       intended release (must be renamed to release.yml)
```

**Stack today:** Rust 2024 edition, ratatui 0.30, tokio, clap, serde/toml, dirs.  
**Planned deps for M2:** `trash`, `arboard` (clipboard).

---

## 8. Core domains

| Domain | Responsibility | Primary location |
|--------|----------------|------------------|
| **App state** | cwd, selection, filter, sort, modes | `models/app.rs` |
| **Directory cache** | Per-path listing (in-app map) | `models/app.rs` |
| **Listing tasks** | Browser vs side-pane requests, events | `models/client.rs` |
| **Navigation** | Parent, enter dir, scroll, home→root | `events/utils.rs` |
| **Side pane** | Folder listing / file preview | `models/previewer.rs` + `ui/` |
| **File ops** | Rename (shipping); delete planned | `models/app.rs` |
| **Config** | Theme, sort, trash flag, bookmarks, preview | `config/` |
| **Themes** | Forest, Midnight, Solar, Mono | `config/app.rs` |

### 8.1 Core crate

- **`get_artifact_entries`** / async variant with **`ArtifactOptions`** (hidden, sort).
- **Cap:** 50,000 entries per directory; **`partial`** flag when truncated (UI does not surface it yet).
- Errors as **`RTVUIError` / `FileSystemErrors`**.

---

## 9. UX specification

### 9.1 Layout (default)

```text
┌─ path: ~/proj/src                                          ┐
├──────────────────────────────┬─────────────────────────────┤
│  NAME          SIZE    MODIFIED │  SIDE PANE                 │
│  > src/        -       ...     │  (folder listing or        │
│    main.rs     4.2K    ...     │   text preview)            │
├──────────────────────────────┴─────────────────────────────┤
│ status message · key hints                                  │
└─────────────────────────────────────────────────────────────┘
```

### 9.2 Keybindings (shipping on master)

| Key | Action | Status |
|-----|--------|--------|
| `w` / `↑` | Move selection up | Shipping |
| `s` / `↓` | Move selection down | Shipping |
| `a` / `←` | Parent directory | Shipping |
| `d` / `→` | Enter selected **directory** | Shipping |
| `Enter` | Open file with system app, or enter directory | Shipping |
| `h` / `Home` | Walk to filesystem root (`/`) | Shipping (not `$HOME`) |
| `/` | Filter mode (live substring filter) | Shipping |
| `o` | Cycle sort (name / size / modified) | Shipping |
| `.` | Toggle hidden files | Shipping |
| `t` | Cycle theme (saved to config) | Shipping |
| `r` | Refresh listing (clears cache) | Shipping |
| `F2` | Rename | Shipping |
| `g` | Go to path (type path, Enter jumps; `~` ok) | Shipping |
| `?` | Help overlay | Shipping |
| `q` / `Esc` | Quit in normal mode; **Esc cancels** transient modes | Shipping |

### 9.3 Planned bindings (not shipping)

| Key | Planned action |
|-----|----------------|
| `j` / `k` / `l` | Vim-style nav (optional alias) |
| `G` | Jump to `$HOME` |
| `u` / `i` | History back / forward |
| `y` | Copy path to clipboard |
| `b` / `1`–`9` | Bookmark / jump |
| `P` | Cycle preview mode (`OnMove` / `Always` / `Never`) |
| Delete key (TBD — not `d`) | Delete with confirm; trash if `enable_trash` |

### 9.4 Modes

| Mode | Enter | Esc | Notes |
|------|-------|-----|-------|
| **Active** (normal) | — | Quit | Also `q` quits |
| **Filter** | `/` | Clear + Active | Typed chars narrow list (`q` is literal) |
| **Rename** | `F2` | Active | Enter commits |
| **GoTo** | `g` | Clear + Active | Type path; Enter jumps (`q` is literal) |
| **Help** | `?` | Active | Overlay; `q` also closes |
| **Confirm** | (planned delete) | Active | Unreachable until delete ships |

### 9.5 File operations

- **Rename:** `F2`, inline buffer, validate before apply — **shipping**.
- **Delete:** confirm flow + optional trash — **planned** (`enable_trash` in config unused).
- **Open file:** system handler (silent no-op if spawn fails).

---

## 10. Preview & side pane

### 10.1 Pipeline (shipping)

1. User changes selection → request side pane (unless `preview == Never`).
2. **Directory:** side pane shows folder listing (async).
3. **File:** text prefix (~64 KiB) into side pane.
4. New selection bumps **previewer generation**; stale results dropped.

`OnMove` and `Always` currently behave the same; only `Never` disables the pane. A `P` cycle is planned.

### 10.2 Planned (M3+)

- Spotter (fast mime/binary sniff) before full read.
- Pluggable previewers; external `bat` / `chafa` hooks.
- Image protocols (kitty / iTerm2) behind feature flags.

---

## 11. Configuration

- **Path (actual):** `~/.r_tvui/.config.toml`
- **Auto-created** on first run with defaults.
- **No** `RTVUI_CONFIG` override yet (earlier docs mentioned XDG `~/.config/rtvui/`).

Example (shape matches serde types on master):

```toml
theme = "Forest"   # Forest | Midnight | Solar | Mono

[settings]
sort = "Name"           # Name | Size | Modified
enable_trash = true     # persisted; not consumed until delete ships
preview = "OnMove"      # OnMove | Always | Never
show_hidden = false

[cache]
bookmarks = []          # persisted; no keybindings yet
```

**Themes:** Forest, Midnight, Solar, Mono (no `gotyme`).

---

## 12. Quality attributes

| Attribute | Target | Approach on master |
|-----------|--------|--------------------|
| Cold start | Fast first frame | Single binary, minimal init |
| Large directories | Responsive UI | Async list + 50k cap + cache |
| Memory | Bounded | Cache cleared on refresh/sort/hidden |
| Accessibility | Readable | High-contrast `Mono` theme |

---

## 13. Security

- Operates only on paths the **user can read**; opens files only on **explicit** Enter.
- **No network service** — local filesystem only.
- Rename validates separators / `..` / collisions.
- Path “jail” / root restriction — **not enforced**.
- Release binaries should be built from **tagged CI** sources (CI workflow filenames currently prevent Actions from running — see §14).

---

## 14. Testing & CI

### 14.1 Local (desired gate)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

Makefile today: `make fmt` **mutates**; `make lint` does **not** pass `-D warnings`. Prefer the commands above for release checks. Coverage: see [test.md](./test.md).

### 14.2 CI

Intended: on push/PR to `master` — fmt check, clippy `-D warnings`, workspace tests.  
**Blocker:** workflow files are still named `.github.yml` / `.release.yml` under `.github/workflows/` — GitHub ignores hidden workflow filenames. Rename to `ci.yml` / `release.yml` before relying on CI.

### 14.3 Tests in tree

- Unit tests under `src/tests/` and `crates/core/src/tests/` (~97 cases when green).
- Prefer `tempfile` fixtures for filesystem integration tests.

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

1. Bump `version` in root `Cargo.toml` (must match the tag).
2. `cargo test --workspace` and clippy `-D warnings` green.
3. Tag a **new** semver — do **not** reuse existing `v2.0.0` / `v3.0.0` (those point at older trees). Prefer continuing from the `v1.x` line or a clear `0.x` until M2 is complete.
4. **Release workflow** (once renamed) builds matrix artifacts → GitHub Releases.
5. Update release notes / CHANGELOG.

### 15.3 Requirements for users

- Modern terminal (true-color recommended).
- macOS: `open` built in.
- Linux: `xdg-open` (`xdg-utils` package).
- Windows: `cmd start`.

---

## 16. Known limitations & roadmap

### 16.1 Current limitations (0.1.0)

1. No explicit task cancel — generation discard only.
2. One Tokio runtime per app instance.
3. M2 incomplete: no delete/trash, clipboard, bookmarks, history.
4. GoTo and Help ship; Confirm still unused until delete.
5. `h`/`Home` → `/`, not `$HOME`; no `G` home jump.
6. `partial` (50k cap) not shown in UI; async list errors may look like empty dirs.
7. No tabs, visual multi-select, or copy/move queue.
8. CI/release workflows not discoverable until renamed (see §14.2).
9. Version tags / Cargo / this doc historically drifted — treat **Cargo.toml** as source of truth.

### 16.2 Recommended next steps

| Priority | Task |
|----------|------|
| **High** | M2 slice: delete + Confirm + `enable_trash` |
| **Medium** | Bookmarks / clipboard / history; `P` preview cycle |
| **Medium** | Rename CI workflows; `fmt --check`; clippy `-D warnings` |
| **Low** | `$HOME` jump; surface `partial`; abort in-flight listing |
| **Low** | Multi-tab; git column (M3) |

---

## 17. Risks

| Risk | Mitigation |
|------|------------|
| Terminal preview fragmentation | Capability probe before image preview (M3) |
| Feature creep vs Yazi | Phased roadmap; ship small binary |
| Async race bugs | Generation tokens; tests for navigation |
| Doc/tag overclaim | Keep this file aligned with Cargo version + [review.md](./review.md) |

---

## 18. Glossary

| Term | Meaning |
|------|---------|
| **Browser** | Main file list for current `cwd` |
| **Side pane** | Right column: folder contents or file preview |
| **Generation** | Monotonic counter to ignore stale async results |
| **AsyncEventClient** | Tokio + channel coordinator for listings/previews |
| **Spotter** | (Planned) Cheap metadata before full preview |
| **Previewer** | Side-pane content (text or folder listing) |

---

*This document supersedes `DESIGN_SPEC.md`, `INSTALL.md`, `DISTRIBUTION.md`, and `planning/*`. Implementation gaps live in [review.md](./review.md).*
