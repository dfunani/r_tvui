# R-TVUI — review (v2.0.0)

Review date: 2026-05-21  
Scope: `src/`, `crates/core`, `crates/filesystem`, release CI, user docs.

## Summary

R-TVUI is a keyboard-driven TUI file browser with split-pane preview, bookmarks, filter/sort, trash delete, and system default app integration. **v2** moves directory listing and side-pane reads off the UI thread, caches repeated folder listings, and opens files in the background so the terminal stays responsive on large trees.

## Architecture

```
src/
  main.rs          CLI entry (clap)
  models/          App state, SidePane, AppMode
  events/          Poll-based input loop + listing drain
  ui/render.rs     ratatui layout
  utils/
    listing.rs     Tokio runtime, DirectoriesCache, mpsc events
    navigation.rs  Async refresh, generation tokens (stale discard)
    browser.rs     Async side-pane folder/preview requests
    opener.rs      Background system open
crates/
  core/            File, paths, formatters
  filesystem/      sync + async list_directories, sort, 50k cap
```

Single binary; no plugin host.

## What works well

- Clear module split: models / events / ui / utils
- Filesystem logic isolated in `filesystem` crate
- **Non-blocking listing** — browser and side-pane use `list_directories_async` via a dedicated Tokio runtime
- **Directory cache** — `DirectoriesCache` (TTL + LRU) shared across browser and side-pane reads
- **Stale-result guard** — `browser_listing_gen` / `side_pane_gen` drop outdated async results after fast navigation
- **Background open** — macOS/Linux/Windows `open` no longer blocks the TUI loop
- Config, themes, bookmarks, release CI, panic hook, 20 tests, clippy clean

## Changes in v2.0.0

| Item | Action |
|------|--------|
| Sync listing on UI thread | Replaced with async requests + 16 ms event poll loop |
| Unused `DirectoriesCache` | Wired through `ListingService` |
| macOS `open` blocked UI | `open_in_background` via `thread::spawn` |
| Double listing on preview | Side-pane hits cache when cwd was listed recently |
| Manual refresh (`r`) | Invalidates cache entry for cwd then reloads |
| Sort / hidden toggle | Clears full cache before reload |
| Delete / rename | Clears cache, async refresh |

## Known limitations

1. **No explicit cancel** — In-flight listing tasks still complete; results are discarded via generation counters (acceptable; cheap when cached).
2. **One Tokio runtime per app** — Fine for a TUI; no shared global executor.
3. **Core types ahead of UI** — `GitStatus`, `DisplayPath`, ID newtypes reserved for later.
4. **Large single directories** — Still capped at 50k entries with partial flag; async avoids freeze but very large reads remain heavy.

## Recommended next steps (post-v2)

| Priority | Task |
|----------|------|
| Medium | Abort/cancel in-flight listing tasks on navigation (tokio `JoinHandle`) |
| Low | Integration tests with `tempfile` for rename/delete |
| Low | Re-export fewer items from `filesystem` until needed |
| Low | Tabs / git status (see `docs/planning/`) |

## Release checklist

- [x] `cargo test --workspace` and `clippy -D warnings` green
- [ ] Tag `v2.0.0` after CI assets appear on Releases
- [ ] Set `RELEASE_VERSION=2.0.0` on the website (`r_tvui_web`)
- [ ] Verify `/releases/latest/download/r_tvui-aarch64-apple-darwin.tar.gz` returns 200

## Standards

- Rust 2024 edition, `clippy -D warnings` in CI
- Minimal public surface on workspace crates
- Silent no-op when no associated app for a file type
- Docs describe what ships; planning docs in `docs/planning/`
