use std::io;

use color_eyre::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::prelude::*;

use crate::app::App;
use crate::ui;

pub async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut event_stream = EventStream::new();

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
        }
    }

    Ok(())
}
