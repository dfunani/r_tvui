# R-TVUI — Tutorial: rebuild from scratch

This is a **step-by-step, phase-by-phase** guide to building R-TVUI from zero. It assumes you are new to Rust, terminal UIs, and this repo — we explain what to install, **where every file goes**, **how to implement each piece**, and how to verify your work before moving on.

**Companion doc:** [design.md](./design.md) explains architecture and design decisions in one place.

**Rule:** The **last phase is always release** — shipping binaries and install paths so other people can use your app without Rust installed.

---

## Before you start

### What you are building

A **keyboard-driven file explorer** that runs inside your terminal:

- Lists files and folders in the current directory
- Lets you move with `j`/`k`, enter folders with `l`, go up with `h`
- Shows a **side pane** with preview or subfolder listing
- Saves settings to `~/.config/rtvui/config.toml`
- Ships as a single binary for macOS, Linux, and Windows

### What you need on your machine

| Tool | Why | Install |
|------|-----|---------|
| **Rust** | Language + `cargo` build tool | https://rustup.rs — then `rustup update stable` |
| **A terminal** | Where the app runs | iTerm2, Terminal.app, Alacritty, WezTerm, Windows Terminal, GNOME Terminal |
| **Git** | Version control | `git --version` |
| **Editor** | Write code | VS Code, Cursor, or any IDE |

Optional by platform:

- **Linux:** `sudo apt install xdg-utils` (provides `xdg-open` for opening files)
- **macOS:** nothing extra (`open` is built in)
- **Windows:** nothing extra for basic `start` open

Verify Rust:

```bash
rustc --version
cargo --version
```

You should see stable Rust 1.75 or newer.

### How to use this tutorial

1. Complete phases **in order** — each phase builds on the previous.
2. Run the **checkpoint** at the end of every phase; do not skip until it passes.
3. Commit your work with git after each phase (good habit).
4. When stuck, read [design.md](./design.md) for the “why” behind a module.
5. **Copy the code snippets into the file paths shown** — every snippet says where it lives.

---

## Repository layout — how files grow by phase

You do **not** create the full tree on day one. This table shows what exists after each phase:

| After phase | What exists on disk |
|-------------|---------------------|
| **0** | `Cargo.toml`, `src/main.rs` only |
| **1** | Same — everything lives in `src/main.rs` (one file is fine for the spike) |
| **2** | `crates/core/`, `crates/filesystem/`, root workspace `Cargo.toml` |
| **3+** | Split `src/` into modules (see tree below) |

**Target layout** (Phases 3–10 — create files gradually, not all at once):

```text
r_tvui/
├── Cargo.toml                 # workspace root + binary package
├── README.md
├── LICENSE
├── src/
│   ├── main.rs                # ONLY: panic hook, CLI parse, call ratatui::run
│   ├── lib.rs                 # pub mod models; pub mod ui; pub mod events; …
│   ├── models/
│   │   ├── mod.rs             # pub mod app; pub mod mode;
│   │   ├── app.rs             # struct App { cwd, entries, selected, … }
│   │   └── mode.rs            # enum AppMode { Normal, Filter, … }
│   ├── events/
│   │   ├── mod.rs
│   │   ├── app.rs             # main loop: drain events → poll keys → draw
│   │   └── keys.rs            # KeyEvent → action (when to use which mode)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── render.rs          # terminal.draw layout from App snapshot
│   │   └── app.rs             # small widget helpers
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── listing.rs         # ListingService (Phase 7)
│   │   ├── navigation.rs
│   │   ├── config.rs
│   │   ├── previewer.rs
│   │   ├── opener.rs
│   │   └── …
│   └── theme.rs               # ThemeId → ratatui Style (Phase 5)
├── crates/
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs         # File, FileType, formatters
│   └── filesystem/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── cache.rs       # Phase 7
├── docs/
│   ├── design.md
│   └── tutorial.md            # this file
├── scripts/
│   └── install.sh             # Phase 10
└── .github/workflows/
    ├── ci.yml                 # Phase 9
    └── release.yml            # Phase 10
```

**Where does `App` live?**

| Phase | Location | Why |
|-------|----------|-----|
| 1 | `src/main.rs` (top of file) | Fastest way to learn; no module boilerplate |
| 3+ | `src/models/app.rs` | State separate from drawing and input |

**Where does the main loop live?**

| Phase | Location |
|-------|----------|
| 1 | `fn app_loop` in `src/main.rs` |
| 3+ | `src/events/app.rs` — called from `main.rs` via `App::run(terminal)` |

---

## Phase 0 — Empty project & first run

**Goal:** A Rust binary that prints a message and exits. You learn `cargo` basics.

### Folder structure (Phase 0)

```text
r_tvui/
├── Cargo.toml
└── src/
    └── main.rs
```

### Steps

1. Create the project:

