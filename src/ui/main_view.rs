use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table};

use crate::app::{App, MrFilter, Tab};
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
        Constraint::Percentage(60),
        Constraint::Min(20),
    ])
    .split(area);

    let project_path = app
        .projects
        .get(app.selected_project)
        .map(|p| p.path_with_namespace.as_str())
        .unwrap_or("No project");

    let project_name = shorten_path(project_path, 34);
    let selector_text = format!(" ⏷ {} ", project_name);
    let selector_width = 40u16.min(header_layout[0].width.saturating_sub(7));

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
        width: selector_width,
        height: header_layout[0].height,
    };
    frame.render_widget(selector, selector_area);
    app.project_selector_area = Some(selector_area);

    let find_link = Paragraph::new(Span::styled(
        " Find",
        Style::default().fg(t.accent),
    ));
    let find_area = Rect {
        x: selector_area.x + selector_area.width + 1,
        y: selector_area.y + 1,
        width: 5,
        height: 1,
    };
    frame.render_widget(find_link, find_area);
    app.find_link_area = Some(find_area);

    let right_area = header_layout[1];

    let right_text = if let Some(ref user) = app.current_user {
        vec![
            Span::styled("lazyglab", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(format!("@{}", user.username), Style::default().fg(t.success)),
            Span::styled("  logout", Style::default().fg(t.text_dim)),
            Span::raw(" "),
        ]
    } else {
        vec![
            Span::styled("lazyglab", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]
    };

    let right_widget = Paragraph::new(Line::from(right_text)).alignment(Alignment::Right);
    frame.render_widget(right_widget, right_area);

    // logout click area: last ~8 chars on the right
    if app.current_user.is_some() {
        let logout_width = 8u16;
        app.logout_link_area = Some(Rect {
            x: right_area.x + right_area.width.saturating_sub(logout_width + 1),
            y: right_area.y,
            width: logout_width,
            height: right_area.height,
        });
    } else {
        app.logout_link_area = None;
    }
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

    let mr_tab = Paragraph::new(Span::styled(" merge requests ", mr_style));
    let pipe_tab = Paragraph::new(Span::styled(" pipelines ", pipeline_style));
    let settings_link = Paragraph::new(Span::styled(
        "settings ",
        Style::default().fg(t.text_dim),
    ))
    .alignment(Alignment::Right);

    frame.render_widget(mr_tab, tabs_layout[0]);
    frame.render_widget(pipe_tab, tabs_layout[1]);
    frame.render_widget(settings_link, tabs_layout[2]);

    app.tab_mr_area = Some(tabs_layout[0]);
    app.tab_pipelines_area = Some(tabs_layout[1]);

    let settings_width = 9u16;
    app.settings_link_area = Some(Rect {
        x: tabs_layout[2].x + tabs_layout[2].width.saturating_sub(settings_width),
        y: tabs_layout[2].y,
        width: settings_width,
        height: tabs_layout[2].height,
    });
}

fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.active_tab {
        Tab::MergeRequests => render_merge_requests(frame, app, area),
        Tab::Pipelines => render_pipelines(frame, app, area),  // already &mut App
    }
}

fn render_merge_requests(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    render_mr_filters(frame, app, chunks[0]);

    let content_area = chunks[1];

    if app.mrs_loading {
        let loading = Paragraph::new("Loading merge requests...")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(loading, content_area);
        return;
    }

    let filtered: Vec<&_> = app
        .merge_requests
        .iter()
        .filter(|mr| app.mr_filter.matches(&mr.state))
        .collect();

    if filtered.is_empty() {
        let msg = if app.merge_requests.is_empty() {
            "No project selected or no merge requests"
        } else {
            "No merge requests match this filter"
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec!["IID", "Title", "Author", "Branch", "Updated"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = filtered
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

    frame.render_widget(table, content_area);
}

fn render_mr_filters(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let mut filter_areas = Vec::new();
    let mut x_offset = area.x;

    let spans: Vec<Span> = MrFilter::ALL_FILTERS
        .iter()
        .flat_map(|f| {
            let style = if *f == app.mr_filter {
                Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text_dim)
            };
            let label = format!(" {} ", f.label());
            let width = label.len() as u16;
            filter_areas.push(Rect {
                x: x_offset,
                y: area.y,
                width,
                height: 1,
            });
            x_offset += width + 1;
            vec![
                Span::styled(label, style),
                Span::raw(" "),
            ]
        })
        .collect();

    app.mr_filter_areas = filter_areas;

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

fn render_pipelines(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let top = chunks[0];

    let checkbox = if app.autoreload_pipelines { "☑" } else { "☐" };
    let checkbox_style = if app.autoreload_pipelines {
        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_dim)
    };

    let countdown = if app.autoreload_pipelines {
        if let Some(last) = app.last_pipeline_refresh {
            let elapsed = last.elapsed().as_secs();
            let remaining = app.refresh_interval_secs.saturating_sub(elapsed);
            format!(" {}s", remaining)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let hints = Line::from(vec![
        Span::styled(format!("{} autoreload", checkbox), checkbox_style),
        Span::styled(countdown, Style::default().fg(t.text_dim)),
    ]);
    frame.render_widget(Paragraph::new(hints), top);

    // checkbox click area: covers "☑ autoreload" (13 chars)
    app.autoreload_checkbox_area = Some(Rect { x: top.x, y: top.y, width: 14, height: 1 });

    let content_area = chunks[1];

    if app.pipelines_loading {
        let loading = Paragraph::new("Loading pipelines...")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(loading, content_area);
        return;
    }

    if app.pipelines.is_empty() {
        let empty = Paragraph::new("No project selected or no pipelines")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(empty, content_area);
        return;
    }

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

    frame.render_widget(table, content_area);
}

fn render_project_dropdown(frame: &mut Frame, app: &mut App, header_area: Rect) {
    let t = app.theme;

    let dropdown_x = header_area.x;
    let max_item_len = app
        .projects
        .iter()
        .map(|p| p.path_with_namespace.len() as u16 + 16) // prefix + star + hint
        .max()
        .unwrap_or(30);
    let dropdown_width = (max_item_len + 2).max(50).min(header_area.width);
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
            let is_selected = i == app.selected_project;
            let style = if is_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if is_selected { " ▸ ★ " } else { "   ★ " };
            let suffix = if is_selected { "  [s] unfav" } else { "" };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(t.warning)),
                Span::styled(p.path_with_namespace.clone(), style),
                Span::styled(suffix, Style::default().fg(t.text_dim)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.accent))
            .title(" Favorites ")
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

fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        return path.to_string();
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    format!("{}/.../{}", first, last)
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
        Span::styled(" f", Style::default().fg(t.accent)),
        Span::styled(" find ", Style::default().fg(t.text_dim)),
        Span::styled(" r", Style::default().fg(t.accent)),
        Span::styled(" refresh ", Style::default().fg(t.text_dim)),
        Span::styled(" ,", Style::default().fg(t.accent)),
        Span::styled(" settings", Style::default().fg(t.text_dim)),
    ];

    let footer = Paragraph::new(Line::from(hints));
    frame.render_widget(footer, area);
}
