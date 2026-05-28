# R-TVUI — Tutorial: rebuild from scratch

This is a **step-by-step, phase-by-phase** guide to building R-TVUI from zero. It assumes you are new to Rust, terminal UIs, and this repo — we explain what to install, what each phase delivers, and how to verify your work before moving on.

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

---

## Phase 0 — Empty project & first run

**Goal:** A Rust binary that prints a message and exits. You learn `cargo` basics.

### Steps

1. Create the project:

```bash
mkdir r_tvui
cd r_tvui
cargo init
```

2. Open `Cargo.toml`. You will see `[package]` with `name = "r_tvui"`.

3. In `src/main.rs`, replace contents with:

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

### Concepts

- **TUI** = Text User Interface (draws with characters, not windows).
- **ratatui** = Rust library for layouts, lists, borders.
- **Raw mode** = terminal stops echoing keys line-by-line; your app reads keys directly.
- **Alternate screen** = separate buffer so your UI does not scroll away shell history.

### Steps

1. Add dependencies in `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.30"
# crossterm is pulled in by ratatui
```

2. Use ratatui’s high-level runner (simplest path):

```rust
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal: &mut DefaultTerminal| app_loop(terminal))
}

fn app_loop(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // 1. read_dir current directory into Vec<(name, is_dir)>
    // 2. loop: draw list with selected index highlighted
    // 3. poll key: j/k move, l enter dir, h parent, q break
    Ok(())
}
```

3. Implement **minimal state**:

```rust
struct App {
    cwd: PathBuf,
    entries: Vec<String>,
    selected: usize,
}
```

4. On **`l`** / Enter: if entry is directory, set `cwd`, reload entries, reset `selected` to 0.
5. On **`h`**: set `cwd` to parent (`cwd.pop()` or `parent()`).
6. Install a **panic hook** so the terminal restores if your code panics:

```rust
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original(info);
    }));
}
```

Call `install_panic_hook()` at the start of `main`.

### Checkpoint

- [ ] App fills the terminal with a file list
- [ ] `j`/`k` move highlight; `l` enters folders; `h` goes up
- [ ] `q` quits and your shell prompt looks normal (no broken terminal)
- [ ] Empty directory does not panic

---

## Phase 2 — Workspace crates (`core` + `filesystem`)

**Goal:** Move path types and directory reading into library crates so the binary stays thin.

### Why split crates?

- **Test filesystem logic** without spinning up a TUI.
- **Reuse** listing code when you add async and cache later.

### Steps

1. Convert to a workspace. Root `Cargo.toml`:

```toml
[workspace]
members = [".", "crates/core", "crates/filesystem"]
resolver = "2"

[package]
name = "r_tvui"
# ...

[dependencies]
rtvui-core = { path = "crates/core", package = "core" }
filesystem = { path = "crates/filesystem" }
```

2. Create `crates/core`:

```bash
cargo new --lib crates/core
```

Add types like `File`, `FileType`, `AbsolutePath`, and helpers `format_size`, `format_mtime`.

3. Create `crates/filesystem`:

```bash
cargo new --lib crates/filesystem
```

Implement:

- `list_directories(path, options) -> DirectoryListResult`
- Sort by name / size / modified
- **Cap at 50_000** entries; set `partial = true` if truncated
- `is_hidden(name)` — leading `.` on Unix

4. Wire the binary: replace inline `read_dir` with `filesystem::list_directories`.

### Checkpoint

- [ ] `cargo test -p filesystem` with a temp directory fixture
- [ ] TUI still works using the crate
- [ ] `cargo clippy --workspace` has no warnings you care about

---

## Phase 3 — Layout, status bar, and CLI args

**Goal:** Looks like a real app: path bar, file table columns (name, size, modified), status line, optional start path.

### Steps

1. Add **clap** for CLI:

```toml
clap = { version = "4", features = ["derive"] }
```

```rust
#[derive(Parser)]
#[command(name = "r_tvui", version, about)]
struct Cli {
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,
}
```

2. Split `src/` modules (create files as you go):

```text
src/lib.rs       pub mod models; pub mod ui; pub mod events; …
src/models/app.rs
src/ui/render.rs
src/events/app.rs
```

