# R-TVUI — Implementation Review

**Reviewed:** Jul 18, 2026 (close-out after M4-C / M5 / hygiene)  
**Package version:** 0.2.0 (`Cargo.toml`)  
**Scope:** Remaining review items closed in one pass.

> **Milestone IDs** (`M1`–`M5`, …) are review tracking IDs, not `design.md` phases.

## Method & build status

- `cargo test --workspace --all-targets` — **OK** (98 app + 21 core)
- `cargo clippy --workspace --all-targets -- -D warnings` — **OK**

---

## Status of previously-reported items

| Item | Status | Notes |
|------|--------|-------|
| M1–M3, H1–H3, M4-A/A2/B | **FIXED** | Prior work |
| M4-C — history `u`/`i` | **FIXED** | Canonical path stack + back/forward |
| M5 — preview `P` | **FIXED** | Cycles OnMove → Always → Never; persists |
| N1 — `$HOME` | **FIXED** | `G` → home; `h`/`Home` → filesystem root |
| MO1 — filtered selection | **FIXED** | `scroll_forward` / enter use `entries_filtered` |
| MO3 — open feedback | **FIXED** | Status `Opening {name}` |
| MO4 — partial listing | **FIXED** | Status when truncated |
| MO5 — async list errors | **FIXED** | `BrowserDone.error` → status |
| D1 — CI filenames | **FIXED** | `ci.yml` / `release.yml` |
| D2 — quality gates | **FIXED** | `make fmt` = check; `make lint` = `-D warnings` |
| D5 — INSTALL.md | **FIXED** | `docs/INSTALL.md` + install.sh pointer |
| D9 / D10 | **FIXED** | Version 0.2.0; CHANGELOG; tempfile → dev-deps |

Open / deferred (non-blocking):

| Item | Notes |
|------|-------|
| D3 | Docs already describe `crates/core` only |
| D4 | Actual config path documented in design/README |
| D6 | Themes are Forest/Midnight/Solar/Mono (no gotyme) |
| MO2 | Sync `reload` kept for tests |
| M3/M4 design phases | Power features / plugins still planned |

---

## Shipping on 0.2.0

Browse, filter, rename, delete+trash, copy path, bookmarks, history, go-to, help, themes/sort/hidden, preview modes, `$HOME` / root jumps, async listing with error + truncation feedback.

---

## Prioritized recommendations (remaining)

1. ~~M4 / M5 feature gaps~~ **Done for review scope.**
2. Optional: integration E2E / TTY tests; `JoinHandle` cancel for listings.
3. Tag release as **`v0.2.0`** (do not reuse `v2.0.0` / `v3.0.0`).
