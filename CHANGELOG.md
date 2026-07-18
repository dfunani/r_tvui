# Changelog

## 3.1.0 — 2026-07-18

v3 power release (design M3). Tag as **`v3.1.0`** — do not reuse historical `v3.0.0`.

### Added
- Multi-tab browsing (`N` / `W` / `[` / `]`, up to 9 tabs)
- Dual-cwd split (`\`) with `Tab` focus swap; pane copy/move (`c` / `m`)
- Visual marks (`Space` / `U`); bulk delete and multi-path clipboard yank
- Vim-style `j` / `k` / `l` navigation aliases (alongside WASD)
- Git status column (`[M]` / `[?]` / …) when inside a work tree
- Image preview summary (type, size, PNG/GIF/JPEG dimensions)
- Open in `$EDITOR` / `$VISUAL` (`e`)
- Real modified timestamps in the file table
- Crates.io-oriented package metadata (`description`, `license`, `repository`, …)

### Changed
- Package version `0.2.0` / release line `v2.1.0` → **`3.1.0`**
- Browser state is per-tab (`BrowserPane`); async listings route by **globally unique** generation counters (no cross-tab collisions)
- `$EDITOR` launch suspends the TUI (leave raw mode / alt screen) and restores it afterwards
- Pane move falls back to copy + delete across filesystems; copy/move **skip existing destinations** and report skips/failures
- Closing a tab drops the split when the peer would alias the active tab
- Help overlay and status hints cover tabs, split, marks, and editor
- CI runs on `master` and `release/**`; release builds use `cargo` directly (no `make` dependency on Windows runners)

### Distribution
- GitHub Release assets via `.github/workflows/release.yml` (matrix tar.gz / zip)
- `scripts/install.sh` installs latest release to `~/.local/bin`
- Config path documented as `~/.r_tvui/.config.toml`

## 0.2.0 — 2026-07-18

Released on GitHub as **v2.1.0**.

### Added
- Go-to path (`g`), help overlay (`?`), delete with confirm (`x`/`Delete`) and optional trash
- Copy path (`y`), bookmarks (`b` / `1`–`9`), navigation history (`u` / `i`)
- Jump to `$HOME` (`G`); preview mode cycle (`P`)
- Status prompts for filter / go-to / rename / confirm / help
- CI workflows renamed to `ci.yml` / `release.yml` (discovered by GitHub Actions)
- `docs/INSTALL.md`

### Changed
- Package version `0.1.0` → `0.2.0`
- Mode Esc/`q` cancel contract (transient modes no longer quit the app)
- Makefile: `fmt` is `--check`; `lint` uses `-D warnings`; `fmt-fix` applies formatting
- `tempfile` moved to dev-dependencies
- Design / README / review docs aligned with shipping behavior

### Fixed
- Selection uses `entries_filtered` for enter / forward navigation
- Async listing errors surface in the status bar
- Truncated listings (50k cap) noted in status when applicable
