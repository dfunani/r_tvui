# R-TVUI — Implementation Review

**Reviewed:** Jul 18, 2026 (updated after H1–H3 + M4-A)  
**Package version:** 0.1.0 (`Cargo.toml`) — `design.md` aligned to 0.1.0; git tags already include `v2.0.0` / `v3.0.0` on *older* trees; `git describe` on HEAD ≈ `v1.1.2-6-g…`  
**Scope:** Full re-review of master against `docs/design.md`, `docs/tutorial.md`, and prior `review.md` findings. Build/tests re-run this pass.

> **Milestone IDs in this file** (`M1`–`M5`, `C*`, `N*`, …) are **review tracking IDs**, not `design.md` phases. Design’s M2 = file ops ≈ this review’s **M4**. Design’s M3/M4 (power / plugins) are out of scope here.

## Method & build status

- Re-read `src/`, `crates/core/`, `docs/design.md`, `docs/tutorial.md`, `README.md`, `makefile`, `.github/workflows/`, `scripts/install.sh`.
- `cargo build --workspace` — **OK**
- `cargo clippy --workspace --all-targets -- -D warnings` — **OK** (clean)
- `cargo test --workspace --all-targets` — **OK** after H1–H3 + M4-A.

Code-structure liberties (match vs if, WASD vs vim bindings, module shapes) are treated as fine where intent is preserved.

---

## Status of previously-reported items

| Item | Status | Notes |
|------|--------|-------|
| C1 — mode switches discarded | **FIXED** | `dispatch_key` / `app_loop` assign `app.state`; break only on `Quit`. |
| Panic #1 — Enter on empty dir | **FIXED** | Guarded in `handle_key_event_enter_mode`. |
| Panic #2 — preview read `unwrap` | **FIXED** | `read_text_prefix` uses `?`. |
| Panic #3 — render re-reads dir `unwrap` | **FIXED** | Preview consumes `app.previewer`. |
| Panic #4 — `save_config().unwrap()` | **FIXED** | `unwrap_or_default()`. |
| M1 — async listing / cache / generation | **FIXED** | `AsyncEventClient` + generation guards + path cache. |
| M2 — filter | **FIXED** | `/` live input, `entries_filtered`, Esc clears. |
| M3 — themes | **FIXED** | `t` cycles + persists; `Palette` applied in UI. |
| M5 — unused config fields | **DONE** (except preview polish) | `sort`, `show_hidden`, `preview==Never`, `enable_trash`, **`bookmarks`** wired. No `P` cycle / Always vs OnMove. |
| N2 — Enter on a directory | **FIXED** | Enter opens files / enters dirs. |
| P1 — side pane re-reads each frame | **FIXED** | Async + cached preview. |
| P2 — `entries_cache` stale after `cd` | **FIXED** | Refreshed on async reload path. |
| HIGH — Esc in GoTo/Help/Confirm quits | **FIXED** | Outer cancel → `Active`; GoTo uses Esc-only so `q` is path input. |
| H3 — GoTo / Help stubs | **FIXED** | Path jump + help overlay ship. |
| M4-A — delete / Confirm / trash | **FIXED** | `x`/`Delete` → Confirm; `y`/`n`; `trash` crate when `enable_trash`. |
| M4-A2 — copy path | **FIXED** | `y` in normal mode + `arboard`; status on success/failure. |
| M4-B — bookmarks | **FIXED** | `b` saves cwd (cap 9); `1`–`9` jump; persisted. |
| D5 — `scripts/install.sh` missing | **FIXED** | Script present; still references missing `docs/INSTALL.md` and wrong config path in comments/errors. |
| D7 — status hint garbled | **FIXED** | Status bar lists live bindings; mode prompts including delete confirm. |
| D8 — README understates UX | **FIXED** | README key table updated. |

---

## Resolved this round

- **H1–H3** — mode-exit, GoTo, Help.
- **M4-A** — delete with confirm + `enable_trash` + `trash` crate (`x`/`Delete`, not `d`).
- **design.md / README** — delete marked shipping.

---

## HIGH — none open

---

## MAJOR — M4 file operations (remaining)

**Implemented:**
- **Rename (`F2`)**, **Delete**, **Copy path (`y`)**, **Bookmarks (`b` / `1`–`9`)**, **Refresh / sort / hidden**
- **GoTo (`g`)**, **Help (`?`)**, **Filter (`/`)**

**Not implemented:**
- **History (`u` / `i`)** — absent (**M4-C**).

Next: **M4-C** history.

---

## MAJOR — navigation deviation

### N1. `h` / `Home` jumps to filesystem root, not `$HOME`

`scroll_home` pops every path component → `/`. Design §9.2 wants `G` → home. `dirs::home_dir()` is already used by config. Tests encode root walk (`home_key_walks_toward_root`) — change code **and** tests together.

Suggested binding: keep `a`/Left = parent; map `G` (or a dedicated key) to `$HOME`; decide whether `h`/`Home` means parent, root, or home and document it once.

---

## MODERATE — correctness