```bash
mkdir r_tvui
cd r_tvui
cargo init
```

2. Open **`Cargo.toml`** (project root). You will see `[package]` with `name = "r_tvui"`.

3. Replace **`src/main.rs`** contents with:

```rust
fn main() {
    println!("R-TVUI: hello");
}
```

4. Run:

```bash
cargo run
```

You should see `R-TVUI: hello`.

### Checkpoint

- [ ] `cargo run` succeeds
- [ ] `cargo build --release` creates `target/release/r_tvui` (or `r_tvui.exe` on Windows)

---

## Phase 1 — Terminal UI spike (M0)

**Goal:** Fullscreen TUI that lists **one directory**, navigate with `j`/`k`/`h`/`l`, quit with `q`. No async yet — sync `read_dir` is fine.

**Folder structure (Phase 1):** still only `src/main.rs` + `Cargo.toml`. Put **everything** in `main.rs` for now.

### Concepts (read once)

| Term | Meaning |
|------|---------|
| **TUI** | Text User Interface — draws with characters inside your terminal |
| **ratatui** | Rust library for layouts, lists, borders |
| **Raw mode** | Terminal stops echoing keys line-by-line; your app reads keys directly |
| **Alternate screen** | Separate buffer so your UI does not scroll away shell history |
| **`ratatui::run`** | Sets up terminal, runs your loop, restores terminal on exit |

### Step 1 — Dependencies

Edit **`Cargo.toml`** at the project root:

```toml
[package]
name = "r_tvui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
# crossterm is pulled in by ratatui — you import events through ratatui::crossterm
```

Run `cargo build` once so dependencies download.

### Step 2 — Panic hook (do this first)

If your code panics while raw mode is on, the terminal can look “broken” (no cursor, no echo). Install a hook **before** `ratatui::run`.

Add to **`src/main.rs`**:

```rust
fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}
```

Call `install_panic_hook();` at the very start of `main()`.

### Step 3 — The three parts of `app_loop` (what you are implementing)

The spike has **three responsibilities**. Each maps to a function or block in your code:

```text
app_loop
  │
  ├─ (A) LOAD STATE     read_dir → fill App.entries for App.cwd
  │
  └─ loop {
       (B) DRAW          terminal.draw → List widget, highlight selected row
       (C) INPUT         poll keyboard → j/k/h/l/q update state or break
     }
```

You implement **(A) once at start** and again after `h`/`l` change directory. **(B)** and **(C)** repeat every frame.

---

#### Part (A) — Load directory entries

**What:** Read the current folder from disk into memory.

**Where:** helper function `read_dir_entries(path) -> io::Result<Vec<Entry>>` in `main.rs`.

**How `read_dir` works:**

1. `std::fs::read_dir(path)` returns an iterator over directory entries.
2. Each entry has a `file_name()` and `file_type()` (file vs directory).
3. Store `{ name, is_dir }` in a `Vec`.
4. Sort: directories first, then alphabetical (optional but nicer).

```rust
use std::fs;
use std::io;
use std::path::Path;

/// One row in the file list.
struct Entry {
    name: String,
    is_dir: bool,
}

fn read_dir_entries(path: &Path) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();

    // read_dir can fail (permission denied, path not found)
    for item in fs::read_dir(path)? {
        let item = item?; // ? propagates per-entry errors
        let file_type = item.file_type()?;
        let name = item.file_name().to_string_lossy().into_owned();
        entries.push(Entry {
            name,
            is_dir: file_type.is_dir(),
        });
    }

    // Directories first, then name (case-insensitive)
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}
```

**`App` struct** — holds all mutable state for the spike. Put it in **`src/main.rs`** above `read_dir_entries`:

```rust
use std::path::PathBuf;
use ratatui::widgets::ListState;

struct App {
    /// Current directory we are browsing
    cwd: PathBuf,
    /// Rows shown in the list (reloaded from disk)
    entries: Vec<Entry>,
    /// Index into `entries` (0 = first row)
    selected: usize,
    /// ratatui needs this to know which row is highlighted
    list_state: ListState,
}

impl App {
    /// Start in the process current working directory (where you ran `cargo run`)
    fn new() -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let entries = read_dir_entries(&cwd)?;
        let mut list_state = ListState::default();
        list_state.select(if entries.is_empty() { None } else { Some(0) });

        Ok(Self {
            cwd,
            entries,
            selected: 0,
            list_state,
        })
    }

    /// Call after `cwd` changes (enter dir or go to parent)
    fn reload(&mut self) -> io::Result<()> {
        self.entries = read_dir_entries(&self.cwd)?;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.list_state.select(if self.entries.is_empty() {
            None
        } else {
            Some(self.selected)
        });
        Ok(())
    }
}
```

---

#### Part (B) — Draw the list

**What:** Each loop iteration, paint the UI from current `App` state.

**Where:** function `fn draw(frame: &mut Frame, app: &mut App)` in `main.rs`.

