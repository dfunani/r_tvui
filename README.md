# R-TVUI

Keyboard-driven TUI file explorer in Rust. Browse with WASD or vim keys, preview before you open, mark and move files across a dual-cwd split — all in one binary.

**Version:** 3.1.0 · [**Download**](https://r-tvui-web.vercel.app/download) · [Website](https://r-tvui-web.vercel.app) · [Releases](https://github.com/dfunani/r_tvui/releases/tag/v3.1.0)

## Install

**Easiest:** pick your platform on the [**download page**](https://r-tvui-web.vercel.app/download) — one-liner, direct tarballs/zip, and per-OS steps (macOS, Linux x86_64/ARM64, Windows).

**macOS / Linux** (script → `~/.local/bin`):

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_tvui/master/scripts/install.sh | bash
```

Pin a version: `RTVUI_VERSION=3.1.0 curl -fsSL https://raw.githubusercontent.com/dfunani/r_tvui/master/scripts/install.sh | bash`

**All platforms** — [Download](https://r-tvui-web.vercel.app/download) · [GitHub Releases](https://github.com/dfunani/r_tvui/releases) · [docs/INSTALL.md](docs/INSTALL.md)

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `r_tvui-*-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `r_tvui-*-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `r_tvui-*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `r_tvui-*-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | `r_tvui-*-x86_64-pc-windows-msvc.zip` |

## Run

```bash
r_tvui                 # cwd
r_tvui ~/Projects      # path
r_tvui --version

# from source
cargo run --release -- ~/Projects
```

**Makefile:** `make run` · `make test` · `make lint` · `make help`

## Features (shipped)

- Async directory listing (50k cap + cache) with truncation / error status
- Text preview (64 KiB) · image summary (size + PNG/GIF/JPEG dimensions)
- Filter, sort, themes, hidden toggle, preview modes
- Rename · delete (confirm + optional trash) · copy path(s) · bookmarks · history
- Go-to path · help overlay · `$HOME` / filesystem root jumps
- Multi-tab · dual-cwd split · marks / bulk delete · pane copy/move
- Git porcelain status in the list · open with system app or `$EDITOR`

## UI layout

| Area | Content |
|------|---------|
| Top | Tab index + current path |
| Left (60%) | Files: mark · git · name · size · modified |
| Right (40%) | Folder children, text preview, or image summary |
| Split (`\`) | Two tabs side by side (preview hidden) |
| Bottom | Status / mode prompts |

Directories show a trailing `/`. Marked rows show `*`. Git status appears as `[M]` / `[?]` / … inside a work tree.

## Keybindings

### Normal mode

| Key | Action |
|-----|--------|
| `w` / `k` / `↑` | Selection up |
| `s` / `j` / `↓` | Selection down |
| `a` / `←` | Parent directory |
| `d` / `l` / `→` | Enter directory |
| `Enter` | Open with system default app |
| `e` | Open file in `$EDITOR` / `$VISUAL` |
| `h` / `Home` | Filesystem root |
| `G` | `$HOME` |
| `/` | Filter |
| `g` | Go to path (`~` ok) |
| `?` | Help |
| `t` · `o` · `.` · `P` · `r` | Theme · sort · hidden · preview · refresh |
| `F2` | Rename |
| `Space` / `U` | Mark / clear marks |
| `x` / `Delete` | Delete selection or marked set (confirm) |
| `y` | Copy path(s) to clipboard |
| `b` / `1`–`9` | Bookmark / jump |
| `u` / `i` | History back / forward |
| `N` / `W` | New / close tab |
| `[` / `]` | Prev / next tab |
| `\` | Toggle dual-cwd split |
| `Tab` | Focus other split pane |
| `c` / `m` | Copy / move into other pane (split on) |
| `q` / `Esc` | Quit |

`d`/`l` enter folders; `Enter` opens externally; `e` uses the editor. Delete is `x`/`Delete` (`d` is enter-dir). `h` is root, not vim-left.

### Modes

| Mode | Cancel | Notes |
|------|--------|-------|
| Filter (`/`) | `Esc` | `q` is filter input |
| Go to (`g`) | `Esc` | Enter jumps; `q` is path input |
| Rename (`F2`) | `Esc` | Enter commits |
| Help (`?`) | `Esc` / `q` | Key reference overlay |
| Confirm (`x`) | `Esc` / `q` / `n` | `y` confirms delete |

## Configuration

Created on first run at `~/.r_tvui/.config.toml`:

| Field | Values | Default |
|-------|--------|---------|
| `theme` | `Forest`, `Midnight`, `Solar`, `Mono` | `Forest` |
| `settings.sort` | `Name`, `Size`, `Modified` | `Name` |
| `settings.enable_trash` | `true` / `false` | `true` |
| `settings.preview` | `OnMove`, `Always`, `Never` | `OnMove` |
| `settings.show_hidden` | `true` / `false` | `false` |
| `cache.bookmarks` | list of paths | `[]` |

Theme, sort, hidden, trash, preview, and bookmarks persist from the UI.

## Development

Contributors: start with [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/design.md](docs/design.md).

```text
r_tvui/          # TUI binary (src/)
crates/core/     # Listing + Artifact types
```

```bash
make fmt && make lint && make test
# or
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Documentation

| Doc | Audience |
|-----|----------|
| [**Download (web)**](https://r-tvui-web.vercel.app/download) | Install binaries |
| [Website](https://r-tvui-web.vercel.app) | Usage and overview |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to develop and open PRs |
| [docs/design.md](docs/design.md) | Architecture and runtime model |
| [docs/test.md](docs/test.md) | Testing conventions |
| [docs/INSTALL.md](docs/INSTALL.md) | Install details |
| [docs/tutorial.md](docs/tutorial.md) | Rebuild-from-scratch guide |
| [CHANGELOG.md](CHANGELOG.md) | Release notes |
| [docs/README.md](docs/README.md) | Full doc index |

## License

MIT
