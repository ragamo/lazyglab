use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};

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
    app.click_regions.header.project_selector = Some(selector_area);

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
    app.click_regions.header.find_link = Some(find_area);

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
        app.click_regions.header.logout_link = Some(Rect {
            x: right_area.x + right_area.width.saturating_sub(logout_width + 1),
            y: right_area.y,
            width: logout_width,
            height: right_area.height,
        });
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

    app.click_regions.header.tab_mr = Some(tabs_layout[0]);
    app.click_regions.header.tab_pipelines = Some(tabs_layout[1]);

    let settings_width = 9u16;
    app.click_regions.header.settings_link = Some(Rect {
        x: tabs_layout[2].x + tabs_layout[2].width.saturating_sub(settings_width),
        y: tabs_layout[2].y,
        width: settings_width,
        height: tabs_layout[2].height,
    });
}

fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.active_tab {
        Tab::MergeRequests => {
            if app.mr_detail_open {
                // Init height on first open
                if app.mr_detail_height == 0 {
                    app.mr_detail_height = (area.height * 70 / 100).max(14);
                }
                let detail_height = app.mr_detail_height.min(area.height.saturating_sub(6));
                let table_height = area.height.saturating_sub(detail_height);
                let splits = Layout::vertical([
                    Constraint::Length(table_height),
                    Constraint::Length(detail_height),
                ])
                .split(area);
                render_merge_requests(frame, app, splits[0]);
                render_mr_detail(frame, app, splits[1]);

                // Extend resize area to also cover the bottom border of the table
                let table_bottom_row = splits[0].y + splits[0].height.saturating_sub(1);
                app.click_regions.mr_detail.resize = Some(Rect {
                    x: splits[1].x,
                    y: table_bottom_row,
                    width: splits[1].width,
                    height: 2, // bottom border of table + top border of detail
                });
            } else {
                app.mr_detail_height = 0;
                render_merge_requests(frame, app, area);
            }
        }
        Tab::Pipelines => {
            if app.pipeline_detail_open {
                if app.pipeline_detail_height == 0 {
                    app.pipeline_detail_height = (area.height * 70 / 100).max(14);
                }
                let detail_height = app.pipeline_detail_height.min(area.height.saturating_sub(6));
                let table_height = area.height.saturating_sub(detail_height);
                let splits = Layout::vertical([
                    Constraint::Length(table_height),
                    Constraint::Length(detail_height),
                ])
                .split(area);
                render_pipelines(frame, app, splits[0]);
                render_pipeline_detail(frame, app, splits[1]);
            } else {
                app.pipeline_detail_height = 0;
                render_pipelines(frame, app, area);
            }
        }
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

    // Clamp offset to valid range
    let visible_rows = content_area.height.saturating_sub(4) as usize; // border + header + margin + border
    app.mr_nav.visible_rows = visible_rows;
    let max_offset = filtered.len().saturating_sub(visible_rows);
    if app.mr_nav.offset > max_offset {
        app.mr_nav.offset = max_offset;
    }

    let visible_filtered: Vec<&_> = filtered
        .iter()
        .skip(app.mr_nav.offset)
        .take(visible_rows)
        .copied()
        .collect();

    let header = Row::new(vec!["IID", "Title", "Author", "Branch", "Updated"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = visible_filtered
        .iter()
        .enumerate()
        .map(|(i, mr)| {
            let actual_index = app.mr_nav.offset + i;
            let is_selected = app.mr_nav.selected == Some(actual_index);

            let status_color = match mr.state.as_str() {
                "opened" => t.success,
                "merged" => t.highlight,
                "closed" => t.error,
                _ => t.border,
            };

            let row = Row::new(vec![
                Cell::from(format!("!{}", mr.iid)).style(Style::default().fg(status_color)),
                Cell::from(mr.title.clone()).style(Style::default().fg(t.text)),
                Cell::from(format!("@{}", mr.author.username)).style(Style::default().fg(t.warning)),
                Cell::from(mr.source_branch.clone()).style(Style::default().fg(t.info)),
                Cell::from(mr.updated_at[..10].to_string()).style(Style::default().fg(t.text_dim)),
            ]);

            if is_selected {
                row.style(Style::default().bg(t.border))
            } else {
                row
            }
        })
        .collect();

    let scroll_indicator = if filtered.len() > visible_rows {
        format!(" {}-{}/{} ", app.mr_nav.offset + 1, (app.mr_nav.offset + visible_filtered.len()).min(filtered.len()), filtered.len())
    } else {
        String::new()
    };

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
            .border_style(Style::default().fg(t.border))
            .title_bottom(Line::from(scroll_indicator).right_aligned())
            .title_style(Style::default().fg(t.text_dim)),
    );

    frame.render_widget(table, content_area);

    // Scrollbar (only when content overflows)
    if filtered.len() > visible_rows {
        let mut scrollbar_state = ScrollbarState::new(filtered.len().saturating_sub(visible_rows))
            .position(app.mr_nav.offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let scrollbar_area = Rect {
            x: content_area.x + content_area.width.saturating_sub(1),
            y: content_area.y + 1,
            width: 1,
            height: content_area.height.saturating_sub(2),
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Register click areas for visible rows (indices map to filtered[offset + i])
    let row_start_y = content_area.y + 3; // border + header + margin
    app.click_regions.main.mr_row_areas = visible_filtered
        .iter()
        .enumerate()
        .map(|(i, _)| Rect {
            x: content_area.x + 1,
            y: row_start_y + i as u16,
            width: content_area.width.saturating_sub(2),
            height: 1,
        })
        .collect();
}

fn render_mr_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let mr = {
        let filtered: Vec<&_> = app
            .merge_requests
            .iter()
            .filter(|mr| app.mr_filter.matches(&mr.state))
            .collect();
        match app.mr_nav.selected.and_then(|i| filtered.get(i)) {
            Some(mr) => (*mr).clone(),
            None => return,
        }
    };

    // Drag resize area = top border row
    app.click_regions.mr_detail.resize = Some(Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    });
    app.click_regions.mr_detail.bounds = Some(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // [X] close — top-right corner
    let close_area = Rect {
        x: area.x + area.width.saturating_sub(4),
        y: area.y,
        width: 3,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled("[X]", Style::default().fg(t.text_dim))),
        close_area,
    );
    app.click_regions.mr_detail.close = Some(close_area);

    // Split inner: tabs row + blank line + content
    let inner_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    // Render tabs
    let tab_row = inner_chunks[0];
    let mut x_offset = tab_row.x;
    let mut tab_areas = Vec::new();

    let tab_spans: Vec<Span> = crate::app::MrDetailTab::ALL
        .iter()
        .flat_map(|tab| {
            let is_active = *tab == app.mr_detail_tab;
            let style = if is_active {
                Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text_dim)
            };
            let label = tab.label();
            let width = label.len() as u16;
            tab_areas.push(Rect {
                x: x_offset,
                y: tab_row.y,
                width,
                height: 1,
            });
            x_offset += width + 1;
            vec![Span::styled(label, style), Span::raw(" ")]
        })
        .collect();

    app.click_regions.mr_detail.tab_areas = tab_areas;
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tab_row);

    // Render content
    let content_area = inner_chunks[2]; // [1] is blank line
    match app.mr_detail_tab {
        crate::app::MrDetailTab::Overview => render_mr_overview(frame, app, &mr, content_area),
        crate::app::MrDetailTab::Commits => render_mr_commits(frame, app, content_area),
        _ => {
            let placeholder = Paragraph::new(Span::styled(
                "coming soon",
                Style::default().fg(t.text_dim),
            ));
            frame.render_widget(placeholder, content_area);
        }
    }
}