**How ratatui drawing works:**

1. `terminal.draw(|frame| { ... })` gives you a `Frame` with the terminal size.
2. Build a `List` widget from `app.entries`.
3. `frame.render_stateful_widget(list, area, &mut app.list_state)` draws it with selection.

```rust
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

fn draw(frame: &mut Frame, app: &mut App) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| {
            // trailing / for folders is a common convention
            let label = if e.is_dir {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            };
            ListItem::from(label)
        })
        .collect();

    let title = format!(" {}", app.cwd.display());

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::new()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Cyan),
        );

    // frame.area() = full terminal; later phases split into panes
    frame.render_stateful_widget(list, frame.area(), &mut app.list_state);
}
```

---

#### Part (C) — Poll keyboard input

**What:** Wait briefly for a key; update `App` or quit.

**Where:** inside `app_loop`, after each `draw`.

**How crossterm events work:**

1. `event::poll(timeout)` — returns `true` if a key is ready (non-blocking wait).
2. `event::read()` — read the event.
3. Check `key.kind == KeyEventKind::Press` (ignore key release on some platforms).
4. Match `key.code` for `j`, `k`, `h`, `l`, `q`.

| Key | Action |
|-----|--------|
| `j` or Down | `selected += 1` (clamp to last row) |
| `k` or Up | `selected -= 1` (clamp to 0) |
| `l` or Enter | if row is directory: `cwd.push(name)`, `selected = 0`, `reload()` |
| `h` | `cwd.pop()` (go to parent), `selected = 0`, `reload()` |
| `q` | `break` out of loop |

```rust
use std::time::Duration;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

fn app_loop(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::new()?;

    loop {
        // (B) DRAW — always draw first so user sees current state
        terminal.draw(|frame| draw(frame, &mut app))?;

        // (C) INPUT — poll with timeout so we redraw even when idle
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,

                        KeyCode::Char('j') | KeyCode::Down => {
                            if app.selected + 1 < app.entries.len() {
                                app.selected += 1;
                                app.list_state.select(Some(app.selected));
                            }
                        }

                        KeyCode::Char('k') | KeyCode::Up => {
                            if app.selected > 0 {
                                app.selected -= 1;
                                app.list_state.select(Some(app.selected));
                            }
                        }

                        KeyCode::Char('l') | KeyCode::Enter => {
                            if let Some(entry) = app.entries.get(app.selected) {
                                if entry.is_dir {
                                    app.cwd.push(&entry.name);
                                    app.selected = 0;
                                    app.reload()?;
                                }
                                // Phase 8: open files with system app on Enter
                            }
                        }

                        KeyCode::Char('h') => {
                            // pop() removes last path component (= parent directory)
                            if app.cwd.pop() {
                                app.selected = 0;
                                app.reload()?;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
```

**Why `poll` with a timeout?** Without it, `read()` blocks forever until a key arrives — the UI would not redraw on resize or future async updates. Phase 7 uses ~16 ms; 250 ms is fine for the spike.

---

### Step 4 — Wire `main`

**File:** `src/main.rs`

```rust
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    install_panic_hook();
    ratatui::run(app_loop)
}

// … install_panic_hook, App, Entry, read_dir_entries, draw, app_loop …
```

Note: `ratatui::run(app_loop)` works when `app_loop` has signature `fn(&mut DefaultTerminal) -> io::Result<()>`.

### Step 5 — Full `main.rs` checklist

Your Phase 1 file should contain **in order**:

1. `use` statements
2. `Entry` struct
3. `App` struct + `impl App { new, reload }`
4. `read_dir_entries`
5. `draw`
6. `app_loop`
7. `install_panic_hook`
8. `main`

Run:

```bash
cargo run
```

### Common mistakes (Phase 1)

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Terminal broken after crash | No panic hook | Add `install_panic_hook()` |
| Every key fires twice | Ignoring `KeyEventKind::Press` | Wrap handler in `if key.kind == KeyEventKind::Press` |
| List empty but `ls` shows files | Wrong `cwd` | Print `app.cwd` in title bar; start with `std::env::current_dir()` |
| `l` does nothing | `is_dir` false or selection wrong | Check `file_type().is_dir()` in `read_dir_entries` |
| Cannot go up from `/` | `pop()` on root | `if app.cwd.pop()` — no-op at filesystem root |

### Checkpoint

- [ ] App fills the terminal with a file list
- [ ] Title bar shows current path
- [ ] `j`/`k` move highlight; `l` enters folders; `h` goes up
- [ ] `q` quits and your shell prompt looks normal (no broken terminal)
- [ ] Empty directory shows empty list (no panic)

---

## Phase 2 — Workspace crates (`core` + `filesystem`)

**Goal:** Move path types and directory reading into library crates so the binary stays thin.

### Folder structure (Phase 2)

