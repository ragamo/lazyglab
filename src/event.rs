use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::prelude::*;
use tokio::time;

use crate::app::{App, AppMessage};
use crate::ui;

pub async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut event_stream = EventStream::new();
    // UI tick: every second for the countdown display
    let mut ui_tick = time::interval(Duration::from_secs(1));
    // Reload tick: fires at the configured interval
    let mut reload_tick = time::interval(Duration::from_secs(app.refresh_interval_secs));
    ui_tick.tick().await;
    reload_tick.tick().await;

    loop {
        let app_ref = &mut *app;
        terminal.draw(|frame| ui::render(frame, app_ref))?;

        if app.should_quit {
            break;
        }

        // Rebuild reload interval if changed
        if reload_tick.period() != Duration::from_secs(app.refresh_interval_secs) {
            reload_tick = time::interval(Duration::from_secs(app.refresh_interval_secs));
            reload_tick.tick().await;
        }

        tokio::select! {
            maybe_event = event_stream.next() => {
                if let Some(Ok(event)) = maybe_event {
                    app.handle_event(event);
                }
            }
            Some(msg) = app.message_rx.recv() => {
                app.handle_message(msg);
            }
            _ = ui_tick.tick() => {
                // Redraws happen at top of loop; this just wakes us up every second
                // so the countdown display stays current. No action needed.
            }
            _ = reload_tick.tick() => {
                app.handle_message(AppMessage::Tick);
            }
        }
    }

    Ok(())
}
