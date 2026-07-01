# lazyglab

A terminal UI (TUI) for GitLab — browse merge requests and pipeline statuses without leaving your terminal.

Built with Rust. Mouse-first interaction. Heavily inspired by [herdr](https://herdr.dev/).

<video src="https://github.com/user-attachments/assets/63cdd76b-889c-4319-a362-89765f053828" autoplay loop muted playsinline></video>

<div>
  <a href="caps/10_merge_request_pipeline_detail.png"><img src="caps/10_merge_request_pipeline_detail.png" width="24%" alt="Pipeline detail" /></a>
  <a href="caps/05_merge_requests_all.png"><img src="caps/05_merge_requests_all.png" width="24%" alt="All merge requests" /></a>
  <a href="caps/08_merge_request_changes.png"><img src="caps/08_merge_request_changes.png" width="24%" alt="Changes diff" /></a>
  <a href="caps/13_pipeline_stage_detail.png"><img src="caps/13_pipeline_stage_detail.png" width="24%" alt="Pipeline stage detail" /></a>
</div>

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

The token can also be provided via environment variable — `GITLAB_TOKEN` takes precedence over the config file:

```bash
GITLAB_TOKEN=glpat-xxx lazyglab
```

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

## Views

### Setup

<div>
  <a href="caps/00_no_project.png"><img src="caps/00_no_project.png" width="49%" alt="No project" /></a>
  <a href="caps/01_search_project.png"><img src="caps/01_search_project.png" width="49%" alt="Find project" /></a>
</div>

### Merge requests

<div align="center">
  <a href="caps/05_merge_requests_all.png"><img src="caps/05_merge_requests_all.png" width="80%" alt="All merge requests" /></a>
</div>

<div align="center">
  <a href="caps/02_merge_requests_open.png"><img src="caps/02_merge_requests_open.png" width="32%" alt="Open" /></a>
  <a href="caps/03_merge_requests_merged.png"><img src="caps/03_merge_requests_merged.png" width="32%" alt="Merged" /></a>
  <a href="caps/04_merge_requests_closed.png"><img src="caps/04_merge_requests_closed.png" width="32%" alt="Closed" /></a>
</div>

### Merge request detail

<table>
  <tr>
    <td><a href="caps/06_merge_request_overview.png"><img src="caps/06_merge_request_overview.png" alt="Overview" /></a></td>
    <td><a href="caps/07_merge_request_commits.png"><img src="caps/07_merge_request_commits.png" alt="Commits" /></a></td>
  </tr>
  <tr>
    <td><a href="caps/08_merge_request_changes.png"><img src="caps/08_merge_request_changes.png" alt="Changes" /></a></td>
    <td><a href="caps/10_merge_request_pipeline_detail.png"><img src="caps/10_merge_request_pipeline_detail.png" alt="Pipeline detail" /></a></td>
  </tr>
</table>

### Pipelines

<div>
  <a href="caps/11_pipelines.png"><img src="caps/11_pipelines.png" width="32%" alt="Pipelines" /></a>
  <a href="caps/12_pipeline_stages.png"><img src="caps/12_pipeline_stages.png" width="32%" alt="Pipeline stages" /></a>
  <a href="caps/13_pipeline_stage_detail.png"><img src="caps/13_pipeline_stage_detail.png" width="32%" alt="Stage detail" /></a>
</div>

### Themes

<table>
  <tr>
    <td><a href="caps/14_settings_theme_catpucci.png"><img src="caps/14_settings_theme_catpucci.png" alt="Catppuccin" /></a></td>
    <td><a href="caps/15_settings_theme_one_dark.png"><img src="caps/15_settings_theme_one_dark.png" alt="One Dark" /></a></td>
    <td><a href="caps/16_settings_theme_dracula.png"><img src="caps/16_settings_theme_dracula.png" alt="Dracula" /></a></td>
  </tr>
  <tr>
    <td><a href="caps/20_settings_theme_terminal.png"><img src="caps/20_settings_theme_terminal.png" alt="Terminal" /></a></td>
    <td><a href="caps/18_settings_theme_gruvbox.png"><img src="caps/18_settings_theme_gruvbox.png" alt="Gruvbox" /></a></td>
    <td><a href="caps/19_settings_theme_solarized.png"><img src="caps/19_settings_theme_solarized.png" alt="Solarized" /></a></td>
  </tr>
</table>

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
