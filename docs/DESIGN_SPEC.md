# R-TVUI — Design specification

**Version:** 0.1 (planning)  
**Status:** Draft

## 1. Problem statement

Power users live in the terminal but default to `ls`, `cd`, and ad-hoc scripts. A **dedicated TUI file explorer** reduces friction: preview before open, batch operations with visual selection, bookmarks, and integrations with search tools—without leaving the shell.

**R-TVUI** targets that workflow in **Rust**, with **Yazi-class** responsiveness and extensibility as the north star.

## 2. Product principles

1. **Never block the UI thread** — directory listing, preview fetch, and search run as cancellable background tasks.
2. **Keyboard-first, mouse-optional** — every action has a default binding; rebinding is data-driven (TOML).
3. **Predictable file operations** — confirm destructive actions; prefer trash integration where the OS supports it.
4. **Extension without core bloat** — previews and “spotters” (quick metadata) are plugins; core stays small.
5. **Terminal realism** — detect capabilities (truecolor, sixel, kitty graphics, unicode width) and degrade gracefully.

## 3. Personas

| Persona | Needs |
|--------|--------|
| **Dev on large repos** | Fast navigation, git status hints, ripgrep integration, ignore rules. |
| **Ops / SRE** | Remote paths (SSH/SFTP later), mount awareness, clear progress for long copies. |
| **Minimalist** | Defaults work out of the box; no Lua required to browse files. |

## 4. High-level architecture

### 4.1 Runtime layers

```text
Terminal (stdin/stdout) ↔ TUI backend (ratatui + crossterm)
    ↔ App state (tabs, panes, selection, cwd stack)
    ↔ Task runtime (async workers, cancellation tokens)
    ↔ VFS abstraction (local fs v0; remote later)
    ↔ Preview / spotter registry (plugins)
    ↔ Config + keymap loader (TOML)
```

### 4.2 Core domains

| Domain | Responsibility |
|--------|----------------|
| **App state** | Tabs, active pane, selection set, filter mode, sort order, history (back/forward). |
| **Directory cache** | Per-path listing with mtime invalidation; cap memory for huge dirs. |
| **Task manager** | Copy/move/delete/search jobs with progress UI and cancel. |
| **Preview pipeline** | Pick previewer by mime/extension; stream partial content; timeout. |
| **Plugin host** | Register previewers, spotters, custom commands; sandbox policy TBD. |
| **Config** | Keymaps, theme, ui options, ignore globs, external tool paths. |

### 4.3 Yazi-inspired features (phased)

| Feature | Phase |
|---------|--------|
| Single-pane browser + status line | MVP |
| Multi-tab | MVP |
| Async listing + preview (text) | MVP |
| Visual selection + bulk ops | v1 |
| Image preview (terminal-dependent) | v1 |
| Split panes (dual cwd) | v1 |
| Plugin API (Rust dylib or WASM) | v1 |
| DDS / cross-instance pub-sub | stretch (evaluate need vs complexity) |
| Virtual FS (remote) | later |

## 5. UX specification (MVP)

### 5.1 Layout (default)

```text
┌─ tabs: ~/proj │ ~/Downloads ─────────────────────────────┐
│ path: ~/proj/src                    [filter] [git: dirty] │
├──────────────────────────────────────┬────────────────────┤
│  NAME          SIZE    MODIFIED       │  PREVIEW           │
│  > src/        -       ...           │  (selected file)   │
│    main.rs     4.2K    ...           │                    │
│    lib.rs      1.1K    ...           │                    │
│  > tests/      -       ...           │                    │
├──────────────────────────────────────┴────────────────────┤
│ 3 selected │ j/k move │ / filter │ ? help │ q quit        │
└───────────────────────────────────────────────────────────┘
```

### 5.2 Default keybindings (Vim-flavored, all rebindable)

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection down / up |
| `h` / `l` | Parent directory / enter directory or open file |
| `g` `g` | Go to path (prompt) |
| `~` | Home directory |
| `-` | Toggle hidden files |
| `/` | Filter listing |
| `Tab` | Next tab |
| `1`–`9` | Jump to tab |
| `Space` | Toggle visual selection |
| `y` / `p` | Yank path / paste (copy/move per mode) |
| `d` `d` | Delete (with confirm; trash if configured) |
| `r` | Rename |
| `!` | Shell command on selection |
| `?` | Help overlay |

### 5.3 File operations

- **Copy / move**: background task queue; show bytes/sec and ETA when available.
- **Delete**: confirm; use `trash` crate or platform API when enabled in config.
- **Rename**: inline editor overlay; validate illegal names before apply.

## 6. Preview system

### 6.1 Pipeline

1. **Spotter** (fast): size, mime guess, binary sniff—no full read.
2. **Previewer** (slow): render content into preview pane (text with syntax highlight optional later).
3. **Timeout + cancel**: new selection cancels in-flight preview.

### 6.2 Built-in previewers (v0 target)

| Type | Behavior |
|------|----------|
| Text | First N KB, UTF-8 lossy fallback |
| Directory | Entry count + size summary |
| Binary | Hex head + “binary file” message |
| Image | Terminal protocol adapter (feature-gated) |

### 6.3 External tool hooks (optional)

- Delegate to `bat`, `chafa`, or user-defined command in config (spawn with timeout).

## 7. Configuration

- **File**: `~/.config/r-tvui/config.toml` (XDG on Linux; platform dirs elsewhere).
- **Sections**: `general`, `keymap`, `theme`, `preview`, `tools` (paths to `rg`, `fd`, `fzf`).
- **Theme**: ratatui style tokens (fg/bg/modifiers); ship 2–3 built-in themes.

## 8. Quality attributes

| Attribute | Target |
|-----------|--------|
| **Cold start** | <100 ms to first frame on typical laptop |
| **Listing 10k entries** | UI responsive; sort/filter off main thread |
| **Memory** | Bounded cache; drop tab → drop listings |
| **Accessibility** | High-contrast theme; optional reduced motion |

## 9. Security

- **Path canonicalization** before operations; reject `..` escapes outside allowed roots when “jail” mode enabled.
- **Plugin sandbox**: no network/filesystem by default for WASM plugins (if adopted).
- **Shell `!`**: explicit user action; log command in status area.

## 10. Risks

| Risk | Mitigation |
|------|------------|
| Terminal preview fragmentation | Capability probe + documented matrix |
| Feature creep vs Yazi | Publish compatibility tier; MVP is browse + preview + ops |
| Plugin ABI churn | Versioned plugin API; semver for host |

## 11. Glossary

- **Pane**: One directory listing view (single or split layout).
- **Tab**: Independent cwd + history stack.
- **Spotter**: Cheap metadata hook before full preview.
- **Previewer**: Renders preview pane content for a file type.
