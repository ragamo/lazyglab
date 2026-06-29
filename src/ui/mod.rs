pub mod splash;
pub mod auth_modal;
pub mod main_view;

use ratatui::prelude::*;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.screen {
        crate::app::AppScreen::Splash => splash::render(frame),
        crate::app::AppScreen::AuthModal => {
            splash::render(frame);
            auth_modal::render(frame, app);
        }
        crate::app::AppScreen::Main => main_view::render(frame, app),
    }
}
