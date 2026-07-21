# Contributing to R-TVUI

Thanks for your interest in contributing. This guide covers setup, project conventions, and where to make common kinds of changes.

**Related docs**

| Doc | Purpose |
|-----|---------|
| [docs/design.md](docs/design.md) | Architecture, state model, async flow |
| [docs/test.md](docs/test.md) | Testing layout, coverage, conventions |
| [docs/INSTALL.md](docs/INSTALL.md) | Install from source or releases |
| [README.md](README.md) | User-facing features and keybindings |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

---

## Development setup

**Requirements:** Rust stable (2024 edition), a modern terminal.

```bash
git clone https://github.com/dfunani/r_tvui.git
cd r_tvui
cargo build
cargo run --release -- .
```

### Quality checks

Run before opening a PR:

```bash
make fmt && make lint && make test
```

Equivalent manual commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

---

## Project structure (quick reference)

```text
src/models/app.rs      Tab coordinator, file ops, async event application
src/models/pane.rs     Per-tab state, git marks, marks, history
src/models/client.rs   Async listing/preview tasks
src/events/keys.rs     Dispatch table by AppState
src/events/key.rs      Normal-mode and input-mode handlers
src/events/app.rs      Main loop (do not block here)
src/ui/                ratatui layout and widgets
crates/core/           Filesystem listing — keep UI-free
```

**Rule of thumb:** filesystem and sorting logic belong in `crates/core`; interaction and state belong in `src/models` and `src/events`; drawing belongs in `src/ui`.

---

## Common tasks

### Add a keybinding

1. Decide mode: usually `Active` → `events/key.rs` (`handle_options_keys` or `handle_navigation_keys`), or a dedicated handler in `events/keys.rs` for top-level shortcuts.
2. Implement behavior on `App` or `BrowserPane` in `models/app.rs`.
3. Update `ui/views.rs` (`HELP_LINES`) and [README.md](README.md) keybindings.
4. Add a test in `src/tests/events.rs` (dispatch is testable without a real terminal).

Preserve [UX contracts](docs/design.md#11-ux-contracts) — especially Esc-to-cancel in transient modes.

### Add a file operation

1. Add mutation logic on `App` in `models/app.rs`.
2. Wire a key or mode transition in `events/`.
3. Surface feedback via `status_message`.
4. Call `refresh()` or `async_reload()` if the listing changes.
5. Test with `tempfile` fixtures in `src/tests/app.rs`.

Destructive operations must go through `AppState::Confirm` unless explicitly read-only.

### Add async work

1. Extend `AsyncEvents` in `models/client.rs` if you need a new completion type.
2. Spawn from `AsyncEventClient` using the existing Tokio runtime.
3. Assign a **new** generation from `App::next_generation` or `next_previewer_generation` before sending.
4. Apply in `App::apply_async_event` only when the generation still matches the target pane.

Do not block in `events/app.rs` or `ui/renders.rs`.

### Change listing or sorting

Edit `crates/core` only. Keep the 50k cap behavior and `partial` flag unless deliberately changing product limits. Add tests in `crates/core/src/tests/core.rs`.

### Change themes or config

- Types and defaults: `config/app.rs`
- Persistence: `config/utils.rs`
- Tests: `src/tests/config.rs`

Config path is `~/.r_tvui/.config.toml` — document any new fields in README and design.md.

---

## Code style

- Match existing module layout and naming (`snake_case`, minimal abstraction).
- Run `cargo fmt` before committing.
- Clippy must pass with `-D warnings`.
- Prefer extending existing functions over one-off helpers.
- Comments only for non-obvious invariants (generation matching, terminal suspend around editor, etc.).

---

## Pull requests

1. Branch from `master` (or `release/*` for release-line fixes).
2. Keep PRs focused — one feature or fix per PR when possible.
3. Include tests for behavior changes.
4. Update README and/or `docs/design.md` if user-visible behavior or architecture changes.
5. Ensure CI passes (fmt, clippy, test).

No need to bump `Cargo.toml` version in contributor PRs unless a maintainer is cutting a release.

---

## Releases (maintainers)

1. Bump version in root `Cargo.toml` and `crates/core/Cargo.toml`.
2. Update [CHANGELOG.md](CHANGELOG.md).
3. Merge to `master`, tag `vX.Y.Z`, push tag — `release.yml` builds assets.
4. Verify [download page](https://r-tvui-web.vercel.app/download) and `install.sh` against the new release.

Do not reuse historical tags `v2.0.0` or `v3.0.0` (they point at older trees).

GitHub release body template: [.github/release_template.md](.github/release_template.md).

---

## Getting help

- Architecture questions → [docs/design.md](docs/design.md)
- Test failures → [docs/test.md](docs/test.md)
- Rebuild-from-scratch learning path → [docs/tutorial.md](docs/tutorial.md)

Open an issue or PR on [GitHub](https://github.com/dfunani/r_tvui) with a minimal reproduction when reporting bugs.
