// Provider trait, GitLab JSON models and some enum variants are intentional
// scaffolding (e.g. the future GitHub provider) that isn't wired up yet.
#![allow(dead_code)]

mod app;
mod auth;
mod config;
mod event;
mod provider;
mod table_nav;
mod theme;
mod ui;

use std::io;

use color_eyre::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;

use app::App;

const HELP: &str = "\
lazyglab — a TUI for GitLab and GitHub

USAGE:
    lazyglab [OPTIONS]

OPTIONS:
    -V, --version    Print version and exit
    -h, --help       Print this help and exit

Authenticate with a personal access token via the LAZYGLAB_TOKEN or
GITLAB_TOKEN environment variable, or the config file.";

#[tokio::main]
async fn main() -> Result<()> {
    // Handle CLI flags before taking over the terminal.
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-V" | "--version" => {
                println!("lazyglab {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(());
            }
            _ => {}
        }
    }

    color_eyre::install()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = event::run_event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}