```text
r_tvui/
├── Cargo.toml              # workspace + binary deps
├── src/
│   └── main.rs             # still one file; calls filesystem crate
└── crates/
    ├── core/
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    └── filesystem/
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

### Why split crates?

- **Test filesystem logic** without spinning up a TUI.
- **Reuse** listing code when you add async and cache later.

### Step 1 — Convert root to a workspace

Replace **`Cargo.toml`** (root):

```toml
[workspace]
members = [".", "crates/core", "crates/filesystem"]
resolver = "2"

[package]
name = "r_tvui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.30"
rtvui-core = { path = "crates/core", package = "core" }
filesystem = { path = "crates/filesystem" }
```

### Step 2 — Create `crates/core`

```bash
cargo new --lib crates/core
```

**`crates/core/Cargo.toml`:**

```toml
[package]
name = "core"
version = "0.1.0"
edition = "2021"

[dependencies]
```

**`crates/core/src/lib.rs`** — start with types the TUI will display:

```rust
//! Shared types: files, paths, display formatters.

use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub file_type: FileType,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        0..=KB => format!("{} B", bytes),
        n if n < MB => format!("{:.1} KB", n as f64 / KB as f64),
        n if n < GB => format!("{:.1} MB", n as f64 / MB as f64),
        n => format!("{:.1} GB", n as f64 / GB as f64),
    }
}
```

### Step 3 — Create `crates/filesystem`

```bash
cargo new --lib crates/filesystem
```

**`crates/filesystem/Cargo.toml`:**

```toml
[package]
name = "filesystem"
version = "0.1.0"
edition = "2021"

[dependencies]
core = { path = "../core", package = "core" }
```

**`crates/filesystem/src/lib.rs`:**

```rust
use core::{File, FileType};
use std::fs;
use std::path::Path;

pub struct DirectoryListOptions {
    pub show_hidden: bool,
}

impl Default for DirectoryListOptions {
    fn default() -> Self {
        Self { show_hidden: false }
    }
}

pub struct DirectoryListResult {
    pub entries: Vec<File>,
    /// true when directory had more than 50_000 entries and we truncated
    pub partial: bool,
}

const MAX_ENTRIES: usize = 50_000;

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

