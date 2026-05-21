# R-TVUI

**R-TVUI** is a planned **terminal-based file explorer** written in **Rust**: fast keyboard-driven navigation, rich previews, and a composable architecture—taking inspiration from modern TUI file managers like **[Yazi](https://github.com/sxyazi/yazi)** (async I/O, plugin ecosystem, multi-tab UX) while defining its own defaults and extension model.

This repository is in the **design and planning** stage.

## Documentation

- [Design specification](./DESIGN_SPEC.md) — UX, architecture, preview pipeline, plugins, compatibility targets.
- [Implementation plan](./IMPLEMENTATION_PLAN.md) — milestones, crate layout, testing, release strategy.

## Goals (short)

- **Fast** on large directories (async metadata, bounded work, cancellable tasks).
- **Comfortable** for daily use: Vim-style keys (configurable), tabs, selection, trash-aware deletes.
- **Extensible** without forking: previewers, spotters, themes, and keymaps as first-class extension points.
- **Single static binary** — install once, run anywhere your terminal supports.

## Non-goals (initial)

- Full GUI file manager replacement (Finder, Explorer).
- Built-in cloud sync or proprietary backends (optional plugins later).
- Re-implementing every Yazi plugin on day one.

## Inspiration (Yazi and peers)

| Idea from Yazi | R-TVUI direction |
|----------------|-----------------|
| Async, non-blocking I/O | Core event loop never blocks on `stat`/`read_dir` storms. |
| Plugin-heavy UI | Start with **Rust-native plugins** (WASM or dylib TBD); scripting layer later if needed. |
| Rich previews | Pluggable preview pipeline (text, image, archive metadata). |
| Multi-tab / cross-pane | First-class tabs + optional split panes in v1 roadmap. |
| Tool integration | `rg`, `fd`, `fzf`-style pickers via spawn, not reimplementation. |

## License

TBD.
