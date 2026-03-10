# RustyCode

A terminal UI for [OpenCode](https://github.com/sst/opencode) built with [Ratatui](https://ratatui.rs).

![Rust](https://img.shields.io/badge/rust-2021-orange)

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- A running [OpenCode](https://github.com/sst/opencode) server

## Build

```bash
cargo build --release
```

The binary will be at `target/release/rustycode`.

## Run

```bash
# Auto-detect a local OpenCode server (checks ports 4000-4002, 4100)
cargo run --release

# Or specify the server URL
cargo run --release -- --server http://localhost:4000

# Or use the environment variable
OPENCODE_SERVER=http://localhost:4000 cargo run --release
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-s`, `--server <URL>` | OpenCode server URL (env: `OPENCODE_SERVER`) |
| `-d`, `--debug` | Enable debug logging to stderr |
| `-t`, `--theme <NAME>` | Theme name (default: `default`) |

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Ctrl+C` | Quit (confirmation) |
| `Ctrl+N` | New session |
| `Ctrl+O` | Session picker |
| `Ctrl+K` | Model picker |
| `Ctrl+P` | Command palette |
| `Ctrl+T` | Theme picker |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+L` | Toggle logs page |
| `?` | Help |

### Permission Dialog

| Key | Action |
|-----|--------|
| `y` | Allow once |
| `a` | Always allow |
| `n` | Reject |

## License

GPL-3.0 — see [LICENSE](LICENSE) for details.
