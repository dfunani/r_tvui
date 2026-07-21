# Install R-TVUI

**Public download page:** [r-tvui-web.vercel.app/download](https://r-tvui-web.vercel.app/download) — platform picker, one-liner, and versioned asset links for **v3.1.0**.

## One-line (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/dfunani/r_tvui/master/scripts/install.sh | bash
```

Installs the latest GitHub Release binary to `~/.local/bin` (override with `PREFIX`).

| Env | Default | Meaning |
|-----|---------|---------|
| `RTVUI_REPO` | `dfunani/r_tvui` | GitHub repo |
| `RTVUI_VERSION` | latest | Version without `v` prefix |
| `PREFIX` | `$HOME/.local` | Install prefix (`bin/` under this) |

Ensure `~/.local/bin` is on your `PATH`.

## Windows / all platforms

Download from the [website](https://r-tvui-web.vercel.app/download) or [GitHub Releases](https://github.com/dfunani/r_tvui/releases):

| Platform | Asset |
|----------|--------|
| macOS Apple Silicon | `r_tvui-*-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `r_tvui-*-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `r_tvui-*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `r_tvui-*-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | `r_tvui-*-x86_64-pc-windows-msvc.zip` |

## From source

```bash
cargo install --path . --locked
# or
cargo build --release
```

## Config

On first run, config is created at `~/.r_tvui/.config.toml`.
