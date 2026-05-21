# R-TVUI

Terminal file explorer in Rust — keyboard-driven, Finder-style split view, themes, and system default app integration.

## Run from source

```bash
cargo run --release
```

## Install for end users

See **[docs/DISTRIBUTION.md](docs/DISTRIBUTION.md)** for:

- Downloading release binaries (macOS, Linux, Windows)
- Building and packaging `r_tvui` for any machine
- GitHub Actions automated releases
- Config path (`~/.config/rtvui/config.toml`)

## Quick reference

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `l` | Open folder |
| `Enter` | Open folder / open file with default app |
| `h` | Parent |
| `t` | Cycle theme |
| `q` | Quit |

## Docs

- **[Project website](https://github.com/dfunani/r_tvui_web)** (`r_tvui_web` Next.js knowledge base — users, developers, contributors)
- [Distribution & installation](docs/DISTRIBUTION.md)
- [Design spec](docs/DESIGN_SPEC.md)
- [Implementation plan](docs/IMPLEMENTATION_PLAN.md)

### Run the website locally

```bash
cd ../r_tvui_web
npm install && npm run dev
```

## License

MIT
