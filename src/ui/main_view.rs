use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table};

use crate::app::{App, Tab};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header with project selector
        Constraint::Length(1), // Tabs
        Constraint::Min(0),   // Content
        Constraint::Length(1), // Footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_tabs(frame, app, chunks[1]);
    render_content(frame, app, chunks[2]);
    render_footer(frame, app.theme, chunks[3]);

    if app.project_selector_open {
        render_project_dropdown(frame, app, chunks[0]);
    }
}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let bg_block = Block::default().style(Style::default().bg(t.header_bg));
    frame.render_widget(bg_block, area);

    let header_layout = Layout::horizontal([
        Constraint::Min(20),
        Constraint::Length(30),
    ])
    .split(area);

    let project_name = app
        .projects
        .get(app.selected_project)
        .map(|p| p.path_with_namespace.as_str())
        .unwrap_or("No project");

    let selector_text = format!(" ⏷ {} ", project_name);
    let selector = Paragraph::new(Span::styled(
        &selector_text,
        Style::default()
            .fg(t.text)
            .bg(t.header_bg)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.accent)),
    );

    let selector_area = Rect {
        x: header_layout[0].x,
        y: header_layout[0].y,
        width: header_layout[0].width.min(40),
        height: header_layout[0].height,
    };
    frame.render_widget(selector, selector_area);
    app.project_selector_area = Some(selector_area);

    let right_text = if let Some(ref user) = app.current_user {
        vec![
            Span::styled("lazyglab", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(format!("@{}", user.username), Style::default().fg(t.success)),
            Span::raw(" "),
        ]
    } else {
        vec![
            Span::styled("lazyglab", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]
    };

    let right_widget = Paragraph::new(Line::from(right_text)).alignment(Alignment::Right);
    frame.render_widget(right_widget, header_layout[1]);
}

fn render_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let bg_block = Block::default().style(Style::default().bg(t.header_bg));
    frame.render_widget(bg_block, area);

    let tabs_layout = Layout::horizontal([
        Constraint::Length(18),
        Constraint::Length(18),
        Constraint::Min(0),
    ])
    .split(area);

    let mr_style = if app.active_tab == Tab::MergeRequests {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_dim)
    };

    let pipeline_style = if app.active_tab == Tab::Pipelines {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_dim)
    };

    let mr_tab = Paragraph::new(Span::styled(" ● Merge Requests ", mr_style));
    let pipe_tab = Paragraph::new(Span::styled(" ● Pipelines ", pipeline_style));

    frame.render_widget(mr_tab, tabs_layout[0]);
    frame.render_widget(pipe_tab, tabs_layout[1]);

    app.tab_mr_area = Some(tabs_layout[0]);
    app.tab_pipelines_area = Some(tabs_layout[1]);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        Tab::MergeRequests => render_merge_requests(frame, app, area),
        Tab::Pipelines => render_pipelines(frame, app, area),
    }
}

fn render_merge_requests(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let header = Row::new(vec!["IID", "Title", "Author", "Branch", "Updated"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .merge_requests
        .iter()
        .map(|mr| {
            let status_color = match mr.state.as_str() {
                "opened" => t.success,
                "merged" => t.highlight,
                "closed" => t.error,
                _ => t.border,
            };

            Row::new(vec![
                Cell::from(format!("!{}", mr.iid)).style(Style::default().fg(status_color)),
                Cell::from(mr.title.clone()).style(Style::default().fg(t.text)),
                Cell::from(format!("@{}", mr.author.username)).style(Style::default().fg(t.warning)),
                Cell::from(mr.source_branch.clone()).style(Style::default().fg(t.info)),
                Cell::from(mr.updated_at[..10].to_string()).style(Style::default().fg(t.text_dim)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Min(30),
            Constraint::Length(15),
            Constraint::Length(22),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.border)),
    );

    frame.render_widget(table, area);
}

fn render_pipelines(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let header = Row::new(vec!["ID", "Status", "Branch", "URL"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .pipelines
        .iter()
        .map(|p| {
            let (status_icon, status_color) = match p.status.as_str() {
                "success" => ("✓", t.success),
                "failed" => ("✗", t.error),
                "running" => ("●", t.warning),
                "canceled" => ("○", t.text_dim),
                "pending" => ("◌", t.info),
                _ => ("?", t.border),
            };

            Row::new(vec![
                Cell::from(format!("#{}", p.id)).style(Style::default().fg(t.text_dim)),
                Cell::from(format!("{} {}", status_icon, p.status)).style(Style::default().fg(status_color)),
                Cell::from(p.r#ref.clone()).style(Style::default().fg(t.info)),
                Cell::from(p.web_url.clone()).style(Style::default().fg(t.text_dim)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(25),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.border)),
    );

    frame.render_widget(table, area);
}

fn render_project_dropdown(frame: &mut Frame, app: &mut App, header_area: Rect) {
    let t = app.theme;

    let dropdown_x = header_area.x;
    let dropdown_width = 40u16.min(header_area.width);
    let dropdown_height = (app.projects.len() as u16 + 2).min(10);

    let dropdown_area = Rect {
        x: dropdown_x,
        y: header_area.y + header_area.height,
        width: dropdown_width,
        height: dropdown_height,
    };

    frame.render_widget(Clear, dropdown_area);

    let items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == app.selected_project {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if i == app.selected_project { "▸ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, p.path_with_namespace)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.accent))
            .title(" Select Project ")
            .title_style(Style::default().fg(t.accent)),
    );

    frame.render_widget(list, dropdown_area);

    app.project_items_areas = (0..app.projects.len())
        .map(|i| Rect {
            x: dropdown_area.x + 1,
            y: dropdown_area.y + 1 + i as u16,
            width: dropdown_area.width.saturating_sub(2),
            height: 1,
        })
        .collect();
}

fn render_footer(frame: &mut Frame, t: &Theme, area: Rect) {
    let hints = vec![
        Span::styled(" q", Style::default().fg(t.accent)),
        Span::styled(" quit ", Style::default().fg(t.text_dim)),
        Span::styled(" Tab", Style::default().fg(t.accent)),
        Span::styled(" switch tab ", Style::default().fg(t.text_dim)),
        Span::styled(" p", Style::default().fg(t.accent)),
        Span::styled(" project ", Style::default().fg(t.text_dim)),
        Span::styled(" 1/2", Style::default().fg(t.accent)),
        Span::styled(" tabs ", Style::default().fg(t.text_dim)),
        Span::styled(" ,", Style::default().fg(t.accent)),
        Span::styled(" settings", Style::default().fg(t.text_dim)),
    ];

    let footer = Paragraph::new(Line::from(hints));
    frame.render_widget(footer, area);
}
