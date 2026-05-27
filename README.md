# R-TVUI

Terminal file explorer in Rust — keyboard-driven, Finder-style split view, themes, and system default app integration. **v2** uses async directory listing and a side-pane cache so large folders stay responsive.

## Install (easiest)

**macOS / Linux** — one command (installs to `~/.local/bin`):

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_tvui/main/scripts/install.sh | bash
```

**All platforms** — download from **[GitHub Releases](https://github.com/dfunani/r_tvui/releases)** or see **[docs/INSTALL.md](docs/INSTALL.md)** (manual tar/zip, Windows, build from source).

| Platform | What to download |
|----------|------------------|
| macOS | `r_tvui-*-apple-darwin.tar.gz` |
| Linux | `r_tvui-*-linux-gnu.tar.gz` |
| Windows | `r_tvui-*-windows-msvc.zip` |

## Run from source (developers)

```bash
cargo run --release
```

## Quick reference

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `l` / Enter | Open folder / file |
| `h` | Parent · `g` go to path · `G` home |
| `/` filter · `s` sort · `y` copy path · `d` delete |
| `u` / `i` history · `b` bookmark · `1`–`9` jump |
| `?` help · `t` theme · `q` quit |

```bash
r_tvui ~/Projects    # open a specific directory
r_tvui --version
```

Config: `~/.config/rtvui/config.toml`

## Docs

- **[Install guide](docs/INSTALL.md)** — users
- **[Distribution](docs/DISTRIBUTION.md)** — maintainers (CI, releases)
- **[Project website](https://github.com/dfunani/r_tvui_web)** — `r_tvui_web` Next.js site
- [Design spec](docs/DESIGN_SPEC.md) · [Review](docs/REVIEW.md) · [Planning archive](docs/planning/)

## License

MIT