pub fn list_directories(path: &Path, opts: &DirectoryListOptions) -> std::io::Result<DirectoryListResult> {
    let mut entries = Vec::new();
    let mut partial = false;

    for item in fs::read_dir(path)? {
        if entries.len() >= MAX_ENTRIES {
            partial = true;
            break;
        }
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if !opts.show_hidden && is_hidden(&name) {
            continue;
        }
        let meta = item.metadata()?;
        let file_type = if meta.is_dir() {
            FileType::Directory
        } else if meta.is_symlink() {
            FileType::Symlink
        } else if meta.is_file() {
            FileType::File
        } else {
            FileType::Other
        };
        entries.push(File {
            name,
            path: item.path(),
            file_type,
            size: meta.len(),
            modified: meta.modified().ok(),
        });
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(DirectoryListResult { entries, partial })
}
```

### Step 4 — Wire the binary

In **`src/main.rs`**, replace `read_dir_entries` with:

```rust
use filesystem::{list_directories, DirectoryListOptions};
use core::FileType;

fn load_entries(cwd: &Path) -> io::Result<Vec<Entry>> {
    let result = list_directories(cwd, &DirectoryListOptions::default())?;
    Ok(result
        .entries
        .into_iter()
        .map(|f| Entry {
            name: f.name,
            is_dir: f.file_type == FileType::Directory,
        })
        .collect())
}
```

Call `load_entries` from `App::new` and `App::reload`.

### Step 5 — Test the crate (no TUI)

**`crates/filesystem/src/lib.rs`** (bottom of file):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lists_files_and_dirs() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"hi").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();

        let result = list_directories(dir.path(), &DirectoryListOptions::default()).unwrap();
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn hides_dotfiles_by_default() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".hidden"), b"").unwrap();
        fs::write(dir.path().join("visible"), b"").unwrap();

        let result = list_directories(dir.path(), &DirectoryListOptions::default()).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].name, "visible");
    }
}
```

Add to **`crates/filesystem/Cargo.toml`**:

```toml
[dev-dependencies]
tempfile = "3"
```

### Checkpoint

- [ ] `cargo test -p filesystem` passes
- [ ] TUI still works using the crate
- [ ] `cargo clippy --workspace` has no warnings you care about

---

## Phase 3 — Layout, status bar, and CLI args

**Goal:** Looks like a real app: path bar, file table columns (name, size, modified), status line, optional start path.

**This is when you split `src/main.rs` into modules.**

### Folder structure (Phase 3)

```text
src/
├── main.rs           # thin entry only
├── lib.rs            # module declarations
├── models/
│   ├── mod.rs
│   └── app.rs        # App struct moves here
├── ui/
│   ├── mod.rs
│   └── render.rs     # draw() moves here
└── events/
    ├── mod.rs
    └── app.rs        # app_loop moves here
```

### Step 1 — Add `clap` for CLI

**`Cargo.toml`:**

```toml
clap = { version = "4", features = ["derive"] }
```

### Step 2 — Create `src/lib.rs`

**`src/lib.rs`:**

```rust
pub mod events;
pub mod models;
pub mod ui;

pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}
```

Tell Cargo this is a library + binary. Add to **`Cargo.toml`**:

```toml
[lib]
path = "src/lib.rs"
```

### Step 3 — Move `App` to `src/models/app.rs`

**`src/models/mod.rs`:**

```rust
pub mod app;
```

**`src/models/app.rs`:**

```rust
use core::File;
use std::path::PathBuf;
use ratatui::widgets::ListState;

pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<File>,
    pub selected: usize,
    pub list_state: ListState,
    pub status_message: String,
}

impl App {
    pub fn new(start_path: PathBuf) -> std::io::Result<Self> {
        // load entries via filesystem crate (same as Phase 2)
        todo!("call list_directories, fill entries")
    }

    pub fn reload(&mut self) -> std::io::Result<()> {
        todo!("reload entries for self.cwd")
    }
}
```

Implement `new` / `reload` using the Phase 2 `filesystem` calls (copy logic from old `main.rs`).

### Step 4 — Move draw to `src/ui/render.rs`

**`src/ui/mod.rs`:** `pub mod render;`

**`src/ui/render.rs`:**

```rust
use crate::models::app::App;
use core::{format_size, FileType};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // path bar
            Constraint::Min(1),    // file table
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    draw_path_bar(frame, chunks[0], app);
    draw_file_table(frame, chunks[1], app);
    draw_status_bar(frame, chunks[2], app);
}

fn draw_path_bar(frame: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::raw(" path: "),
        Span::styled(app.cwd.display().to_string(), Style::new().add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_file_table(frame: &mut Frame, area: Rect, app: &App) {
    let rows: Vec<Row> = app
        .entries
        .iter()
        .map(|f| {
            let name = if f.file_type == FileType::Directory {
                format!("{}/", f.name)
            } else {
                f.name.clone()
            };
            Row::new(vec![
                Cell::from(name),
                Cell::from(format_size(f.size)),
                Cell::from("-"), // add mtime formatting later
            ])
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Percentage(50),
        Constraint::Length(10),
        Constraint::Length(20),
    ])
    .header(Row::new(vec!["NAME", "SIZE", "MODIFIED"]).style(Style::new().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL).title(" Files "));

    // For selection highlight, use TableState similarly to ListState
    frame.render_widget(table, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let hint = " j/k move · l enter · h up · q quit ";
    let text = if app.status_message.is_empty() {
        hint.to_string()
    } else {
        format!(" {} · {} ", app.status_message, hint)
    };
    frame.render_widget(Paragraph::new(text), area);
}
```

Fix the `draw_path_bar` / `frame` reference — pass `frame: &mut Frame` into helpers or use a single `render` function body. Pattern:

```rust
pub fn render(frame: &mut Frame, app: &App) {
    // … split layout …
    frame.render_widget(Paragraph::new(...), chunks[0]);
    // …
}
```

### Step 5 — Move loop to `src/events/app.rs`

**`src/events/mod.rs`:** `pub mod app;`

**`src/events/app.rs`:**

```rust
use crate::models::app::App;
use crate::ui::render;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub fn run(terminal: &mut DefaultTerminal, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render::render(frame, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        // … same j/k/h/l handlers, call app.reload() …
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

### Step 6 — Thin `src/main.rs`

**`src/main.rs`:**

```rust
use clap::Parser;
use r_tvui::{events, install_panic_hook, models::app::App};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "r_tvui", version, about = "Terminal file explorer")]
struct Cli {
    /// Optional directory to open (default: current working directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    install_panic_hook();
    let cli = Cli::parse();

    let start = resolve_start_path(cli.path)?;
    let app = App::new(start)?;

    ratatui::run(|terminal| events::app::run(terminal, app))
}

