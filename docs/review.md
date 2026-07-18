# R-TVUI — Implementation Review

**Reviewed:** Jun 19, 2026  
**Package version:** 0.1.0 (`Cargo.toml`) — note: `design.md` claims v2.0.0 "shipping"  
**Scope:** Full codebase re-reviewed against `docs/design.md` and `docs/tutorial.md`, after fixes for C1–C4, M1–M3, M5, N2, and partial M4.

## Method & build status

- Read every file under `src/` and `crates/core/`, plus `docs/design.md` and `docs/tutorial.md`.
- `cargo build --workspace` — **OK** (no warnings)
- `cargo test --workspace` — **OK** (5 core + 4 app tests pass)
- `cargo clippy --workspace --all-targets` — **OK** (clean)

Code-structure liberties (match vs if, WASD vs vim bindings, module shapes) are treated as fine where intent is preserved.

---

## Status of previously-reported items

| Item | Status | Notes |
|------|--------|-------|
| C1 — mode switches discarded | **FIXED** | `app_loop` assigns `app.state` and breaks only on `Quit` (`events/app.rs:25-50`). |
| Panic #1 — Enter on empty dir | **FIXED** | Guarded in `handle_key_event_enter_mode`. |
| Panic #2 — preview read `unwrap` | **FIXED** | Moved to `read_text_prefix` using `?` (`models/previewer.rs:24-30`). |
| Panic #3 — render re-reads dir `unwrap` | **FIXED** | Preview now consumes precomputed `app.previewer`; no disk I/O on the render path. |
| Panic #4 — `save_config().unwrap()` | **FIXED** | `unwrap_or_default()` (`config/utils.rs:21`). |
| M1 — async listing / cache / generation | **FIXED** | `AsyncEventClient` drained in the loop; generation guards + per-path cache in `App`. |
| M2 — filter | **FIXED** | `/` enters filter, live input, renders `entries_filtered`, Esc clears. |
| M3 — themes | **FIXED** | `t` cycles + persists; `Palette` applied to highlight/header/borders/bars. |
| M5 — unused config fields | **PARTIAL** | `sort`, `preview`, `show_hidden` wired; `enable_trash` + `bookmarks` deferred to M4. |
| N2 — Enter on a directory | **FIXED** | Enter now opens files / enters dirs (`events/key.rs:87-88`). |
| P1 — side pane re-reads disk each frame | **FIXED** | Preview is async + cached. |
| P2 — `entries_cache` stale after `cd` | **FIXED** | Refreshed in `apply_async_event` / `async_reload`. |

---

## Resolved this round

- **B1 — test suite compiles again.** The core test fixtures now pass the new `sort` field; `cargo test --workspace` is green (9 tests).
- **W1 — unused import removed** (`events/key.rs`); `cargo build`/`clippy` are warning-free.

---

## HIGH — mode-exit contract is wrong (regression risk)

Entering **GoTo** (`g`) or **Help** (`?`) and then pressing `Esc`/`q` **quits the entire app**, because the outer dispatchers return `Quit`:

```52:63:src/events/keys.rs
pub fn handle_key_events_go_to_mode(app: &mut App, event_key: KeyEvent) -> Result<AppState> {
    match event_key.code {
        KeyCode::Esc | KeyCode::Char('q') => Ok(AppState::Quit),
        _ => handle_key_event_go_to_mode(app, event_key),
    }
}
```

`go_to`, `confirm`, and `help` all do this (`keys.rs:47,54,61`). The inner handlers for confirm/help actually return `Active` (`key.rs:70,79`), but the outer dispatcher shadows them, so that code is dead. `Filter` and `Rename` get it right (only `Esc` intercepted, returns `Active`). Make the contract uniform: `Esc` should return `Active` for every transient mode; only normal mode's `q`/`Esc` should quit.

---

## MAJOR — M4 file operations (partial)

**Implemented:**
- **Rename (`F2`)** — `begin_rename` seeds the buffer, `Rename` mode handles input (`key.rs:31-47`), `commit_rename` validates separators/`..`/collisions and calls `std::fs::rename` then `refresh()` (`app.rs:287-315`). Buffer shown in the status bar (`layout.rs:22-23`). Solid.
- **Refresh (`r`)**, **sort cycle (`o`)**, **hidden toggle (`.`)** — all call `refresh()` which clears the (option-keyed-stale) cache and re-lists. Sort/hidden persist to config.

