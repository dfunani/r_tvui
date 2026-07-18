# R-TVUI

A keyboard-driven terminal file explorer in Rust. Browse directories in a split view: file list on the left, folder listing or text preview on the right.

## Install (easiest)

**macOS / Linux** — one command (installs to `~/.local/bin`):

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_tvui/master/scripts/install.sh | bash
```

**All platforms** — download from **[GitHub Releases](https://github.com/dfunani/r_tvui/releases)** or see **[tutorial.md](docs/tutorial.md)** Phase 10 (manual tar/zip, Windows, build from source).

| Platform | What to download |
|----------|------------------|
| macOS | `r_tvui-*-apple-darwin.tar.gz` |
| Linux | `r_tvui-*-linux-gnu.tar.gz` |
| Windows | `r_tvui-*-windows-msvc.zip` |

## Run

```bash
# current directory
cargo run --release

# specific path
cargo run --release -- ~/Projects

# installed binary
r_tvui ~/Downloads
r_tvui --version
```

**Makefile shortcuts:** `make run`, `make test`, `make lint`, `make help`

## UI layout

| Area | Content |
|------|---------|
| Top | Current path |
| Left (60%) | Files and folders (`NAME`, `SIZE`, `MODIFIED`) |
| Right (40%) | Selected **folder** → children; selected **file** → text preview (first 64 KiB) |
| Bottom | Status hints |

Directories are listed with a trailing `/`. The selected row is highlighted (reversed + cyan).

## Keybindings

### Normal mode (default)

| Key | Action |
|-----|--------|
| `w` or `↑` | Move selection up |
| `s` or `↓` | Move selection down |
| `a` or `←` | Parent directory (up one level) |
| `d` | Enter selected **directory** |
| `Enter` | Open selected item with the system default app (`open` / `xdg-open` / Windows `start`) |
| `h` or `Home` | Jump to filesystem root (`/`) |
| `/` | Filter listing by name (type to narrow; Esc clears) |
| `g` | Go to path (type path, Enter jumps; `~` supported) |
| `?` | Help overlay |
| `t` | Cycle theme (saved) |
| `o` | Cycle sort (name / size / modified) |
| `.` | Toggle hidden files |
| `r` | Refresh listing |
| `F2` | Rename selected entry |
| `x` or `Delete` | Delete selected entry (confirm; trash if enabled) |
| `q` or `Esc` | Quit |

> **Note:** `d` enters folders; `Enter` opens files (and other types) externally. This differs from some vim-style explorers that use `l` / `Enter` only for navigation. Delete uses `x`/`Delete` because `d` is already enter-dir.

### Filter / go-to / rename / help / confirm modes

| Mode | Cancel | Notes |
|------|--------|-------|
| Filter (`/`) | `Esc` | Typed chars (including `q`) filter the list |
| Go to (`g`) | `Esc` | Enter jumps to an existing directory; `q` is path input |
| Rename (`F2`) | `Esc` | Enter commits |
| Help (`?`) | `Esc` or `q` | Overlay with key reference |
| Confirm delete (`x`) | `Esc` / `q` / `n` | `y` deletes (trash or permanent per config) |

### Planned (not bound yet)

| Key | Planned action |
|-----|----------------|
| `j` / `k` | Alternative move up/down |
| `l` | Enter directory |
| `G` | Jump to `$HOME` |
| `P` | Cycle preview mode |
| `y` | Copy path |
| Bookmarks, history | See [design.md](docs/design.md) / [tutorial.md](docs/tutorial.md) |

## Configuration

On first run, config is created at:

```text
~/.r_tvui/.config.toml
```

Example fields (TOML):

| Field | Values | Default |
|-------|--------|---------|
| `theme` | `Forest`, `Midnight`, `Solar`, `Mono` | `Forest` |
| `settings.sort` | `Name`, `Size`, `Modified` | `Name` |
| `settings.enable_trash` | `true` / `false` | `true` |
| `settings.preview` | `OnMove`, `Always`, `Never` | `OnMove` |
| `cache.bookmarks` | list of paths | `[]` |

Theme, sort, hidden, and trash preference persist. Bookmarks are stored but not yet wired to keybindings. Set `enable_trash = false` for permanent deletes.

## Development

Rust workspace:

```text
r_tvui/          # TUI binary (src/)
crates/core/     # Directory listing, Artifact types
```

```bash
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets
```

## Docs

- **[design.md](docs/design.md)** — architecture and phased design
- **[tutorial.md](docs/tutorial.md)** — rebuild from scratch; release & publishing
- **[Project website](https://github.com/dfunani/r_tvui_web)** — Next.js site for the project

## License

MIT