fn resolve_start_path(arg: Option<PathBuf>) -> std::io::Result<PathBuf> {
    let path = match arg {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    // Expand ~ manually or use shellexpand crate later
    if path.is_dir() {
        Ok(path.canonicalize().unwrap_or(path))
    } else {
        std::env::current_dir()
    }
}
```

Add `r_tvui = { path = ".", package = "r_tvui" }` is not needed — binary crate uses `crate::` via lib.

For binary to use lib modules, in **`main.rs`** use:

```rust
use r_tvui::install_panic_hook;
```

(The package name from `Cargo.toml` is `r_tvui`.)

### Checkpoint

- [ ] `cargo run --release -- ~/Downloads` opens in Downloads
- [ ] Columns show size (and modified time when implemented)
- [ ] Status line shows hints (`j/k move`, `q quit`)
- [ ] `App` lives in `src/models/app.rs`, loop in `src/events/app.rs`, draw in `src/ui/render.rs`

---

## Phase 4 — Input modes: filter, go-to-path, help

**Goal:** `AppMode` enum and key dispatch — not everything happens in one giant `match`.

### New / updated files

```text
src/models/mode.rs       # NEW — AppMode enum
src/events/keys.rs       # NEW — key dispatch by mode
src/models/app.rs        # ADD mode, filter_buffer fields
src/events/app.rs        # CALL keys::handle instead of inline match
```

### Step 1 — Define modes

**`src/models/mod.rs`:** add `pub mod mode;`

**`src/models/mode.rs`:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Filter,
    GoToPath,
    ConfirmDelete,
    Help,
}
```

**Add to `App` in `src/models/app.rs`:**

```rust
use crate::models::mode::AppMode;

pub struct App {
    // … existing fields …
    pub mode: AppMode,
    pub input_buffer: String,
    pub all_entries: Vec<File>,   // full listing from disk
    pub visible_entries: Vec<usize>, // indices into all_entries after filter
}
```

On reload: set `all_entries` from disk, rebuild `visible_entries` as `0..n`.

### Step 2 — Key dispatch

**`src/events/keys.rs`:**

```rust
use crate::models::{app::App, mode::AppMode};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub enum Action {
    Quit,
    MoveDown,
    MoveUp,
    Enter,
    Parent,
    StartFilter,
    StartGoToPath,
    ToggleHelp,
    CancelMode,
    AppendChar(char),
    Backspace,
    SubmitInput,
    Noop,
}

pub fn map_key(app: &App, key: KeyEvent) -> Action {
    use AppMode::*;
    match app.mode {
        Normal => match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('l') | KeyCode::Enter => Action::Enter,
            KeyCode::Char('h') => Action::Parent,
            KeyCode::Char('/') => Action::StartFilter,
            KeyCode::Char('g') => Action::StartGoToPath,
            KeyCode::Char('?') => Action::ToggleHelp,
            _ => Action::Noop,
        },
        Filter | GoToPath => match key.code {
            KeyCode::Esc => Action::CancelMode,
            KeyCode::Enter => Action::SubmitInput,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(c) => Action::AppendChar(c),
            _ => Action::Noop,
        },
        Help => match key.code {
            KeyCode::Esc | KeyCode::Char('?') => Action::CancelMode,
            _ => Action::Noop,
        },
        ConfirmDelete => Action::Noop, // Phase 8
    }
}
```

**`src/events/app.rs`** — replace inline `match key.code` with:

```rust
match keys::map_key(&app, key) {
    Action::Quit => break,
    Action::MoveDown => app.move_down(),
    Action::StartFilter => { app.mode = AppMode::Filter; app.input_buffer.clear(); }
    Action::AppendChar(c) => app.input_buffer.push(c),
    Action::SubmitInput => app.submit_input()?,
    // …
    Action::Noop => {}
}
```

Implement `App::apply_filter` in **`src/models/app.rs`**:

```rust
pub fn apply_filter(&mut self) {
    let needle = self.input_buffer.to_lowercase();
    self.visible_entries = self
        .all_entries
        .iter()
        .enumerate()
        .filter(|(_, f)| f.name.to_lowercase().contains(&needle))
        .map(|(i, _)| i)
        .collect();
    self.selected = 0;
}
```

### Checkpoint

- [ ] `/` filters live as you type
- [ ] `g` then path + Enter jumps directory
- [ ] `?` shows key help; `Esc` dismisses

---

## Phase 5 — Config, themes, and persistence

**Goal:** Settings survive restarts; themes change colors.

### New files

```text
src/utils/config.rs     # load/save TOML
src/utils/mod.rs
src/theme.rs            # ThemeId → ratatui Style
```

**`src/lib.rs`:** add `pub mod utils; pub mod theme;`

### Step 1 — Dependencies

```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
dirs = "5"
```

### Step 2 — Config struct

**`src/utils/config.rs`:**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub sort: String,
    pub use_trash: bool,
    pub preview_on_move: bool,
    pub bookmarks: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "forest".into(),
            sort: "name".into(),
            use_trash: true,
            preview_on_move: true,
            bookmarks: vec![],
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".config/rtvui/config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

pub fn load_or_create() -> std::io::Result<Config> {
    let path = config_path();
    if path.exists() {
        let text = fs::read_to_string(&path)?;
        Ok(toml::from_str(&text).unwrap_or_default())
    } else {
        let cfg = Config::default();
        save(&cfg)?;
        Ok(cfg)
    }
}

pub fn save(cfg: &Config) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, toml::to_string_pretty(cfg).unwrap())?;
    Ok(())
}
```

### Step 3 — Wire into `App::new`

In **`src/models/app.rs`:**

```rust
pub struct App {
    pub config: Config,
    pub theme_id: ThemeId,
    // …
}

