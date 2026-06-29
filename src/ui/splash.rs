use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::theme::Theme;

pub fn render(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();

    let block = Block::default()
        .title(" lazyglab ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "lazyglab",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("GitLab & GitHub TUI", Style::default().fg(theme.text))),
        Line::from(Span::styled("PRs · Pipelines · Reviews", Style::default().fg(theme.text))),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to quit",
            Style::default().fg(theme.text_dim),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}
