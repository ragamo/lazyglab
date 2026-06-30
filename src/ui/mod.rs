pub mod splash;
pub mod auth_modal;
pub mod main_view;
pub mod settings_modal;
pub mod find_modal;
pub mod click_regions;

use ratatui::prelude::*;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.click_regions.clear();
    match app.screen {
        crate::app::AppScreen::Splash => splash::render(frame, app.theme),
        crate::app::AppScreen::AuthModal => {
            splash::render(frame, app.theme);
            auth_modal::render(frame, app);
        }
        crate::app::AppScreen::Main => {
            main_view::render(frame, app);
            if app.find_modal_open {
                find_modal::render(frame, app);
            } else if app.settings_open {
                settings_modal::render(frame, app);
            }
        }
    }
}
