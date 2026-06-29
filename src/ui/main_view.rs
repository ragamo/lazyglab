use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title(" lazyglab ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let user_info = if let Some(ref user) = app.current_user {
        format!("Authenticated as: {} (@{})", user.name, user.username)
    } else {
        "Loading...".to_string()
    };

    let mr_info = if app.merge_requests.is_empty() {
        "No merge requests loaded yet.".to_string()
    } else {
        format!("{} open merge requests", app.merge_requests.len())
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            &user_info,
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(mr_info),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}
