# R-TVUI — Design (closed checklist)

**Version:** 3.1.0 (`Cargo.toml`)  
**Status:** **CLOSED** for the v3 product scope (M0–M3). This file is the finished design checklist, not an open backlog.  
**Audit:** [review.md](./review.md) (also closed).

Hands-on rebuild curriculum: [tutorial.md](./tutorial.md).

---

## Close-out summary

| Gate | Result |
|------|--------|
| Product phases M0–M3 | **Complete** |
| Design M4 (plugins) / remote VFS | **Out of scope** for 3.1.0 — consciously deferred |
| Docs aligned with Cargo `3.1.0` | **Yes** |
| Distribution path (CI + Releases + install.sh) | **Yes** |

---

## Table of contents

1. [Problem & goals](#1-problem--goals)
2. [Product principles](#2-product-principles)
3. [Personas](#3-personas)
4. [Phased delivery checklist](#4-phased-delivery-checklist)
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
16. [Accepted limitations](#16-accepted-limitations)
17. [Risks (closed)](#17-risks-closed)
18. [Glossary](#18-glossary)

---

## 1. Problem & goals

Power users live in the terminal but often fall back to `ls`, `cd`, and ad-hoc scripts. **R-TVUI** is a dedicated **TUI file explorer** in Rust: browse with the keyboard, preview before open, filter and sort, and open files without leaving the shell.

**North star (met for 3.1.0):** Responsive non-blocking I/O with a **small, shippable core** (single binary + `crates/core`).

**Ships in 3.1.0:** async browse · filter · rename · delete/trash · marks/bulk · copy path · bookmarks · history · go-to · help · themes/sort/hidden/preview · multi-tab · dual-cwd split · git column · image summary · `$EDITOR`.

---

## 2. Product principles

| # | Principle | 3.1.0 |
|---|-----------|-------|
| 1 | Never block the UI thread (Tokio + background open) | ✅ |
| 2 | Keyboard-first; defaults + persisted config | ✅ |
| 3 | Confirm destructive ops; optional trash | ✅ |
| 4 | True-color themes; `Mono` degrade path | ✅ |
| 5 | Single binary — no dynamic plugins in this release | ✅ |

---

## 3. Personas

| Persona | Needs | 3.1.0 |
|--------|--------|-------|
| **Dev on large repos** | Fast nav, filter, sort, path copy, git cues | ✅ |
| **Ops / SRE** | Clear errors, trash, system open, editor | ✅ |
| **Minimalist** | Defaults + saved config | ✅ |

---

## 4. Phased delivery checklist

Milestones **M0–M3** are the **closed product scope**. M4 was never part of the v3 exit criteria.

| Phase | Scope | Exit criteria | Checklist |
|-------|--------|---------------|-----------|
| **M0 — Spike** | TUI frame, list one directory, quit | Opens, lists, quits | ✅ Done |
| **M1 — MVP browser** | Navigate, filter, sort, text preview, config, themes | Daily-usable browser | ✅ Done |
| **M2 — File ops** | Delete/trash, rename, clipboard, bookmarks, history | Confirm + persist | ✅ Done |
| **M2.5 — Async** | Async list, cache, generation guards, background open | Large dirs stay responsive | ✅ Done |
| **M3 — Power** | Dual-cwd, image summary, git column, editor, tabs, marks | Power-user parity | ✅ Done |
| **M4 — Plugins** | Previewer/spotter API | Extensibility | ⬜ Out of scope (3.1.0) |

### 4.1 Structure (as shipped)

- **Binary** `r_tvui` — `models`, `events`, `ui`, `config`, `os`, `cli`
- **`crates/core`** — artifacts, listing, sort, 50k cap, errors

### 4.2 Feature checklist

| Feature | Phase | Checklist |
|---------|-------|-----------|
| Single-pane browser + status | M0 | ✅ |
| Multi-tab | M3 | ✅ |
| Async listing + text preview | M1 / M2.5 | ✅ |
| Filter / sort / themes / rename | M1–M2 | ✅ |
| Delete / trash / clipboard / bookmarks / history | M2 | ✅ |
| GoTo / Help | M1 | ✅ |
| Marks + bulk delete; split pane copy/move | M3 | ✅ |
| Image preview summary (dims for PNG/GIF/JPEG) | M3 | ✅ |
| Dual-cwd split | M3 | ✅ |
| Vim `j`/`k`/`l` (`h` = root) | M3 | ✅ |
| Git status column | M3 | ✅ |
| `$EDITOR` open | M3 | ✅ |
| Plugin API | M4 | ⬜ Out of scope |
| Remote VFS | Later | ⬜ Out of scope |
| Kitty/sixel raster images | Beyond M3 | ⬜ Out of scope |

---

## 5. Architecture (as built)

```text
Terminal (stdin/stdout)
    ↔ ratatui 0.30 + crossterm (via ratatui::run)
    ↔ App (tabs of BrowserPane — cwd, selection, marks, git, previewer)
    ↔ Event loop (events/app.rs — poll + key dispatch)
    ↔ AsyncEventClient (Tokio runtime, mpsc events)
    ↔ list_artifact_entries[_async] (crates/core)
    ↔ Config (~/.r_tvui/.config.toml)
    ↔ Opener (os/mod.rs — system open + $EDITOR)
```

### 5.1 Module map

```text
src/
  main.rs              CLI, panic hook, ratatui::run
  lib.rs               modules + tests
  models/
    app.rs             App, tabs, modes, file ops
    pane.rs            BrowserPane (per-tab state)
    client.rs          AsyncEventClient
    previewer.rs       Text / folder / image-summary preview
  events/              Loop, mode dispatch, keys, scroll helpers
  ui/                  Layout, views, render
  config/              AppConfig + load/save
  os/mod.rs            open_file + open_with_editor
  cli/                 Clap + start path
crates/core/           Artifact listing, sort, formatters, errors
```

### 5.2 Layer responsibilities

| Layer | Owns | Must not |
|-------|------|----------|
| `main` / `events` | argv, terminal lifecycle, poll | Path business rules |
| `models` | App / pane state, async client | Keymap tables |
| `ui` | ratatui from App snapshot | Blocking I/O |
| `crates/core` | Directory reads, sort, cap | UI or keybindings |

---

## 6. Runtime & data flow

1. `App::new` loads config, starts first tab listing.
2. Loop: drain async events (generation-matched) → dispatch keys → draw.
3. Listings/previews run on a dedicated Tokio runtime; per-path cache cleared on refresh/sort/hidden.
4. Stale results discarded via generation counters (tasks are not aborted — accepted).
5. System open is background-spawned; `$EDITOR` runs synchronously by design.

---

## 7. Workspace layout

```text
r_tvui/
  Cargo.toml
  src/
  crates/core/
  docs/          design.md (this) · review.md · INSTALL.md · tutorial.md · test.md
  scripts/install.sh
  .github/workflows/ci.yml · release.yml
```

**Stack:** Rust 2024, ratatui 0.30, tokio, clap, serde/toml, dirs, trash, arboard, shellexpand.

---

## 8. Core domains

| Domain | Location | Checklist |
|--------|----------|-----------|
| App + tabs | `models/app.rs`, `pane.rs` | ✅ |
| Directory cache | `models/app.rs` | ✅ |
| Async listing / preview | `models/client.rs` | ✅ |
| Navigation | `events/utils.rs` | ✅ |
| Side pane | `previewer.rs` + `ui/` | ✅ |
| File ops (rename/delete/copy/marks/split transfer) | `models/app.rs` | ✅ |
| Config / themes | `config/` | ✅ |
| Git marks | `models/pane.rs` | ✅ |

**Core crate:** sync/async listing, `ArtifactOptions`, 50k cap + `partial` (surfaced in status), `RTVUIError`.

---

## 9. UX specification

### 9.1 Layout

```text
┌─ [1/2] path: ~/proj/src                                    ┐
├──────────────────────────────┬─────────────────────────────┤
│  * [M] NAME     SIZE  MODIFIED │  SIDE PANE / PREVIEW       │
├──────────────────────────────┴─────────────────────────────┤
│ status · key hints                                          │
└─────────────────────────────────────────────────────────────┘
```

Split mode (`\`): two tab panes side by side; preview hidden.

### 9.2 Keybindings checklist

| Key | Action | Checklist |
|-----|--------|-----------|
| `w`/`k`/`↑` · `s`/`j`/`↓` | Move | ✅ |
| `a`/`←` · `d`/`l`/`→` | Parent / enter dir | ✅ |
| `Enter` | System open | ✅ |
| `e` | `$EDITOR` | ✅ |
| `h`/`Home` · `G` | Root · home | ✅ |
| `/` · `g` · `?` | Filter · go-to · help | ✅ |
| `o` · `.` · `t` · `P` · `r` | Sort · hidden · theme · preview · refresh | ✅ |
| `F2` · `x`/`Delete` · `y` | Rename · delete · copy path(s) | ✅ |
| `Space`/`U` | Mark / clear | ✅ |
| `b` · `1`–`9` · `u`/`i` | Bookmarks · history | ✅ |
| `N`/`W` · `[`/`]` · `\` · `Tab` | Tabs · split · focus | ✅ |
| `c`/`m` | Pane copy/move (split) | ✅ |
| `q`/`Esc` | Quit (Esc cancels modes) | ✅ |

### 9.3 Modes checklist

| Mode | Enter | Esc | Checklist |
|------|-------|-----|-----------|
| Active | — | Quit | ✅ |
| Filter | `/` | Clear + Active | ✅ |
| Rename | `F2` | Active | ✅ |
| GoTo | `g` | Clear + Active | ✅ |
| Help | `?` | Active (`q` too) | ✅ |
| Confirm | `x`/`Delete` | Active (`n`/`q`); `y` applies | ✅ |

### 9.4 File operations checklist

| Op | Behavior | Checklist |
|----|----------|-----------|
| Rename | Validate separators / collisions | ✅ |
| Delete | Confirm; trash or permanent; supports marks | ✅ |
| Copy path | Selection or marked set via `arboard` | ✅ |
| Bookmarks | Cap 9, persisted | ✅ |
| Pane copy/move | Requires split | ✅ |
| System / editor open | `Enter` / `e` | ✅ |

---

## 10. Preview & side pane

| Behavior | Checklist |
|----------|-----------|
| Folder listing in side pane | ✅ |
| Text prefix ~64 KiB | ✅ |
| Image summary + common dims | ✅ |
| Generation discard for stale preview | ✅ |
| `P` cycles OnMove → Always → Never (persisted) | ✅ |
| Spotter / plugin previewers / kitty·sixel | ⬜ Out of scope |

---

## 11. Configuration

- **Path:** `~/.r_tvui/.config.toml` (auto-created)
- **Persisted:** theme, sort, trash, preview, hidden, bookmarks

```toml
theme = "Forest"

[settings]
sort = "Name"
enable_trash = true
preview = "OnMove"
show_hidden = false

[cache]
bookmarks = []
```

---

## 12. Quality attributes

| Attribute | Approach | Checklist |
|-----------|----------|-----------|
| Cold start | Single binary | ✅ |
| Large dirs | Async + 50k cap + cache | ✅ |
| Memory | Cache clear on refresh/sort/hidden | ✅ |
| Accessibility | `Mono` theme | ✅ |

---

## 13. Security

| Item | Checklist |
|------|-----------|
| Local FS only; no network service | ✅ |
| Open only on explicit Enter / `e` | ✅ |
| Rename path-separator / `..` / collision checks | ✅ |
| Tagged CI release builds | ✅ |
| Path jail | ⬜ Not required for 3.1.0 |

---

## 14. Testing & CI

### Gate (required)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

| Item | Checklist |
|------|-----------|
| `make fmt` = check; `make lint` = `-D warnings` | ✅ |
| `.github/workflows/ci.yml` on push/PR | ✅ |
| `.github/workflows/release.yml` on `v*` tags | ✅ |
| Unit tests (`src/tests/`, `crates/core`) | ✅ (~123 cases) |

---

## 15. Distribution & releases

| Method | Checklist |
|--------|-----------|
| `scripts/install.sh` → `~/.local/bin` | ✅ |
| GitHub Releases matrix (darwin/linux/windows) | ✅ |
| Versioned + `latest` asset aliases | ✅ |
| Cargo package metadata (crates.io-ready) | ✅ |
| crates.io publish | ⬜ Optional / not required to close v3 |

### Maintainer tag flow

1. Cargo versions = tag without `v` (e.g. `3.1.0` ↔ `v3.1.0`).
2. Tests + clippy green.
3. **Never reuse** historical `v2.0.0` / `v3.0.0`.
4. Push tag → release workflow → verify assets → smoke `install.sh`.

---

## 16. Accepted limitations

These are **closed as accepted** for 3.1.0 (not open checklist debt):

1. In-flight listings are discarded by generation, not aborted.
2. One Tokio runtime per app instance.
3. Image preview is metadata-only (no terminal graphics protocols).
4. `OnMove` and `Always` behave similarly; `Never` fully disables the pane.
5. No plugin / spotter API (M4).
6. No remote VFS.

---

## 17. Risks (closed)

| Risk | Mitigation in 3.1.0 | Status |
|------|---------------------|--------|
| Terminal image fragmentation | Metadata summary only | ✅ Accepted |
| Feature creep vs Yazi | M0–M3 scope freeze | ✅ Closed |
| Async races | Generation tokens + tests | ✅ Mitigated |
| Doc/tag overclaim | Design + review closed at 3.1.0 | ✅ Closed |

---

## 18. Glossary

| Term | Meaning |
|------|---------|
| **BrowserPane** | Per-tab cwd, list, marks, git, history, previewer |
| **Side pane** | Right column: folder / text / image summary |
| **Generation** | Counter to ignore stale async results |
| **AsyncEventClient** | Tokio + channel for listings/previews |
| **Previewer** | Side-pane content model |

---

*Design checklist closed for **3.1.0 / v3**. Do not reopen M0–M3 rows; future work starts a new phase document or a dated addendum.*
