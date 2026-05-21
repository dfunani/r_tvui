# R-TVUI — Implementation plan

**Version:** 0.1 (planning)  
**Related:** [Design specification](./DESIGN_SPEC.md)

## 0. Milestone overview

| Milestone | Scope | Exit criteria |
|-----------|--------|----------------|
| **M0 — Spike** | TUI frame + read one directory | `r-tvui` opens, lists cwd, quits cleanly |
| **M1 — MVP** | Navigate, tabs, text preview, config | Daily-usable local browser |
| **M2 — v1** | Visual mode, bulk ops, trash, themes | Copy/move/delete with progress |
| **M3 — v1+** | Image preview, splits, tool spawn | Matches “power user” expectations |
| **M4 — Plugins** | Stable previewer/spotter API | Third-party previewer loads |

## 1. Technology stack (proposed)

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Language | Rust (edition 2021+) | Performance, safety, single binary |
| TUI | [ratatui](https://github.com/ratatui/ratatui) + crossterm | Ecosystem standard; Yazi-adjacent patterns |
| Async | tokio | Background I/O, cancellation |
| CLI | clap | Subcommands: `r-tvui`, `r-tvui --version`, `r-tvui doctor` |
| Config | toml + serde | Human-editable keymaps |
| Errors | anyhow (app) / thiserror (libs) | Practical diagnostics |
| Trash | `trash` crate (optional feature) | Cross-platform delete-to-trash |

*Validate ratatui + tokio integration in M0 (some TUI apps use `tokio::select!` with crossterm events).*

## 2. Crate layout (proposed)

```
R-TVUI/
  Cargo.toml                 # workspace
  crates/
    r-tvui_cli/               # entrypoint, clap, logging
    r-tvui_app/               # app state machine, key dispatch
    r-tvui_ui/                # ratatui widgets: browser, preview, status, tabs
    r-tvui_fs/                # async read_dir, metadata, canonical paths
    r-tvui_tasks/             # copy/move progress, cancel
    r-tvui_preview/           # preview pipeline + built-in previewers
    r-tvui_config/            # load/merge TOML, keymap resolution
    r-tvui_terminal/          # capability detection, color depth
    r-tvui_plugin_api/        # traits + versioning (M4)
  tests/
    integration/
  examples/
```

## 3. Workstreams

### 3.1 M0 — TUI spike

**Tasks**

1. `cargo new` workspace; binary `r-tvui`.
2. Raw mode + alternate screen; restore on panic (`color-eyre` or custom hook).
3. Render static layout: path bar + file list from sync `read_dir` (temporary).
4. `j`/`k` selection, `q` quit, `l` enter dir, `h` parent.

**Exit:** manual test on macOS + Linux terminal.

### 3.2 M1 — MVP browser

**Tasks**

1. **Async listing**: `r-tvui_fs::list_dir(path)` → channel → UI updates listing widget.
2. **Tabs**: state vec; `Tab` to cycle; close tab; persist last paths in config optional.
3. **Filter**: `/` opens input line; substring filter on cached names.
4. **Text preview**: read first 64 KiB async; show in right pane.
5. **Config**: default `config.toml` shipped; keymap overrides.
6. **Sort**: name / size / mtime (toggle in status commands).

**Exit:** dogfood one week on real `~/` tree.

### 3.3 M2 — File operations + visual mode

**Tasks**

1. Visual selection (`v`, `V`, `Space`); yank list of paths.
2. **Task queue UI**: bottom line progress; `c` cancel active task.
3. Copy/move with `tokio::fs` + progress aggregation.
4. Delete with confirm; `trash` feature flag.
5. Rename overlay + validation.
6. Second built-in theme; `r-tvui doctor` prints terminal capabilities.

### 3.4 M3 — Power features

**Tasks**

1. **Split pane** (horizontal): two `PaneId`s, independent cwd.
2. **Image preview**: feature `image-preview` — detect kitty/iterm2; fallback message.
3. **External tools**: config `preview.command`, `search.command` spawning `rg`/`fd`.
4. **Git column** (optional): spawn `git status --porcelain` per repo root (cached, debounced).

### 3.5 M4 — Plugin system

**Tasks**

1. Define `Previewer` / `Spotter` traits in `r-tvui_plugin_api`.
2. Dynamic loading: start with **static registration** macro list; then `dlopen` or WASM (spike both).
3. Example plugin crate in `examples/plugins/markdown_preview`.

## 4. Yazi comparison (intentional gaps early)

| Yazi feature | R-TVUI MVP | Notes |
|--------------|-----------|--------|
| Lua plugins | No | Rust plugins later; avoid dual language in M1 |
| DDS pub-sub | No | Revisit if multi-instance needed |
| Package manager for plugins | No | `cargo install` / manual for now |
| Async I/O | Yes | Core requirement |
| Multi-tab | Yes | MVP |
| Image protocols | v1+ | Document supported terminals |

## 5. Testing strategy

| Layer | Approach |
|-------|----------|
| `r-tvui_fs` | Unit tests with tempdir fixtures |
| `r-tvui_config` | Parse golden TOML files |
| UI | Limited: extract state transitions tests without full terminal |
| Integration | `expect` script or `vt100` crate for buffered terminal snapshots (optional) |

## 6. CI

- `cargo fmt --check`, `clippy -D warnings`, `cargo test`.
- Build matrix: `ubuntu-latest`, `macos-latest` (Windows later).
- MSRV pinned in README once chosen (suggest 1.75+).

## 7. Release engineering

- GitHub Releases: static binaries per target triple.
- Man page / `r-tvui --help` as primary docs until website exists.
- `CHANGELOG.md` (Keep a Changelog) from first public tag.

## 8. Immediate next actions (when coding starts)

1. Initialize Cargo workspace + M0 spike branch.
2. Lock default keymap and config schema (version field `config_version = 1`).
3. Implement `list_dir` cache with invalidation policy (document in code).
4. Ship `examples/minimal_config.toml`.

## 9. Definition of Done (per milestone)

- README “Quick start” works on fresh clone.
- No panic on empty dir, permission denied, or broken symlink (graceful error row).
- All new behavior has at least one automated test where feasible.
