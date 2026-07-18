# Changelog

## 0.2.0 — 2026-07-18

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
