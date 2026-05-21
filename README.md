# R-TVUI

Terminal file explorer in Rust — keyboard-driven, Finder-style split view, themes, and system default app integration.

## Install (easiest)

Download from **[GitHub Releases](https://github.com/dfunani/r_tvui/releases)** — see **[docs/INSTALL.md](docs/INSTALL.md)** for copy-paste commands.

| Platform | What to download |
|----------|------------------|
| macOS | `r_tvui-*-apple-darwin.tar.gz` |
| Linux | `r_tvui-*-linux-gnu.tar.gz` |
| Windows | `r_tvui-*-windows-msvc.zip` |

```bash
# Example: macOS Apple Silicon (after a tagged release with CI assets)
curl -fL -O https://github.com/dfunani/r_tvui/releases/latest/download/r_tvui-aarch64-apple-darwin.tar.gz
tar xzf r_tvui-aarch64-apple-darwin.tar.gz && chmod +x r_tvui && r_tvui
```

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
- [Design spec](docs/DESIGN_SPEC.md) · [Implementation plan](docs/IMPLEMENTATION_PLAN.md)

## License

MIT
