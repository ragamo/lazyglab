use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::App;

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

pub fn render(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 35, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Authentication ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(2), // Label
        Constraint::Length(3), // Input
        Constraint::Length(2), // Warning/Error
        Constraint::Min(0),   // Spacer
        Constraint::Length(1), // Footer
    ])
    .split(inner);

    let label = Paragraph::new("GitLab Personal Access Token:")
        .style(Style::default().fg(Color::White));
    frame.render_widget(label, chunks[0]);

    let masked: String = "●".repeat(app.token_input.len());
    let cursor = if app.is_validating { "" } else { "▌" };
    let input_text = format!("{}{}", masked, cursor);

    let input = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[1]);

    if let Some(ref warning) = app.token_source_warning {
        let warn = Paragraph::new(warning.as_str())
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: true });
        frame.render_widget(warn, chunks[2]);
    } else if let Some(ref error) = app.auth_error {
        let err = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        frame.render_widget(err, chunks[2]);
    }

    let footer_text = if app.is_validating {
        "Validating..."
    } else {
        "[Enter] Submit  [Esc] Quit"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}
