# lazyglab

A terminal UI (TUI) for GitLab and GitHub — browse merge requests and pipeline statuses without leaving your terminal.

Built with [ratatui](https://ratatui.rs) and Rust. Mouse-first interaction.

## Features

- **Merge requests** — list MRs by project with open / merged / closed / all filters
- **Pipelines** — live pipeline status with configurable auto-reload
- **Project favorites** — bookmark projects via the Find modal; favorites persist across sessions
- **12 color themes** — One Dark, Catppuccin, Tokyo Night, Dracula, Nord, Gruvbox, Solarized, and more; live preview while browsing
- **GitLab API** — authenticated via personal access token (env var or config file)
- **Mouse support** — every interactive element is clickable

## Installation

Requires Rust 1.75+.

```bash
git clone https://github.com/ragamo/lazyglab
cd lazyglab
cargo build --release
./target/release/lazyglab
```

## Configuration

Config is stored at `~/Library/Application Support/lazyglab/config.toml` (macOS) or `~/.config/lazyglab/config.toml` (Linux).

```toml
[auth]
token = "glpat-xxxxxxxxxxxxxxxxxxxx"

[gitlab]
base_url = "https://gitlab.com"   # optional, defaults to gitlab.com

[ui]
theme = "one_dark"
```

The token can also be provided via environment variable — `LAZYGLAB_TOKEN` takes precedence over the config file:

```bash
LAZYGLAB_TOKEN=glpat-xxx lazyglab
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch between merge requests / pipelines |
| `1` / `2` | Jump to merge requests / pipelines tab |
| `←` / `→` | Cycle MR filters (open, merged, closed, all) |
| `p` | Open project selector |
| `f` | Open project search (Find) |
| `r` | Manual refresh |
| `,` | Open settings |
| `q` / `Esc` | Quit |

Mouse clicks work on all interactive elements: tabs, filters, project selector, find results, autoreload checkbox, logout, and settings.

## Architecture

```
src/
├── app.rs          # Application state machine and event dispatch
├── auth.rs         # Token resolution (env var → config → modal)
├── config/         # TOML config load/save
├── event.rs        # Async event loop (crossterm + tokio::select!)
├── provider/       # Provider trait + GitLab implementation
│   ├── mod.rs      # Provider trait and error types
│   ├── types.rs    # Shared domain types (MergeRequest, Pipeline, ...)
│   └── gitlab.rs   # GitLab REST API client
├── theme.rs        # Color palette definitions (12 themes)
└── ui/             # Ratatui widgets
    ├── main_view.rs    # Main layout: header, tabs, content, footer
    ├── auth_modal.rs   # Token input modal
    ├── find_modal.rs   # Project search modal
    └── settings_modal.rs # Theme selector
```

The provider layer is designed to be extensible — a GitHub connector can be added by implementing the `Provider` trait.