**Not implemented (and their states are traps or no-ops):**
- **Delete / trash** — no key sets `AppState::Confirm`, so the entire confirm flow is unreachable, and `Settings.enable_trash` still has no consumer. The `trash` crate is not in `Cargo.toml`.
- **Copy path (`y`)** — absent; `arboard` not a dependency.
- **Bookmarks (`b`, `1`–`9`)** — absent; `cache.bookmarks` still unread/unwritten.
- **History (`u`/`i`)** — absent.
- **GoTo (`g`)** — reachable but `handle_key_event_go_to_mode` does nothing (no path input, no UI, no jump). Effectively a dead state that can only quit.
- **Help (`?`)** — reachable but `render` has no Help branch, so the screen is unchanged; it's a no-op state that can only quit.

---

## MAJOR — navigation deviation

### N1. `h` / `Home` jumps to the filesystem root, not `$HOME`
```36:43:src/events/utils.rs
pub fn scroll_home(app: &mut App) -> Result<()> {
    for _ in 0..app.current_working_directory.components().count() {
        app.current_working_directory.pop();
    }
    app.scroll_state.select(Some(0));
    app.async_reload()?;
    Ok(())
}
```
Design intends a jump to the home directory (`G`). This walks all the way to `/`. There is still no jump-to-`$HOME` (`dirs::home_dir()` is available and already used by config).

---

## MODERATE — correctness

- **MO1. Selection indexes two different lists.** `scroll_forward` / `handle_key_event_enter_mode` index `app.artifacts` (`events/utils.rs:49,64`), while `selected_artifact` / `is_file_artifact` index `app.entries_filtered` (`app.rs:271-274`). These are equal only while no filter is active (the only time those nav keys run today), so it's currently safe — but it's a latent bug if navigation is ever allowed from a filtered view. Pick one list (prefer `entries_filtered`, the rendered one).
- **MO2. `App::reload` (sync) appears dead** outside tests — the live path is `async_reload`/`refresh`. Either remove it or keep it test-only and note that.
- **MO3. `open_file` swallows all errors** (`os/mod.rs:8-16`) — silent per design §9.4, but spawn failures vanish with no status feedback.

---

## DOCS / CI / DISTRIBUTION mismatches (carried over — re-confirm before release)

- **D1. CI workflows are dotfiles** (`.github/workflows/.github.yml`, `.release.yml`) — GitHub ignores hidden files in `workflows/`. Rename to `ci.yml` / `release.yml`.
- **D2. CI doesn't gate quality** — `make fmt` mutates instead of `--check`; `make lint` lacks `-D warnings` (contradicts design §14.1; would catch W1); `make clean` between lint and test wastes the cache.
- **D3. `crates/filesystem` doesn't exist** — design §5.1/§7/§8 describe it; folded into `crates/core`. Members are only `[".", "crates/core"]`.
- **D4. Config schema mismatch** — design §11 documents `~/.config/rtvui/config.toml` with flat keys and an `RTVUI_CONFIG` override; actual is `~/.r_tvui/.config.toml`, nested tables, capitalized enum variants, no env override. The documented TOML wouldn't deserialize.
- **D5. `scripts/install.sh` is missing** — README/`tutorial` Phase 10 reference it.
- **D6. Theme list mismatch** — design lists `gotyme, midnight, forest, solar, mono`; code has `Forest, Midnight, Solar, Mono` (no `gotyme`).
- **D7. Status hint string is garbled** — `" w/s ↑↓ · a/d ⇆ · d enter· q quit "` (`ui/layout.rs:20`): `d` appears twice, missing space before `q`, and omits the new `r`/`o`/`.`/`t`/`/`/`F2` bindings.

---

## MINOR / clippy

- `crates/core/src/lib.rs` — `ArtifactListResult` now uses `#[derive(Default)]` (prior manual-impl note resolved).
- `src/cli/utils.rs:23` — `return Ok(parent.canonicalize()?)` can be `return parent.canonicalize();`.
- `src/events/utils.rs:10` — the `if selection > 0` guard around `saturating_sub` is redundant.

---

## Prioritized recommendations

1. **Fix the mode-exit contract:** `Esc` returns `Active` for GoTo/Confirm/Help (not `Quit`) — pressing `g` or `?` then `Esc` currently kills the app.
2. **Finish or hide the half-states:** either implement GoTo (path input + jump) and a Help screen, or don't transition into them yet.
3. **Continue M4 in slices:** (A) delete/trash via the Confirm flow + `enable_trash` + `trash` crate, and copy-path `y` + `arboard`; (B) bookmarks `b`/`1`–`9` (consumes `cache.bookmarks`); (C) history `u`/`i`.
4. **N1:** make `G` jump to `$HOME`; keep parent on `a`/Left.
5. **Reconcile docs/CI** (D1–D7): workflow filenames, `fmt --check`, clippy `-D warnings`, config path/schema, install script, theme names, and the status-hint string.