- **MO1. Selection indexes two lists.** `scroll_forward` / `handle_key_event_enter_mode` use `app.artifacts`; selection/preview use `entries_filtered`. Safe today because filter mode blocks nav; latent wrong-file open if filter+nav ever combine. Prefer `entries_filtered` everywhere.
- **MO2. `App::reload` (sync)** — live path is `async_reload` / `refresh`. Keep test-only or delete.
- **MO3. `open_file` swallows errors** — spawn failures silent (`os/mod.rs`). Optional: set `status_message`.
- **MO4. `partial` listing flag** — core sets it at 50k cap; UI never surfaces it.
- **MO5. Async list errors → empty list** — `unwrap_or_default()` in `client.rs` can look like an empty directory; prefer a status error (sync reload already sets one).
- **MO6. `Preview::Always` vs `OnMove`** — only `Never` changes behavior; no `P` key to cycle. Part of finishing M5.

---

## DOCS / CI / DISTRIBUTION

| ID | Status | Notes |
|----|--------|-------|
| D1 | **Open** | Workflows still `.github.yml` / `.release.yml` — GitHub Actions ignores hidden workflow filenames. Rename to `ci.yml` / `release.yml`. |
| D2 | **Open** | `make fmt` mutates; `make lint` lacks `-D warnings`; design §14.1 wants `fmt --check` + clippy deny. |
| D3 | **Open** | `crates/filesystem` documented; code folded into `crates/core`. |
| D4 | **Open** | Config path/schema: design `~/.config/rtvui/…` + `RTVUI_CONFIG`; actual `~/.r_tvui/.config.toml`, nested tables, PascalCase enums. |
| D5 | **Partial** | `install.sh` exists; references missing `docs/INSTALL.md`; config path in script comments/errors still wrong. |
| D6 | **Open** | Design theme `gotyme`; code has Forest/Midnight/Solar/Mono only. |
| D7 | **Fixed** | Status hints + mode prompts updated with H3. |
| D8 | **Fixed** | README key table matches shipping UX. |
| D9 | **New** | Version story broken for release: Cargo `0.1.0`, design “2.0.0 shipping”, tags `v2`/`v3` on different trees. Do not retag `v2` from this tip. |
| D10 | **Open** | No `CHANGELOG`; `tempfile` in `[dependencies]` (test-only). `shellexpand` now used by GoTo. |

---

## MINOR / nits

- `src/cli/utils.rs` — `return Ok(parent.canonicalize()?)` → `return parent.canonicalize();`.
- `src/events/utils.rs` — `if selection > 0` around `saturating_sub` is redundant.
- Filter mode has no status-bar prompt (unlike rename).
- Inner Confirm/Help Esc arms are dead (outer already handles Esc) — fine, or delete for clarity.

---

## Prioritized recommendations

1. ~~**H1 / H2 — unred the suite & finish mode-exit**~~ **Done.**
2. ~~**H3 — hide or finish GoTo/Help**~~ **Done** (finished).
3. **M4 (remaining)** — slices:  
   - ~~**(A)** delete + Confirm + `enable_trash` + `trash` crate~~ **Done** (`x`/`Delete`)  
   - ~~**(A2)** copy-path `y` + `arboard`~~ **Done**  
   - ~~**(B)** bookmarks `b` / `1`–`9`~~ **Done**  
   - **(C)** history `u` / `i`  
   - **(D)** optional M5 polish: `P` cycles `OnMove` → `Always` → `Never`
4. **N1** — `$HOME` jump; reconcile `h`/`Home`/`G` with docs + tests.
5. **Release hygiene (D1–D6, D9–D10)** — rename workflows, `fmt --check`, clippy `-D warnings`, version/CHANGELOG.

---

## How to implement remaining M5 (and why it’s not “next” alone)

**Review M5** = wire every `AppConfig` field to real behavior.

| Field | Today | Done? |
|-------|--------|-------|
| `settings.sort` | `o` → `cycle_sort` + refresh + save | Yes |
| `settings.show_hidden` | `.` toggle + refresh + save | Yes |
| `settings.preview` | `Never` skips side pane; `OnMove`/`Always` identical; no key | Partial |
| `settings.enable_trash` | `x`/`Delete` confirm → trash or permanent | Yes |
| `cache.bookmarks` | `b` / `1`–`9` | Yes |

**Recommended close-out for M5 leftovers (do with M4, not before H1):**

1. **Preview polish (small, pure M5)**  
   - Bind `P` to cycle `OnMove → Always → Never` (or toggle Never).  
   - Persist via `save_config`.  
   - Optional: `Always` = refresh preview even when selection unchanged / on tick; `OnMove` = only on selection change (current behavior).  
   - Tests: cycle persists; `Never` keeps `Previewer::Empty`.

2. ~~**`enable_trash` (M4-A)**~~ **Done** — `trash` crate; `x`/`Delete`; Confirm `y`/`n`; permanent path when `enable_trash = false`.

3. ~~**Bookmarks (M4-B)**~~ **Done** — `b` cap 9; `1`–`9` jump; persist; missing path status.

**M4-A / A2 / B done.** Next: history (C) or preview polish (D).
