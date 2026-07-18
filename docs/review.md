# R-TVUI — Implementation Review (closed checklist)

**Closed:** Jul 18, 2026  
**Package:** 3.1.0 (`Cargo.toml`) · **Intended tag:** `v3.1.0`  
**Verdict:** **PASS — ready to ship.** All in-scope review items are closed. This document is an archive checklist, not an open backlog.

> Older review IDs (`M1`–`M5`, `H*`, `D*`, `MO*`) are historical tracking labels — not `design.md` phases.

---

## Close-out gate

| Gate | Result |
|------|--------|
| `cargo test --workspace --all-targets` | ✅ Pass (106 app + 21 core) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Pass |
| `cargo fmt --all -- --check` | ✅ Pass |
| Docs (README / design / review) match 3.1.0 | ✅ Pass |
| Design M0–M3 exit criteria | ✅ Pass |
| Design M4 / remote VFS / kitty·sixel | ⬜ Out of scope (accepted) |

---

## Checklist A — v2.1.0 hygiene (prior line)

| Item | Status |
|------|--------|
| Mode Esc cancel (no quit trap) | ✅ Closed |
| GoTo + Help finished | ✅ Closed |
| Delete + confirm + trash | ✅ Closed |
| Copy path | ✅ Closed |
| Bookmarks | ✅ Closed |
| History `u`/`i` | ✅ Closed |
| Preview `P` cycle | ✅ Closed |
| `$HOME` / root jumps | ✅ Closed |
| Filtered selection / open feedback / list errors | ✅ Closed |
| CI `ci.yml` + `release.yml` | ✅ Closed |
| Makefile quality gates | ✅ Closed |
| INSTALL + CHANGELOG + version line | ✅ Closed |
| GitHub release `v2.1.0` + matrix assets | ✅ Closed |

---

## Checklist B — v3.1.0 power (design M3)

| Item | Status | Evidence |
|------|--------|----------|
| Multi-tab (`N`/`W`/`[`/`]`) | ✅ Closed | `models/app.rs`, keys |
| Dual-cwd split (`\` + `Tab`) | ✅ Closed | `split_tab`, UI split layout |
| Pane copy/move (`c`/`m`) | ✅ Closed | skip-existing + cross-device move fallback |
| Marks / bulk delete (`Space`/`U`/`x`) | ✅ Closed | `BrowserPane.marked` |
| Vim `j`/`k`/`l` | ✅ Closed | `h` remains root |
| Git status column | ✅ Closed | `load_git_marks` |
| Image preview summary | ✅ Closed | dims for PNG/GIF/JPEG |
| `$EDITOR` (`e`) | ✅ Closed | event loop suspends/restores TUI around editor |
| Modified timestamps in table | ✅ Closed | `Artifact::format_modified` |
| Cargo distribution metadata | ✅ Closed | `Cargo.toml` package fields |
| install.sh config path | ✅ Closed | `~/.r_tvui/.config.toml` |

### Final-review fixes (post-audit, pre-tag)

| Finding | Fix |
|---------|-----|
| B1 — `$EDITOR` under live raw mode corrupted the TUI | Editor queued via `pending_editor`; event loop `ratatui::restore()` → run → `ratatui::init()` |
| B2 — per-tab generation collisions misrouted async results | Globally unique `next_generation` / `next_previewer_generation` counters on `App` |
| S1 — cross-device pane move failed silently | `move_path` falls back to copy + delete |
| S2 — close-tab during split could alias panes | Split cleared when peer aliases active or one tab remains |
| S3 — pane copy/move overwrote destinations | Existing destinations skipped; status reports ok/skipped/failed |
| S4 — CI only on `master` | `ci.yml` triggers include `release/**` |
| S5 — `make` on Windows release runner | `release.yml` calls `cargo build --release` directly |

---

## Checklist C — consciously out of scope

These are **not failures**. They were excluded from the 3.1.0 exit criteria.

| Item | Disposition |
|------|-------------|
| Plugin / spotter API (design M4) | Out of scope |
| Kitty / sixel / iTerm image protocols | Out of scope (metadata preview satisfies M3) |
| Remote VFS (SSH/SFTP) | Out of scope |
| `JoinHandle` abort of in-flight listings | Accepted limitation |
| crates.io publish | Optional; metadata ready |
| Reuse tags `v2.0.0` / `v3.0.0` | Forbidden — use `v3.1.0` |

---

## Shipping statement

**3.1.0** delivers the full v2 feature set plus design **M3** power features (tabs, split, marks, git, image summary, editor, vim aliases).

| Line | Cargo | GitHub tag |
|------|-------|------------|
| Prior | `0.2.0` | `v2.1.0` |
| Current | `3.1.0` | `v3.1.0` |

---

*Review checklist **closed**. Do not append open “next” rows here — start a new dated review if a future release needs an audit.*
