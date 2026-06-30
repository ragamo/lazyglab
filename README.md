# lazyglab

A terminal UI (TUI) for GitLab and GitHub — browse merge requests and pipeline statuses without leaving your terminal.

Built with Rust. Mouse-first interaction. Heavily influenced by [herdr](https://herdr.dev/).

## Features

- **Merge requests** — list MRs by project with open / merged / closed / all filters
- **MR detail view** — tabbed panel with Overview, Commits, Pipelines, and Changes
- **Overview** — rendered Markdown description with scrolling
- **Commits** — full commit list for the MR with author and relative time
- **Changes (diff viewer)** — colored unified diffs with old/new line-number gutter, sticky file headers while scrolling, and faint backgrounds on added/removed lines
- **Pipelines** — live pipeline status with configurable auto-reload
- **Pipeline detail** — stages and jobs, including downstream/child pipelines (bridges)
- **Job logs** — view a job's trace, with live tail/refresh while it is still running
- **Resizable panels** — drag to resize the detail panels
- **Project favorites** — bookmark projects via the Find modal (search by name or URL); favorites persist across sessions
- **12 color themes** — One Dark, Catppuccin, Tokyo Night, Dracula, Nord, Gruvbox, Solarized, and more; live preview while browsing
- **Settings** — theme and refresh interval, persisted to config
- **GitLab API** — authenticated via personal access token (env var or config file)
- **Mouse support** — every interactive element is clickable

## Installation

### Homebrew (macOS / Linux)

```bash
brew install ragamo/tap/lazyglab
```

### Install script

Downloads the prebuilt binary for your platform, verifies its checksum, and installs it to `~/.local/bin` — no sudo required:

```bash
curl -fsSL https://raw.githubusercontent.com/ragamo/lazyglab/master/scripts/install.sh | sh
```

Make sure `~/.local/bin` is on your `PATH`. You can override the install location or pin a version:

```bash
INSTALL_DIR=~/bin VERSION=v0.1.1 \
  sh -c "$(curl -fsSL https://raw.githubusercontent.com/ragamo/lazyglab/master/scripts/install.sh)"
```

### Prebuilt binaries

Grab the archive for your platform from the [latest release](https://github.com/ragamo/lazyglab/releases/latest) (Linux and macOS on x86_64/arm64, Windows on x86_64), extract it, and move the `lazyglab` binary somewhere on your `PATH`:

```bash
tar -xzf lazyglab-<target>.tar.gz
mv lazyglab ~/.local/bin/
```

### From source

Requires Rust 1.85+ (edition 2024).

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
