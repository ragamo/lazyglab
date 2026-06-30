# lazyglab — CLAUDE.md

## Project overview

Rust TUI application built with ratatui. Connects to GitLab (and eventually GitHub) to display merge requests and pipeline statuses in the terminal. Mouse-first interaction.

## Build & run

```bash
cargo build          # debug build
cargo build --release
cargo run            # requires a GitLab token (env var or config file)
```

No test suite yet. Type-check with `cargo check`.

## Architecture

### State machine (`src/app.rs`)

`App` is the single source of truth. Screens: `Splash → AuthModal → Main`. All UI state lives here including click areas (set during render, read during mouse events).

Async results flow through `tokio::sync::mpsc::UnboundedChannel` as `AppMessage` variants. Tasks are spawned with `tokio::spawn`; they send a message when done. The event loop in `src/event.rs` multiplexes terminal events and async messages via `tokio::select!`.

### Provider trait (`src/provider/mod.rs`)

Abstraction layer for hosting platforms. Methods: `validate_token`, `list_merge_requests`, `get_merge_request`, `get_pipeline_status`, `list_pipelines`, `list_user_projects`, `search_projects`. Only GitLab is implemented (`src/provider/gitlab.rs`). To add GitHub, implement the `Provider` trait.

### UI (`src/ui/`)

Immediate-mode rendering with ratatui. Each render pass writes click areas back into `App` (e.g. `app.tab_mr_area`, `app.find_link_area`). Mouse events read these areas.

- `main_view.rs` — main layout: header, tabs bar, content area, footer
- `auth_modal.rs` — masked token input
- `find_modal.rs` — project search with star-to-favorite
- `settings_modal.rs` — theme selector with live preview
- `splash.rs` — startup screen

### Themes (`src/theme.rs`)

`Theme` struct with semantic color fields (`accent`, `text`, `text_dim`, `border`, `success`, `error`, `warning`, `info`, `highlight`, `header_bg`, `bg`). 12 predefined palettes. Active theme stored in `app.theme: &'static Theme`. Changing `app.theme` immediately affects all renders — used for live preview in the settings modal.

### Config (`src/config/`)

TOML at `~/.config/lazyglab/config.toml` (Linux) or `~/Library/Application Support/lazyglab/config.toml` (macOS). Fields: `[auth].token`, `[gitlab].base_url`, `[gitlab].favorites` (list of `{id, name, path_with_namespace}`), `[ui].theme`.

### Token resolution order

1. `LAZYGLAB_TOKEN` env var
2. `GITLAB_TOKEN` env var
3. `[auth].token` in config file (shows yellow warning in auth modal)
4. User input via modal

## Key conventions

- Click areas are `Option<Rect>` fields on `App`, set during render, checked in `handle_mouse`
- All data loading is async via `tokio::spawn` + `AppMessage` channel
- Filtering MRs by state is done client-side after loading all with `state=all`
- Favorites are the only projects shown in the selector; Find modal is the discovery flow
- `load_merge_requests()` and `load_pipelines()` are called together on project change
- Auto-reload for pipelines: tick fires every `refresh_interval_secs` (default 30s), only when `autoreload_pipelines == true` and active tab is `Pipelines`
