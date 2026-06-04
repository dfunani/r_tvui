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
| `/` | Enter filter mode *(input not implemented yet)* |
| `g` | Enter go-to-path mode *(path input not implemented yet)* |
| `?` | Enter help mode *(overlay not implemented yet)* |
| `q` or `Esc` | Quit |

> **Note:** `d` enters folders; `Enter` opens files (and other types) externally. This differs from some vim-style explorers that use `l` / `Enter` only for navigation.

### Filter / go-to / help modes

These modes are wired in the event loop but still minimal. In all of them today:

| Key | Action |
|-----|--------|
| `Esc` or `q` | Quit the app |

Filter does not yet accept typed input; go-to does not yet jump to a path; help does not yet show a key reference panel.

### Planned (not bound yet)

| Key | Planned action |
|-----|----------------|
| `j` / `k` | Alternative move up/down |
| `l` | Enter directory |
| `t` | Cycle theme |
| `P` | Toggle preview-on-move |
| `y` | Copy path |
| Sort, delete, bookmarks, history | See [tutorial.md](docs/tutorial.md) |

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

Theme cycling and preview settings are stored in config but not yet applied from keybindings in the UI.

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
