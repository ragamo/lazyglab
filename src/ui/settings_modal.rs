use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;
use crate::theme;

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
    let area = centered_rect(50, 55, frame.area());
    frame.render_widget(Clear, area);

    if app.theme_selector_open {
        render_theme_selector(frame, app, area);
    } else {
        render_settings_menu(frame, app, area);
    }
}

fn render_settings_menu(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let block = Block::default()
        .title(" Settings ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let settings_items = vec!["Theme"];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.settings_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if i == app.settings_selected { " ▸ " } else { "   " };
            let current = if *item == "Theme" {
                format!("  ({})", app.theme.name)
            } else {
                String::new()
            };
            ListItem::new(format!("{}{}{}", prefix, item, current)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓", Style::default().fg(t.accent)),
        Span::styled(" navigate ", Style::default().fg(t.text_dim)),
        Span::styled(" Enter", Style::default().fg(t.accent)),
        Span::styled(" select ", Style::default().fg(t.text_dim)),
        Span::styled(" Esc", Style::default().fg(t.accent)),
        Span::styled(" close", Style::default().fg(t.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[1]);
}

fn render_theme_selector(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let block = Block::default()
        .title(" Select Theme ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let content = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .split(chunks[0]);

    let items: Vec<ListItem> = theme::ALL_THEMES
        .iter()
        .enumerate()
        .map(|(i, theme)| {
            let style = if i == app.theme_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if i == app.theme_selected { " ▸ " } else { "   " };
            ListItem::new(format!("{}{}", prefix, theme.name)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, content[0]);

    let preview_theme = theme::ALL_THEMES[app.theme_selected];
    render_theme_preview(frame, preview_theme, content[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓", Style::default().fg(t.accent)),
        Span::styled(" navigate ", Style::default().fg(t.text_dim)),
        Span::styled(" Enter", Style::default().fg(t.accent)),
        Span::styled(" apply ", Style::default().fg(t.text_dim)),
        Span::styled(" Esc", Style::default().fg(t.accent)),
        Span::styled(" back", Style::default().fg(t.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[1]);
}

fn render_theme_preview(frame: &mut Frame, preview: &theme::Theme, area: Rect) {
    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(preview.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled("● Accent", Style::default().fg(preview.accent))),
        Line::from(Span::styled("  Text", Style::default().fg(preview.text))),
        Line::from(Span::styled("  Dimmed", Style::default().fg(preview.text_dim))),
        Line::from(Span::styled("✓ Success", Style::default().fg(preview.success))),
        Line::from(Span::styled("✗ Error", Style::default().fg(preview.error))),
        Line::from(Span::styled("● Warning", Style::default().fg(preview.warning))),
        Line::from(Span::styled("  Info", Style::default().fg(preview.info))),
        Line::from(Span::styled("★ Highlight", Style::default().fg(preview.highlight))),
    ];

    let preview_widget = Paragraph::new(lines);
    frame.render_widget(preview_widget, inner);
}
