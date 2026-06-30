use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;
use crate::theme;

const TABS: &[&str] = &["themes", "config"];

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

pub fn render(frame: &mut Frame, app: &mut App) {
    let t = app.theme;
    let screen = frame.area();
    let w = 70u16.min(screen.width);
    let h = 20u16.min(screen.height);
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
        Paragraph::new(Span::styled("settings", Style::default().fg(t.text).add_modifier(Modifier::BOLD))),
        title_area,
    );

    // Tab bar
    let tab_area = Rect { x: body.x, y: body.y + 1, width: body.width, height: 1 };
    let mut tab_spans: Vec<Span> = Vec::new();
    let mut tab_click_areas: Vec<Rect> = Vec::new();
    let mut x_offset = tab_area.x + 1; // leading space
    for (i, &tab) in TABS.iter().enumerate() {
        let is_active = i == app.settings_selected;
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        if i == 0 { tab_spans.push(Span::raw(" ")); }
        tab_click_areas.push(Rect { x: x_offset, y: tab_area.y, width: tab.len() as u16, height: 1 });
        tab_spans.push(Span::styled(tab.to_string(), style));
        tab_spans.push(Span::raw("   "));
        x_offset += tab.len() as u16 + 3;
    }
    app.settings_tab_areas = tab_click_areas;
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tab_area);

    // Divider
    let divider_area = Rect { x: body.x, y: body.y + 2, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(body.width as usize),
            Style::default().fg(t.border),
        )),
        divider_area,
    );

    // Content area
    let content_area = Rect {
        x: body.x,
        y: body.y + 3,
        width: body.width,
        height: body.height.saturating_sub(3),
    };

    match app.settings_selected {
        0 => render_theme_tab(frame, app, content_area),
        _ => {
            frame.render_widget(
                Paragraph::new(Span::styled("coming soon", Style::default().fg(t.text_dim))),
                Rect { x: content_area.x + 2, y: content_area.y + 1, width: content_area.width, height: 1 },
            );
        }
    }

    // Footer
    let footer_area = outer[1];
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓", Style::default().fg(t.accent)),
            Span::styled(" select  ", Style::default().fg(t.text_dim)),
            Span::styled("tab", Style::default().fg(t.accent)),
            Span::styled(" section", Style::default().fg(t.text_dim)),
        ])),
        footer_area,
    );
    let apply_footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↵ apply ", Style::default().fg(t.bg).bg(t.accent)),
        Span::raw(" "),
        Span::styled(" esc close ", Style::default().fg(t.text).bg(t.border)),
    ])).alignment(Alignment::Center);
    frame.render_widget(apply_footer, footer_area);
}

fn render_theme_tab(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let items: Vec<ListItem> = theme::ALL_THEMES
        .iter()
        .enumerate()
        .map(|(i, th)| {
            let is_selected = i == app.theme_selected;
            let is_confirmed = i == app.theme_confirmed;
            let style = if is_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if is_selected { " ▸ " } else { "   " };
            let suffix = if is_confirmed { " ✓" } else { "" };
            ListItem::new(format!("{}{}{}", prefix, th.name, suffix)).style(style)
        })
        .collect();

    let list_area = Rect { x: area.x, y: area.y, width: (area.width / 3).max(24), height: area.height };
    frame.render_widget(List::new(items), list_area);
}