impl App {
    pub fn new(start_path: PathBuf) -> std::io::Result<Self> {
        let config = crate::utils::config::load_or_create()?;
        let theme_id = ThemeId::from_name(&config.theme);
        // …
    }

    pub fn cycle_theme(&mut self) {
        self.theme_id = self.theme_id.next();
        self.config.theme = self.theme_id.name().into();
        let _ = crate::utils::config::save(&self.config);
    }
}
```

Bind **`t`** in `keys.rs` → `Action::CycleTheme`.

### Checkpoint

- [ ] First run creates `~/.config/rtvui/config.toml`
- [ ] `t` cycles theme and file on disk updates
- [ ] Restart app — theme persists

---

## Phase 6 — Side pane and text preview

**Goal:** Right pane shows folder contents or first chunk of a text file.

### Update `src/models/app.rs`

```rust
pub enum SidePane {
    Hidden,
    Folder { path: PathBuf, entries: Vec<File> },
    Preview { title: String, body: String },
}

pub struct App {
    // …
    pub side_pane: SidePane,
}
```

### New file: `src/utils/previewer.rs`

```rust
use std::fs::File;
use std::io::Read;
use std::path::Path;

const PREVIEW_BYTES: usize = 64 * 1024;

pub fn read_text_prefix(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; PREVIEW_BYTES];
    let n = file.read(&mut buf)?;
    buf.truncate(n);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}
```

### Update `src/ui/render.rs`

Split middle row horizontally:

```rust
let middle = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
    .split(chunks[1]);

draw_file_table(middle[0], app);
draw_side_pane(middle[1], app);
```

On selection change (when `config.preview_on_move`): call `utils::browser::update_side_pane(app)` from **`src/utils/browser.rs`**.

### Checkpoint

- [ ] Selecting a `.rs` file shows text on the right
- [ ] Selecting a folder shows its children in the side pane
- [ ] `P` disables auto-preview on move

---

## Phase 7 — Async listing, cache, and responsiveness (v2)

**Goal:** Large directories do not freeze the UI; fast navigation does not show stale data.

Read [design.md §6](./design.md#6-runtime--data-flow) while implementing.

### New / updated files

```text
crates/filesystem/src/cache.rs    # DirectoriesCache
src/utils/listing.rs              # ListingService + ListingEvent
src/events/app.rs                 # drain events before poll keys
src/models/app.rs                 # browser_listing_gen, side_pane_gen
```

### Step 1 — Tokio

**Root `Cargo.toml`:**

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "fs", "time"] }
```

### Step 2 — Generation counters on `App`

**`src/models/app.rs`:**

```rust
pub browser_listing_gen: u64,
pub side_pane_gen: u64,

pub fn bump_browser_gen(&mut self) -> u64 {
    self.browser_listing_gen += 1;
    self.browser_listing_gen
}
```

Before each listing request, increment gen and pass it to the service. When an event arrives, **ignore** if `event.gen != app.browser_listing_gen`.

### Step 3 — ListingService sketch

**`src/utils/listing.rs`:**

```rust
pub enum ListingEvent {
    BrowserDone { gen: u64, entries: Arc<DirectoryListResult> },
    SideFolderDone { gen: u64, entries: Arc<DirectoryListResult> },
    SidePreviewDone { gen: u64, title: String, body: String },
}

pub struct ListingService {
    runtime: tokio::runtime::Runtime,
    tx: mpsc::Sender<ListingEvent>,
    rx: mpsc::Receiver<ListingEvent>,
}

impl ListingService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(64);
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            tx,
            rx,
        }
    }

    pub fn request_browser(&self, gen: u64, path: PathBuf) {
        let tx = self.tx.clone();
        self.runtime.spawn(async move {
            let result = list_directories_async(&path, &Default::default()).await;
            let _ = tx.send(ListingEvent::BrowserDone { gen, entries: Arc::new(result) }).await;
        });
    }

    pub fn drain(&mut self) -> impl Iterator<Item = ListingEvent> + '_ {
        std::iter::from_fn(|| self.rx.try_recv().ok())
    }
}
```

### Step 4 — Updated main loop order

**`src/events/app.rs`:**

```rust
loop {
    // 1. Apply async results FIRST
    for event in app.listing.drain() {
        app.handle_listing_event(event);
    }

    // 2. Poll keys (~16 ms for responsiveness)
    if event::poll(Duration::from_millis(16))? {
        // … keys …
    }

    // 3. Draw LAST
    terminal.draw(|frame| render::render(frame, &app))?;
}
```

### Checkpoint

- [ ] Directory with thousands of entries — UI still accepts keys during load
- [ ] Rapid `j`/`k`/`h`/`l` — side pane never shows wrong folder
- [ ] Second visit to same folder feels instant (cache hit)

