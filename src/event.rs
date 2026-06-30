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
    let mut tick_interval = time::interval(Duration::from_secs(app.refresh_interval_secs()));
    tick_interval.reset();
    let mut spinner_interval = time::interval(Duration::from_millis(100));
    spinner_interval.reset();

    loop {
        let app_ref = &mut *app;
        terminal.draw(|frame| ui::render(frame, app_ref))?;

        if app.should_quit {
            break;
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
            _ = tick_interval.tick() => {
                app.handle_message(AppMessage::Tick);
                tick_interval = time::interval(Duration::from_secs(app.refresh_interval_secs()));
                tick_interval.reset();
            }
            _ = spinner_interval.tick() => {
                app.handle_message(AppMessage::SpinnerTick);
            }
        }
    }

    Ok(())
}
