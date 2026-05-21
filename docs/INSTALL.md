# Install R-TVUI

The easiest way to get R-TVUI is from **[GitHub Releases](https://github.com/dfunani/r_tvui/releases)**. No Rust, no Homebrew tap, no build tools required.

Replace `v0.1.0` below with the [latest release](https://github.com/dfunani/r_tvui/releases/latest) tag if a newer version exists.

---

## Pick your platform

| Platform | File on Releases | Install style |
|----------|------------------|---------------|
| macOS (Apple Silicon) | `r_tvui-*-aarch64-apple-darwin.tar.gz` | Extract → run |
| macOS (Intel) | `r_tvui-*-x86_64-apple-darwin.tar.gz` | Extract → run |
| Linux | `r_tvui-*-x86_64-unknown-linux-gnu.tar.gz` | Extract → run |
| Windows | `r_tvui-*-x86_64-pc-windows-msvc.zip` | Extract → run |

---

## macOS (Apple Silicon) — recommended

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/download/v0.1.0/r_tvui-0.1.0-aarch64-apple-darwin.tar.gz
tar xzf r_tvui-0.1.0-aarch64-apple-darwin.tar.gz
chmod +x r_tvui
sudo mv r_tvui /usr/local/bin/   # optional: install globally
r_tvui
```

If macOS blocks the binary:

```bash
xattr -d com.apple.quarantine r_tvui
```

## macOS (Intel)

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/download/v0.1.0/r_tvui-0.1.0-x86_64-apple-darwin.tar.gz
tar xzf r_tvui-0.1.0-x86_64-apple-darwin.tar.gz
chmod +x r_tvui
mv r_tvui ~/.local/bin/   # add ~/.local/bin to PATH if needed
r_tvui
```

---

## Linux

Works on Ubuntu, Debian, Fedora, Arch, etc.:

```bash
curl -LO https://github.com/dfunani/r_tvui/releases/download/v0.1.0/r_tvui-0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar xzf r_tvui-0.1.0-x86_64-unknown-linux-gnu.tar.gz
chmod +x r_tvui
mv r_tvui ~/.local/bin/
r_tvui
```

---

## Windows

1. Open [GitHub Releases](https://github.com/dfunani/r_tvui/releases).
2. Download `r_tvui-0.1.0-x86_64-pc-windows-msvc.zip`.
3. Extract the zip.
4. Run `r_tvui.exe` in PowerShell or Command Prompt.

Optional: add the folder to your PATH (Settings → System → Environment variables).

---

## Developers — build from source

If you have [Rust](https://rustup.rs/) installed:

```bash
git clone https://github.com/dfunani/r_tvui.git
cd r_tvui
cargo run --release
```

Install the binary into `~/.cargo/bin`:

```bash
cargo install --path . --locked
r_tvui
```

---

## After install

**Run** from any directory:

```bash
r_tvui
cd ~/Projects && r_tvui
```

**Config** (themes, etc.) is saved automatically:

```text
~/.config/rtvui/config.toml
```

Example:

```toml
theme = "forest"
sort = "name"          # name | size | modified
use_trash = true
preview_on_move = true
bookmarks = ["/Users/you/Projects", "/Users/you/Downloads"]
```

**Requirements:** a normal terminal app; true-color recommended for themes.

| Key | Action |
|-----|--------|
| `j` / `k` | Move selection |
| `l` / Enter | Open folder / open file with default app |
| `h` | Parent directory |
| `g` | Go to path |
| `G` | Home directory |
| `u` / `i` | History back / forward |
| `/` | Filter by name |
| `s` | Cycle sort (name / size / modified) |
| `.` | Toggle hidden files |
| `p` / `P` | Refresh preview / toggle preview-on-move |
| `y` | Copy path to clipboard |
| `b` | Bookmark folder · `1`–`9` jump to bookmark |
| `F2` | Rename |
| `d` | Delete (confirm; trash if enabled in config) |
| `t` | Cycle theme |
| `?` | Help overlay |
| `q` / `Esc` | Quit (`Esc` clears an active filter first) |

---

## Troubleshooting

| Problem | Fix |
|---------|-----|
| `command not found` | Put `r_tvui` on your PATH or use full path `./r_tvui` |
| Linux: Enter does not open files | `sudo apt install xdg-utils` |
| Wrong or dull colors | Use a modern terminal; try theme `mono` in config |
| Release asset missing | Check [Releases](https://github.com/dfunani/r_tvui/releases) — CI publishes on each `v*` tag |

---

## For maintainers

Building releases and CI: **[DISTRIBUTION.md](./DISTRIBUTION.md)**.