---

## Phase 8 — File operations and integrations

**Goal:** Delete, rename, open with system app, clipboard, bookmarks, history.

### New files

```text
src/utils/opener.rs
src/utils/ops.rs
src/utils/history.rs
src/utils/filter.rs      # if not already split from app
```

### Opener (background thread)

**`src/utils/opener.rs`:**

```rust
use std::path::Path;
use std::process::Command;
use std::thread;

pub fn open_in_background(path: &Path) {
    let path = path.to_path_buf();
    thread::spawn(move || {
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(&path).spawn();
        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(&path).spawn();
        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd").args(["/C", "start", "", &path.display().to_string()]).spawn();
    });
}
```

Wire **Enter on file** in `App::enter_selection` → `open_in_background(&path)`.

Add dependencies as needed: `trash`, `arboard`.

### Checkpoint

- [ ] Enter on a file opens it in default app
- [ ] Delete with confirm; file lands in trash when enabled
- [ ] Rename works; invalid names show status error
- [ ] `y` copies path; bookmark jump works after restart

---

## Phase 9 — Tests and CI quality gate

**Goal:** Automated checks on every push so regressions are caught early.

### What to test (no real terminal)

| Test | File | Idea |
|------|------|------|
| Config defaults | `src/utils/config.rs` `#[cfg(test)]` | parse empty TOML |
| Filter logic | `src/models/app.rs` tests | `apply_filter` subsets indices |
| Filesystem | `crates/filesystem` | temp dir fixtures |

Example **app filter test** in `src/models/app.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn filter_narrows_visible_entries() {
        // build App with known all_entries, set input_buffer, apply_filter, assert len
    }
}
```

### CI workflow

Create **`.github/workflows/ci.yml`**:

```yaml
name: CI
on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
```

### Checkpoint

- [ ] All tests pass locally
- [ ] CI workflow file committed; green on GitHub after push

---

## Phase 10 — Release & publishing (required final phase)

**Goal:** Anyone in the world can install and run your app **without** installing Rust.

### 10.1 Version and metadata

Set version in **`Cargo.toml`**:

```toml
version = "2.0.0"
```

Ensure **`README.md`** has: one-line description, install command, keybinding table, link to `docs/design.md` and this tutorial.

Add **`LICENSE`** (MIT matches current repo).

### 10.2 Release workflow

Create **`.github/workflows/release.yml`** triggered on tags `v*`:

- Matrix: macOS (arm + intel), Linux x86_64, Linux aarch64, Windows
- `cargo build --release --target <triple>`
- Package `r_tvui` binary + `README.txt` into `r_tvui-<version>-<target>.tar.gz` or `.zip`
- Upload to GitHub Releases with `softprops/action-gh-release`

### 10.3 Cut a release

```bash
git tag v2.0.0
git push origin v2.0.0
```

### 10.4 Install script

Add **`scripts/install.sh`** — detect OS/arch, download tarball, install to `~/.local/bin`.

### Checkpoint (final)

- [ ] GitHub Release has binaries for all target platforms you support
- [ ] Install script works on at least one Mac and one Linux machine
- [ ] A non-developer can run `r_tvui` and browse files

---

## Suggested git milestones

| Tag / branch idea | Delivers |
|-------------------|----------|
| `phase-1-spike` | M0 TUI navigation |
| `phase-2-crates` | core + filesystem |
| `phase-7-async` | ListingService + cache |
| `v1.0.0` | Feature-complete browser |
| `v2.0.0` | Async + performance |
| `vX.Y.Z` | Each public release |

---

## Quick reference — where things live (full app)

| Concern | File |
|---------|------|
| Binary entry, CLI args | `src/main.rs` |
| Panic hook, module exports | `src/lib.rs` |
| App state, side pane, modes | `src/models/app.rs`, `src/models/mode.rs` |
| Main loop (draw / poll / drain) | `src/events/app.rs` |
| Keybindings | `src/events/keys.rs` |
| Layout and widgets | `src/ui/render.rs` |
| Async listing | `src/utils/listing.rs` |
| Config file | `src/utils/config.rs` |
| Directory reads + cache | `crates/filesystem/` |
| Shared types | `crates/core/` |

---

## Where to go next

After Phase 10, you have a **shippable** terminal file explorer. Future work (tabs, visual selection, bulk copy, image preview, plugins) is described in [design.md §4 and §16](./design.md#4-phased-delivery).

If you are **using** R-TVUI rather than building it:

- Install from [GitHub Releases](https://github.com/dfunani/r_tvui/releases) or the install script in the root README
- Config and keys: [design.md §9 and §11](./design.md#9-ux-specification)

---

*This tutorial supersedes scattered install/distribution/planning docs; architecture detail lives in [design.md](./design.md).*
