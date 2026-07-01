use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let t = app.theme;
    let screen = frame.area();
    let w = 70u16.min(screen.width);
    let h = 12u16.min(screen.height);
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(w) / 2,
        y: screen.y + screen.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let outer = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1), // footer
    ]).split(inner);

    let body = outer[0];

    // Title
    let title_area = Rect { x: body.x + 1, y: body.y, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("login", Style::default().fg(t.text).add_modifier(Modifier::BOLD))),
        title_area,
    );

    // Token label
    let label_area = Rect { x: body.x + 2, y: body.y + 2, width: body.width.saturating_sub(4), height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "GitLab Personal Access Token",
            Style::default().fg(t.text),
        )),
        label_area,
    );

    // Token input
    let input_area = Rect { x: body.x + 2, y: body.y + 3, width: body.width.saturating_sub(4), height: 3 };
    let masked: String = "●".repeat(app.token_input.len());
    let cursor = if app.is_validating { "" } else { "▌" };
    let input = Paragraph::new(format!("{}{}", masked, cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.accent)),
        )
        .style(Style::default().fg(t.warning));
    frame.render_widget(input, input_area);

    // Warning / error
    let msg_area = Rect { x: body.x + 2, y: body.y + 6, width: body.width.saturating_sub(4), height: 1 };
    if let Some(ref warning) = app.token_source_warning {
        frame.render_widget(
            Paragraph::new(warning.as_str())
                .style(Style::default().fg(t.warning))
                .wrap(Wrap { trim: true }),
            msg_area,
        );
    } else if let Some(ref error) = app.auth_error {
        frame.render_widget(
            Paragraph::new(error.as_str())
                .style(Style::default().fg(t.error))
                .wrap(Wrap { trim: true }),
            msg_area,
        );
    }

    // Footer
    let footer_area = outer[1];
    if app.is_validating {
        frame.render_widget(
            Paragraph::new(Span::styled(" validating...", Style::default().fg(t.text_dim))),
            footer_area,
        );
    } else {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↵ submit ", Style::default().fg(t.bg).bg(t.accent)),
            Span::raw(" "),
            Span::styled(" esc close ", Style::default().fg(t.text).bg(t.border)),
        ])).alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}
