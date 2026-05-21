# Distributing R-TVUI — install on any machine

**Users installing R-TVUI:** use **[INSTALL.md](./INSTALL.md)** (GitHub Releases, tar/zip — no package manager setup).

This guide is for **maintainers**: how to **build**, **package**, and **ship** releases via CI.

**Repository:** https://github.com/dfunani/r_tvui

---

## What end users need

| Requirement | Notes |
|-------------|--------|
| A terminal | iTerm2, Terminal.app, Alacritty, WezTerm, Windows Terminal, GNOME Terminal, etc. |
| True-color support (recommended) | Themes use 24-bit RGB. Most modern terminals support this. |
| macOS | `open` (built in) — used to launch files with the default app |
| Linux | `xdg-open` — usually from package `xdg-utils` |
| Windows | `cmd start` — built in |

No database, Docker, or GUI toolkit is required.

**Config (created automatically):**

- macOS / Linux: `~/.config/rtvui/config.toml`
- Override: `RTVUI_CONFIG=/path/to/config.toml`

---

## What users download (published on each tag)

CI attaches these assets to [GitHub Releases](https://github.com/dfunani/r_tvui/releases):

| Asset | Platform |
|-------|----------|
| `r_tvui-<version>-aarch64-apple-darwin.tar.gz` | macOS Apple Silicon |
| `r_tvui-<version>-x86_64-apple-darwin.tar.gz` | macOS Intel |
| `r_tvui-<version>-x86_64-unknown-linux-gnu.tar.gz` | Linux |
| `r_tvui-<version>-x86_64-pc-windows-msvc.zip` | Windows |

Copy-paste install commands: **[INSTALL.md](./INSTALL.md)**.

<details>
<summary>Example install snippets (reference)</summary>

Once you publish [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository), users can:

### macOS (Apple Silicon)

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/latest/download/r_tvui-aarch64-apple-darwin.tar.gz
tar xzf r_tvui-aarch64-apple-darwin.tar.gz
chmod +x r_tvui
sudo mv r_tvui /usr/local/bin/   # optional: install globally
r_tvui
```

### macOS (Intel)

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/latest/download/r_tvui-x86_64-apple-darwin.tar.gz
tar xzf r_tvui-x86_64-apple-darwin.tar.gz
chmod +x r_tvui
mv r_tvui ~/.local/bin/   # ensure ~/.local/bin is on PATH
r_tvui
```

### Linux (x86_64)

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/latest/download/r_tvui-x86_64-unknown-linux-gnu.tar.gz
tar xzf r_tvui-x86_64-unknown-linux-gnu.tar.gz
chmod +x r_tvui
mv r_tvui ~/.local/bin/
r_tvui
```

### Windows (PowerShell)

```powershell
# Download r_tvui-x86_64-pc-windows-msvc.zip from Releases, extract, then:
.\r_tvui.exe
# Or add the folder to PATH via System Settings → Environment Variables
```

</details>

---

## Building release binaries (maintainer)

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### 2. Build optimized binary (native)

From the repo root:

```bash
cargo build --release
```

Binary location:

| OS | Path |
|----|------|
| macOS / Linux | `target/release/r_tvui` |
| Windows | `target\release\r_tvui.exe` |

Test it:

```bash
./target/release/r_tvui
```

### 3. Build for other platforms (cross-compile)

Install targets you need:

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
```

**Examples (run from repo root on a machine with the right linker):**

```bash
# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# Linux x86_64 (often built on Linux CI)
cargo build --release --target x86_64-unknown-linux-gnu

# Windows x86_64 (cross-compile from Linux with mingw, or build on Windows)
cargo build --release --target x86_64-pc-windows-msvc
```

**Tip:** The most reliable approach is **GitHub Actions** — one job per OS/target (see below), so you do not need cross-linkers on your laptop.

### 4. Package archives for download

Naming convention (Rust target triple in the filename):

```text
r_tvui-<version>-<target>.tar.gz   # macOS / Linux
r_tvui-<version>-<target>.zip      # Windows
```

Example packaging script (macOS/Linux):

```bash
VERSION=0.1.0
TARGET=aarch64-apple-darwin
BIN=target/${TARGET}/release/r_tvui

mkdir -p dist
cp "$BIN" dist/r_tvui
cp docs/DISTRIBUTION.md dist/README.txt
tar -czvf "dist/r_tvui-${VERSION}-${TARGET}.tar.gz" -C dist r_tvui README.txt
```

For Windows, zip `r_tvui.exe` plus a short README.

### 5. Publish to GitHub Releases

1. Tag a version: `git tag v0.1.0 && git push origin v0.1.0`
2. Open **GitHub → Releases → Draft a new release**
3. Attach all `r_tvui-*` archives
4. Paste release notes (features, keybindings, terminal requirements)

Users download from the **Assets** section — no Rust required.

---

## GitHub Actions (automated releases)

Add `.github/workflows/release.yml` to build on every tag:

```yaml
name: Release

on:
  push:
    tags: ["v*"]

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
            archive: tar.gz
          - target: x86_64-apple-darwin
            os: macos-latest
            archive: tar.gz
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Package (Unix)
        if: matrix.archive == 'tar.gz'
        run: |
          VERSION=${GITHUB_REF_NAME#v}
          mkdir dist && cp target/${{ matrix.target }}/release/r_tvui dist/
          tar -czvf r_tvui-${VERSION}-${{ matrix.target }}.tar.gz -C dist r_tvui
      - name: Package (Windows)
        if: matrix.archive == 'zip'
        shell: pwsh
        run: |
          $VERSION = $env:GITHUB_REF_NAME -replace '^v',''
          mkdir dist
          cp target/${{ matrix.target }}/release/r_tvui.exe dist/
          Compress-Archive -Path dist/* -DestinationPath r_tvui-${VERSION}-${{ matrix.target }}.zip
      - uses: softprops/action-gh-release@v2
        with:
          files: r_tvui-*.*
```

After merging, pushing `v0.1.0` creates release assets automatically.

---

## Release assets (automatic)

On each `v*` tag push, CI builds tarballs/zip for all platforms. Users install via **[INSTALL.md](./INSTALL.md)** — no Homebrew tap or third-party repo required.

Optional later: [crates.io](https://crates.io/) (`cargo install r_tvui`) for developers.

---

## Versioning checklist

Before each release:

- [ ] Bump `version` in root `Cargo.toml` (workspace `0.1.0`)
- [ ] Run `cargo test --workspace`
- [ ] Run `cargo build --release` on at least one machine
- [ ] Tag `v0.1.0` and push
- [ ] Upload or CI-publish binaries
- [ ] Update release notes (keybindings, themes, requirements)

---

## User guide (include in release notes)

**Run:**

```bash
r_tvui          # opens in current directory
cd ~/Projects && r_tvui
```

**Keys:**

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `l` / Enter | Open folder / open file with system app |
| `h` | Parent · `g` go to path · `G` home · `u`/`i` history |
| `/` | Filter · `s` sort · `.` hidden · `?` help |
| `y` | Copy path · `F2` rename · `d` delete (trash) |
| `b` / `1`–`9` | Bookmarks |
| `t` | Cycle theme (saved to config) |
| `q` | Quit |

**Themes** persist in `~/.config/rtvui/config.toml`:

```toml
theme = "forest"
```

Valid values: `gotyme`, `midnight`, `forest`, `solar`, `mono`.

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `command not found: r_tvui` | Add install directory to `PATH` (`~/.local/bin`, `/usr/local/bin`) |
| Colors look wrong | Use a true-color terminal; try theme `mono` |
| Enter does not open files (Linux) | Install `xdg-utils` (`apt install xdg-utils`) |
| Permission denied | `chmod +x r_tvui` |
| macOS “unidentified developer” | `xattr -d com.apple.quarantine r_tvui` or sign the binary (see below) |

### Code signing (macOS, optional)

For wider trust without quarantine warnings:

```bash
codesign --force --sign - target/release/r_tvui
```

For distribution outside the Mac App Store, notarization with an Apple Developer ID is optional and more involved.

---

## Security notes

- R-TVUI only reads directories the user can read and opens files the user explicitly activates.
- It does not run as a network service.
- Release binaries should be built in CI from tagged sources so checksums match public git tags.
- Publish SHA256 checksums alongside each release asset for verification (optional `shasum -a 256` in CI).

---

## Summary

| Goal | Approach |
|------|----------|
| **Downloadable** | GitHub Releases with per-OS `.tar.gz` / `.zip` |
| **Usable anywhere** | Static `r_tvui` binary + normal terminal |
| **No Rust for users** | Prebuilt artifacts only |
| **Maintainable** | `cargo build --release` + tag + CI matrix |
| **Settings follow user** | `~/.config/rtvui/config.toml` |

Start with **native `cargo build --release`**, then add the **GitHub Actions** workflow when you are ready to publish `v0.1.0`.