fn render_mr_overview(
    frame: &mut Frame,
    app: &mut App,
    mr: &crate::provider::types::MergeRequest,
    area: Rect,
) {
    use ratatui::widgets::Wrap;
    let t = app.theme;

    let status_color = match mr.state.as_str() {
        "opened" => t.success,
        "merged" => t.highlight,
        "closed" => t.error,
        _ => t.border,
    };

    let time_ago = format_time_ago(&mr.updated_at);

    let lines = vec![
        // Title
        Line::from(vec![
            Span::styled(format!("!{}", mr.iid), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(mr.title.clone(), Style::default().fg(t.text).add_modifier(Modifier::BOLD)),
        ]),
        // Metadata line
        Line::from(vec![
            Span::styled(format!("@{}", mr.author.username), Style::default().fg(t.warning)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(mr.state.clone(), Style::default().fg(status_color)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(mr.source_branch.clone(), Style::default().fg(t.info)),
            Span::styled(" → ", Style::default().fg(t.text_dim)),
            Span::styled(mr.target_branch.clone(), Style::default().fg(t.info)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(time_ago, Style::default().fg(t.text_dim)),
        ]),
        // URL
        Line::from(Span::styled(mr.web_url.clone(), Style::default().fg(t.text_dim))),
        // Blank line
        Line::from(""),
    ];

    let header_height = lines.len() as u16;
    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(Paragraph::new(lines), chunks[0]);

    // Description
    if app.mr_detail_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            chunks[1],
        );
    } else {
        let desc_text = app
            .mr_detail_full
            .as_ref()
            .and_then(|m| m.description.as_deref())
            .filter(|d| !d.trim().is_empty())
            .unwrap_or("No description");

        let md = tui_markdown::from_str(desc_text);
        let desc_area = chunks[1];
        let visible_height = desc_area.height as u16;
        let content_height = md.lines.len() as u16;

        // Clamp scroll so content doesn't scroll past the end
        let max_scroll = content_height.saturating_sub(visible_height);
        if app.mr_desc_scroll > max_scroll {
            app.mr_desc_scroll = max_scroll;
        }

        let desc = Paragraph::new(md)
            .style(Style::default().fg(t.text))
            .wrap(Wrap { trim: false })
            .scroll((app.mr_desc_scroll, 0));
        frame.render_widget(desc, desc_area);

        // Scrollbar
        if content_height > visible_height {
            let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                .position(app.mr_desc_scroll as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_style(Style::default().fg(t.border))
                .thumb_style(Style::default().fg(t.accent));
            let scrollbar_area = Rect {
                x: desc_area.x + desc_area.width.saturating_sub(1),
                y: desc_area.y,
                width: 1,
                height: desc_area.height,
            };
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }
}

fn render_mr_commits(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    if app.mr_commits_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            area,
        );
        return;
    }

    if app.mr_commits.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("No commits", Style::default().fg(t.text_dim))),
            area,
        );
        return;
    }

    let card_height: u16 = 3; // title + author line + separator
    let visible_height = area.height;
    let content_height = app.mr_commits.len() as u16 * card_height;
    let max_scroll = content_height.saturating_sub(visible_height);
    if app.mr_commits_scroll > max_scroll {
        app.mr_commits_scroll = max_scroll;
    }

    let mut lines: Vec<Line> = Vec::new();
    for commit in &app.mr_commits {
        lines.push(Line::from(vec![
            Span::styled(&commit.short_id, Style::default().fg(t.info)),
            Span::styled("  ", Style::default()),
            Span::styled(&commit.title, Style::default().fg(t.text)),
        ]));
        let time_ago = format_time_ago(&commit.created_at);
        lines.push(Line::from(vec![
            Span::raw("          "),
            Span::styled(
                format!("{} authored {}", commit.author_name, time_ago),
                Style::default().fg(t.text_dim),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            "─".repeat(area.width.saturating_sub(1) as usize),
            Style::default().fg(t.border),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .scroll((app.mr_commits_scroll, 0));
    frame.render_widget(paragraph, area);

    if content_height > visible_height {
        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
            .position(app.mr_commits_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y,
            width: 1,
            height: area.height,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn format_time_ago(date_str: &str) -> String {
    let date_part = &date_str[..10]; // "YYYY-MM-DD"
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 { return date_part.to_string(); }
    let (y, m, d) = match (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
        (Ok(y), Ok(m), Ok(d)) => (y, m, d),
        _ => return date_part.to_string(),
    };
    // Approximate days since epoch for comparison
    let date_days = y * 365 + m * 30 + d;
    let now_days = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (now / 86400) as i64
    };
    // Convert epoch days from 1970 base to same scale
    let epoch_offset = 1970 * 365 + 1 * 30 + 1;
    let diff = now_days - (date_days - epoch_offset);
    if diff < 0 {
        date_part.to_string()
    } else if diff == 0 {
        "today".to_string()
    } else if diff == 1 {
        "yesterday".to_string()
    } else if diff < 30 {
        format!("{} days ago", diff)
    } else if diff < 365 {
        let months = diff / 30;
        if months == 1 { "1 month ago".to_string() } else { format!("{} months ago", months) }
    } else {
        let years = diff / 365;
        if years == 1 { "1 year ago".to_string() } else { format!("{} years ago", years) }
    }
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

    app.click_regions.main.mr_filter_areas = filter_areas;

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
    app.click_regions.main.autoreload_checkbox = Some(Rect { x: top.x, y: top.y, width: 14, height: 1 });

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

    let total = app.pipelines.len();
    let visible_rows = content_area.height.saturating_sub(4) as usize;
    app.pipeline_nav.visible_rows = visible_rows;
    app.pipeline_nav.clamp(total);

    let visible: Vec<&_> = app.pipelines
        .iter()
        .skip(app.pipeline_nav.offset)
        .take(visible_rows)
        .collect();

    let header = Row::new(vec!["ID", "Status", "Branch", "URL"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let actual_index = app.pipeline_nav.offset + i;
            let is_selected = app.pipeline_nav.selected == Some(actual_index);

            let (status_icon, status_color) = match p.status.as_str() {
                "success" => ("✓", t.success),
                "failed" => ("✗", t.error),
                "running" => ("●", t.warning),
                "canceled" => ("○", t.text_dim),
                "pending" => ("◌", t.info),
                _ => ("?", t.border),
            };

            let row = Row::new(vec![
                Cell::from(format!("#{}", p.id)).style(Style::default().fg(t.text_dim)),
                Cell::from(format!("{} {}", status_icon, p.status)).style(Style::default().fg(status_color)),
                Cell::from(p.r#ref.clone()).style(Style::default().fg(t.info)),
                Cell::from(p.web_url.clone()).style(Style::default().fg(t.text_dim)),
            ]);

            if is_selected {
                row.style(Style::default().bg(t.border))
            } else {
                row
            }
        })
        .collect();

    let scroll_indicator = if total > visible_rows {
        format!(" {}-{}/{} ", app.pipeline_nav.offset + 1, (app.pipeline_nav.offset + visible.len()).min(total), total)
    } else {
        String::new()
    };

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
            .border_style(Style::default().fg(t.border))
            .title_bottom(Line::from(scroll_indicator).right_aligned())
            .title_style(Style::default().fg(t.text_dim)),
    );

    frame.render_widget(table, content_area);

    // Register click areas for visible pipeline rows
    let row_start_y = content_area.y + 3; // border + header + margin
    app.click_regions.main.pipeline_row_areas = visible
        .iter()
        .enumerate()
        .map(|(i, _)| Rect {
            x: content_area.x + 1,
            y: row_start_y + i as u16,
            width: content_area.width.saturating_sub(2),
            height: 1,
        })
        .collect();

    if total > visible_rows {
        let mut scrollbar_state = ScrollbarState::new(total.saturating_sub(visible_rows))
            .position(app.pipeline_nav.offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let scrollbar_area = Rect {
            x: content_area.x + content_area.width.saturating_sub(1),
            y: content_area.y + 1,
            width: 1,
            height: content_area.height.saturating_sub(2),
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn render_pipeline_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let pipeline = match app.pipeline_nav.selected.and_then(|i| app.pipelines.get(i)) {
        Some(p) => p,
        None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let title = format!(
        "Pipeline #{} — {} ({})",
        pipeline.id, pipeline.status, pipeline.r#ref
    );
    frame.render_widget(
        Paragraph::new(Span::styled(title, Style::default().fg(t.text).add_modifier(Modifier::BOLD))),
        inner,
    );

    // Close button [X]
    let close_area = Rect {
        x: area.x + area.width.saturating_sub(4),
        y: area.y,
        width: 3,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled("[X]", Style::default().fg(t.text_dim))),
        close_area,
    );
    app.click_regions.pipeline_detail.bounds = Some(area);
    app.click_regions.pipeline_detail.close = Some(close_area);
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

    app.click_regions.project_dropdown.bounds = Some(dropdown_area);
    app.click_regions.project_dropdown.items = (0..app.projects.len())
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