3. Draw layout (see [design.md §9.1](./design.md#91-layout-default)):

- Top: current path (truncate if long)
- Middle: split — file list (left), empty right pane for now
- Bottom: status / hints

4. Resolve start path: CLI arg → expand `~` → if missing, use `std::env::current_dir()`.

### Checkpoint

- [ ] `cargo run --release -- ~/Downloads` opens in Downloads
- [ ] Columns show size and modified time
- [ ] Status line shows hints (`j/k move`, `q quit`)

---

## Phase 4 — Input modes: filter, go-to-path, help

**Goal:** `AppMode` enum and key dispatch — not everything happens in one giant `match`.

### Steps

1. Define modes in `src/models/mode.rs`:

```rust
pub enum AppMode {
    Normal,
    Filter,
    GoToPath,
    ConfirmDelete,
    // Rename later
}
```

2. Create `src/events/keys.rs` — map `KeyEvent` → actions only when mode is `Normal`; in `Filter`, keys append to buffer.

3. Implement:

| Key | Mode | Behavior |
|-----|------|----------|
| `/` | Normal → Filter | substring filter on cached names |
| `g` | Normal → GoToPath | type path, Enter navigates |
| `?` | Normal | toggle help overlay |
| `Esc` | Filter/GoToPath | back to Normal, clear buffer |

4. Keep **filtered list** as a view over `all_entries` (do not re-read disk on each keystroke).

### Checkpoint

- [ ] `/` filters live as you type
- [ ] `g` then path + Enter jumps directory
- [ ] `?` shows key help; `Esc` dismisses

---

## Phase 5 — Config, themes, and persistence

**Goal:** Settings survive restarts; themes change colors.

### Steps

1. Add `serde` + `toml`:

```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

2. Config path: `~/.config/rtvui/config.toml` (create dirs on first save).

3. Fields to support:

```toml
theme = "forest"
sort = "name"
use_trash = true
preview_on_move = true
bookmarks = []
```

4. Load on `App::new`, save when theme changes (`t` cycles themes).

5. Add `src/theme.rs` — map `ThemeId` → ratatui `Style` colors (start with 2 themes, grow to five: gotyme, midnight, forest, solar, mono).

### Checkpoint

- [ ] First run creates config file
- [ ] `t` cycles theme and file on disk updates
- [ ] Restart app — theme persists

---

## Phase 6 — Side pane and text preview

**Goal:** Right pane shows folder contents or first chunk of a text file.

### Steps

1. Add `SidePane` enum to `App`:

```rust
pub enum SidePane {
    Hidden,
    Folder { path: AbsolutePath, entries: Vec<File> },
    Preview { title: String, body: String },
}
```

2. On selection change (if `preview_on_move`):

- Directory → request folder listing for side pane
- File → read first ~64 KiB, UTF-8 lossy, show in preview

3. `p` forces refresh; `P` toggles `preview_on_move`.

4. Put preview text logic in `src/utils/previewer.rs`.

### Checkpoint

- [ ] Selecting a `.rs` file shows text on the right
- [ ] Selecting a folder shows its children in the side pane
- [ ] `P` disables auto-preview on move

---

## Phase 7 — Async listing, cache, and responsiveness (v2)

**Goal:** Large directories do not freeze the UI; fast navigation does not show stale data.

This is the biggest architectural step. Read [design.md §6](./design.md#6-runtime--data-flow) while implementing.

### Steps

1. Add **tokio** to workspace `Cargo.toml`:

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "fs", "time"] }
```

2. In `crates/filesystem`, add `list_directories_async` (spawn on Tokio, return same result type).

3. Implement `DirectoriesCache` in `crates/filesystem/src/cache.rs` (TTL + max entries).

4. Create `src/utils/listing.rs` — **`ListingService`**:

- Owns `Runtime::new()`
- `mpsc` channel for `ListingEvent` (browser done, side folder done, side preview done)
- `request_browser(gen, path, opts)` spawns async work
- `drain()` yields completed events in the main loop

5. Add **generation counters** on `App`:

```rust
pub browser_listing_gen: u64,
pub side_pane_gen: u64,
```

Increment before each request; ignore events with old generation.

6. Change main loop to **poll every ~16 ms**:

- Drain all listing events
- Then read keyboard
- Then draw

7. **Invalidate cache** on refresh (`r`), sort toggle, hidden toggle, delete, rename.

### Checkpoint

- [ ] Directory with thousands of entries — UI still accepts keys during load
- [ ] Rapid `j`/`k`/`h`/`l` — side pane never shows wrong folder
- [ ] Second visit to same folder feels instant (cache hit)

---

## Phase 8 — File operations and integrations

**Goal:** Delete, rename, open with system app, clipboard, bookmarks, history.

### Steps

1. **Open file** — `src/utils/opener.rs`:

- macOS: `open path`
- Linux: `xdg-open path`
- Windows: `cmd /C start "" path`
- Run in `thread::spawn` so UI never blocks

2. **Delete** — `d` → `ConfirmDelete` mode → `y` deletes:

- Add `trash` crate; respect `use_trash` in config
- Clear cache; async refresh listing

3. **Rename** — `F2`, edit name in buffer, Enter applies `std::fs::rename`

4. **Clipboard** — `arboard` crate; `y` copies absolute path

5. **History** — `u` / `i` back/forward stack of visited directories

6. **Bookmarks** — `b` saves cwd to config; `1`–`9` jumps

### Checkpoint

- [ ] Enter on a file opens it in default app (TextEdit, VS Code, etc.)
- [ ] Delete with confirm; file lands in trash when enabled
- [ ] Rename works; invalid names show status error
- [ ] `y` copies path; bookmark jump works after restart

---

## Phase 9 — Tests and CI quality gate

**Goal:** Automated checks on every push so regressions are caught early.

### Steps

1. Add unit tests:

- Config parse / default
- Filter logic
- App state transitions (no real terminal — test `App` methods only)

2. Add `src/tests/` or `#[cfg(test)]` modules; run with `cargo test --workspace`.

3. Create `.github/workflows/ci.yml`:

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

4. Run locally before every push:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Checkpoint

- [ ] All tests pass locally
- [ ] CI workflow file committed; green on GitHub after push

---

## Phase 10 — Release & publishing (required final phase)

**Goal:** Anyone in the world can install and run your app **without** installing Rust. You tag versions, CI builds binaries, and document install.

This phase is **mandatory** — do not consider the project “done” until users can download it.

### 10.1 Version and metadata

1. Set version in `Cargo.toml`:

```toml
version = "2.0.0"
```

2. Ensure `README.md` has: one-line description, install command, keybinding table, link to `docs/design.md` and this tutorial.

3. Add `LICENSE` (MIT matches current repo).

### 10.2 Release workflow

Create `.github/workflows/release.yml` triggered on tags `v*`:

- Matrix: macOS (arm + intel), Linux x86_64, Linux aarch64, Windows
- `cargo build --release --target <triple>`
- Package `r_tvui` binary + `README.txt` into `r_tvui-<version>-<target>.tar.gz` or `.zip`
- Upload to GitHub Releases with `softprops/action-gh-release`

See the shipping repo’s `release.yml` for the full matrix (including `aarch64-unknown-linux-gnu` on `ubuntu-24.04-arm`).

### 10.3 Cut a release

```bash
# On main, with clean tree and passing tests
git tag v2.0.0
git push origin v2.0.0
```

Wait for Actions → open **Releases** on GitHub → confirm assets attached.

### 10.4 Install script for macOS / Linux

Add `scripts/install.sh` that:

1. Detects OS/arch (`uname -s`, `uname -m`)
2. Downloads the matching tarball from GitHub Releases
3. Installs to `~/.local/bin` (or `$PREFIX/bin`)
4. Prints “add to PATH” if needed

Users run:

```bash
curl -fsSL https://raw.githubusercontent.com/<you>/r_tvui/master/scripts/install.sh | bash
```

### 10.5 Verify as a new user

On a machine **without** Rust:

1. Download tarball from Releases (or run install script)
2. Extract, `chmod +x r_tvui`, run `./r_tvui`
3. Navigate, open a file, change theme, quit
4. Confirm `~/.config/rtvui/config.toml` appeared

### 10.6 Release checklist (copy for each version)

- [ ] Bump `Cargo.toml` version
- [ ] `cargo test --workspace` and clippy green
- [ ] Update README if keys or platforms changed
- [ ] Tag `vX.Y.Z` and push
- [ ] CI release assets present on GitHub
- [ ] Smoke-test install script on macOS and Linux
- [ ] Optional: update project website / changelog

### 10.7 Troubleshooting for your users

| Problem | Fix |
|---------|-----|
| `command not found` | Add `~/.local/bin` to PATH |
| Linux `exec format error` | Wrong arch — use `uname -m` to pick x86_64 vs aarch64 build |
| macOS quarantine | `xattr -d com.apple.quarantine r_tvui` |
| Enter does not open files (Linux) | `sudo apt install xdg-utils` |
| `/latest/download/...` 404 | Use versioned URL from Releases page until tag exists |

### Checkpoint (final)

- [ ] GitHub Release has binaries for all target platforms you support
- [ ] Install script works on at least one Mac and one Linux machine
- [ ] A non-developer can run `r_tvui` and browse files
- [ ] You can answer “how do I install?” with one URL and one command

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

## Where to go next

After Phase 10, you have a **shippable** terminal file explorer. Future work (tabs, visual selection, bulk copy, image preview, plugins) is described in [design.md §4 and §16](./design.md#4-phased-delivery).

If you are **using** R-TVUI rather than building it:

- Install from [GitHub Releases](https://github.com/dfunani/r_tvui/releases) or the install script in the root README
- Config and keys: [design.md §9 and §11](./design.md#9-ux-specification)

---

*This tutorial supersedes scattered install/distribution/planning docs; implementation detail lives in [design.md](./design.md).*
