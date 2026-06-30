use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};

use crate::app::{App, MrFilter, Tab};
use crate::theme::Theme;
use crate::ui::pipeline_view::{pipeline_card_lines, PipelineView};

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

    let status_color = match mr.state.as_str() {
        "opened" => t.success,
        "merged" => t.highlight,
        "closed" => t.error,
        _ => t.border,
    };
    let time_ago = format_time_ago(&mr.updated_at);
    let summary_lines = vec![
        Line::from(vec![
            Span::styled(format!("!{}", mr.iid), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("  ", Style::default()),
            Span::styled(mr.title.clone(), Style::default().fg(t.text).add_modifier(Modifier::BOLD)),
        ]),
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
        Line::from(Span::styled(mr.web_url.clone(), Style::default().fg(t.text_dim))),
    ];
    let summary_height = summary_lines.len() as u16;

    // Split inner: summary + blank + tabs + separator + content
    let inner_chunks = Layout::vertical([
        Constraint::Length(summary_height),     // summary
        Constraint::Length(1),                  // blank
        Constraint::Length(1),                  // tabs row
        Constraint::Length(1),                  // separator
        Constraint::Min(0),                     // content
    ])
    .split(inner);

    // Render summary (always visible)
    frame.render_widget(Paragraph::new(summary_lines), inner_chunks[0]);

    // Render tabs
    let tab_row = inner_chunks[2];
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

    // Separator
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(t.border),
        )),
        inner_chunks[3],
    );

    // Render content
    let content_area = inner_chunks[4];
    match app.mr_detail_tab {
        crate::app::MrDetailTab::Overview => render_mr_overview(frame, app, &mr, content_area),
        crate::app::MrDetailTab::Commits => render_mr_commits(frame, app, content_area),
        crate::app::MrDetailTab::Pipelines => render_mr_pipelines(frame, app, content_area),
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
    _mr: &crate::provider::types::MergeRequest,
    area: Rect,
) {
    use ratatui::widgets::Wrap;
    let t = app.theme;

    // Description
    if app.mr_detail_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            area,
        );
    } else {
        let desc_text = app
            .mr_detail_full
            .as_ref()
            .and_then(|m| m.description.as_deref())
            .filter(|d| !d.trim().is_empty())
            .unwrap_or("No description");

        let md = tui_markdown::from_str(desc_text);
        let desc_area = area;
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

fn render_mr_pipelines(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    if app.mr_pipelines_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            area,
        );
        return;
    }

    if app.mr_pipelines.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("No pipelines", Style::default().fg(t.text_dim))),
            area,
        );
        return;
    }

    // Split horizontally if job log is open
    let (list_area, log_area) = if app.job_log_open {
        let splits = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]).split(area);
        (splits[0], Some(splits[1]))
    } else {
        (area, None)
    };

    app.click_regions.mr_detail.job_areas.clear();
    let empty_stages = vec![];
    let mut lines: Vec<Line> = Vec::new();
    let mut line_idx: u16 = 0;
    let scroll = app.mr_pipelines_scroll;

    for pipeline in &app.mr_pipelines {
        let enriched = app.mr_pipeline_enriched.get(&pipeline.id);

        // Header lines (no jobs)
        let view = PipelineView {
            id: pipeline.id,
            status: &pipeline.status,
            r#ref: &pipeline.r#ref,
            created_at: &pipeline.created_at,
            duration: enriched.and_then(|e| e.duration),
            user: enriched.and_then(|e| e.user.as_ref()).map(|u| u.username.as_str()),
            mr_iid: enriched.and_then(|e| e.mr_ref.as_ref()).map(|m| m.iid),
            mr_title: enriched.and_then(|e| e.mr_ref.as_ref()).map(|m| m.title.as_str()),
            stages: &[],
            tick_frame: app.tick_frame,
        };
        let header = pipeline_card_lines(&view, t, list_area.width);
        let header_len = header.len() as u16;
        lines.extend(header);
        line_idx += header_len;

        // Stage/job lines with click areas
        use crate::ui::pipeline_view::spinner_char;
        let spinner = spinner_char(app.tick_frame);
        let stages = enriched.map(|e| e.stages.as_slice()).unwrap_or(&empty_stages);
        for stage in stages {
            let (s_icon, s_color) = match stage.status.as_str() {
                "success" | "passed" => ("✓", t.success),
                "failed" => ("✗", t.error),
                "running" => (spinner, t.warning),
                "pending" => ("◌", t.info),
                "canceled" | "skipped" => ("○", t.text_dim),
                _ => ("?", t.border),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", s_icon), Style::default().fg(s_color)),
                Span::styled(stage.name.clone(), Style::default().fg(t.text)),
            ]));
            line_idx += 1;

            for job in &stage.jobs {
                let (j_icon, j_color) = match job.status.as_str() {
                    "success" | "passed" => ("✓", t.success),
                    "failed" => ("✗", t.error),
                    "running" => (spinner, t.warning),
                    "pending" => ("◌", t.info),
                    "canceled" | "skipped" => ("○", t.text_dim),
                    _ => ("○", t.border),
                };
                let is_selected = app.selected_job_id == Some(job.id);
                let is_bridge = !job.sub_jobs.is_empty();
                let style = if is_selected {
                    Style::default().fg(t.bg).bg(t.accent)
                } else if is_bridge {
                    Style::default().fg(t.text)
                } else {
                    Style::default().fg(t.text_dim)
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("      {} ", j_icon), Style::default().fg(j_color)),
                    Span::styled(job.name.clone(), style),
                ]));
                let visible_row = (line_idx as i32) - (scroll as i32);
                if visible_row >= 0 && (list_area.y + visible_row as u16) < list_area.y + list_area.height {
                    app.click_regions.mr_detail.job_areas.push((Rect {
                        x: list_area.x,
                        y: list_area.y + visible_row as u16,
                        width: list_area.width,
                        height: 1,
                    }, job.id, job.name.clone()));
                }
                line_idx += 1;

                for sub in &job.sub_jobs {
                    let (sj_icon, sj_color) = match sub.status.as_str() {
                        "success" | "passed" => ("✓", t.success),
                        "failed" => ("✗", t.error),
                        "running" => (spinner, t.warning),
                        "pending" => ("◌", t.info),
                        "canceled" | "skipped" => ("○", t.text_dim),
                        _ => ("○", t.border),
                    };
                    let is_sel = app.selected_job_id == Some(sub.id);
                    let sub_style = if is_sel { Style::default().fg(t.bg).bg(t.accent) } else { Style::default().fg(t.text_dim) };
                    lines.push(Line::from(vec![
                        Span::styled(format!("          {} ", sj_icon), Style::default().fg(sj_color)),
                        Span::styled(sub.name.clone(), sub_style),
                    ]));
                    let visible_row = (line_idx as i32) - (scroll as i32);
                    if visible_row >= 0 && (list_area.y + visible_row as u16) < list_area.y + list_area.height {
                        app.click_regions.mr_detail.job_areas.push((Rect {
                            x: list_area.x,
                            y: list_area.y + visible_row as u16,
                            width: list_area.width,
                            height: 1,
                        }, sub.id, sub.name.clone()));
                    }
                    line_idx += 1;
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "─".repeat(list_area.width.saturating_sub(1) as usize),
            Style::default().fg(t.border),
        )));
        line_idx += 2;
    }

    render_scrollable_lines(frame, &mut app.mr_pipelines_scroll, lines, list_area, t);

    if let Some(log_area) = log_area {
        render_job_log(frame, app, log_area);
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
    let content_area = area;

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

    let header = Row::new(vec!["ID", "Status", "Branch", "Source", "Updated"])
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

            let source = p.source.as_deref().unwrap_or("");
            let updated = p.updated_at.as_deref()
                .or(Some(p.created_at.as_str()))
                .and_then(|s| s.get(..10))
                .unwrap_or("");
            let row = Row::new(vec![
                Cell::from(format!("#{}", p.id)).style(Style::default().fg(t.accent)),
                Cell::from(format!("{} {}", status_icon, p.status)).style(Style::default().fg(status_color)),
                Cell::from(p.r#ref.clone()).style(Style::default().fg(t.info)),
                Cell::from(source.to_string()).style(Style::default().fg(t.text_dim)),
                Cell::from(updated.to_string()).style(Style::default().fg(t.text_dim)),
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
            Constraint::Length(10),  // #ID
            Constraint::Length(12),  // Status
            Constraint::Min(20),     // Branch
            Constraint::Length(25),  // Source
            Constraint::Length(12),  // Updated
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

fn render_scrollable_lines(frame: &mut Frame, scroll: &mut u16, lines: Vec<Line>, area: Rect, t: &Theme) {
    let visible_height = area.height;
    let content_height = lines.len() as u16;
    let max_scroll = content_height.saturating_sub(visible_height);
    if *scroll > max_scroll { *scroll = max_scroll; }

    frame.render_widget(Paragraph::new(lines).scroll((*scroll, 0)), area);

    if content_height > visible_height {
        let mut state = ScrollbarState::new(max_scroll as usize).position(*scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None).end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y, width: 1, height: area.height,
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
    }
}

fn render_pipeline_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let pipeline = match app.pipeline_nav.selected.and_then(|i| app.pipelines.get(i)).cloned() {
        Some(p) => p,
        None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

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
    app.click_regions.pipeline_detail.job_areas.clear();

    if app.pipeline_detail_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            inner,
        );
        return;
    }

    // Split inner horizontally if job log is open
    let (detail_area, log_area) = if app.job_log_open {
        let splits = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]).split(inner);
        (splits[0], Some(splits[1]))
    } else {
        (inner, None)
    };

    let empty_stages = vec![];
    let enriched = app.pipeline_detail_enriched.as_ref();

    // Header lines (status, id, ref, user, mr)
    let view = PipelineView {
        id: pipeline.id,
        status: &pipeline.status,
        r#ref: &pipeline.r#ref,
        created_at: &pipeline.created_at,
        duration: enriched.and_then(|e| e.duration),
        user: enriched.and_then(|e| e.user.as_ref()).map(|u| u.username.as_str()),
        mr_iid: enriched.and_then(|e| e.mr_ref.as_ref()).map(|m| m.iid),
        mr_title: enriched.and_then(|e| e.mr_ref.as_ref()).map(|m| m.title.as_str()),
        stages: &[],
        tick_frame: app.tick_frame,
    };
    let header_lines = pipeline_card_lines(&view, t, detail_area.width);
    let header_height = header_lines.len() as u16;

    // Build stage/job lines with click area tracking
    let stages = enriched.map(|e| e.stages.as_slice()).unwrap_or(&empty_stages);
    let mut job_lines: Vec<Line> = Vec::new();
    let mut job_areas: Vec<(Rect, u64, String)> = Vec::new();
    let row_start = detail_area.y + header_height;
    let scroll = app.pipeline_detail_scroll;

    use crate::ui::pipeline_view::spinner_char;
    let spinner = spinner_char(app.tick_frame);

    let mut line_idx: u16 = 0;
    for stage in stages {
        let (s_icon, s_color) = {
            let (i, c) = match stage.status.as_str() {
                "success" | "passed" => ("✓", t.success),
                "failed" => ("✗", t.error),
                "running" => (spinner, t.warning),
                "pending" => ("◌", t.info),
                "canceled" | "skipped" => ("○", t.text_dim),
                _ => ("?", t.border),
            };
            (i, c)
        };
        job_lines.push(Line::from(vec![
            Span::styled(format!("  {} ", s_icon), Style::default().fg(s_color)),
            Span::styled(stage.name.clone(), Style::default().fg(t.text)),
        ]));
        line_idx += 1;

        for job in &stage.jobs {
            let (j_icon, j_color) = match job.status.as_str() {
                "success" | "passed" => ("✓", t.success),
                "failed" => ("✗", t.error),
                "running" => (spinner, t.warning),
                "pending" => ("◌", t.info),
                "canceled" | "skipped" => ("○", t.text_dim),
                _ => ("○", t.border),
            };
            let is_selected = app.selected_job_id == Some(job.id);
            let is_bridge = !job.sub_jobs.is_empty();
            let job_style = if is_selected {
                Style::default().fg(t.bg).bg(t.accent)
            } else if is_bridge {
                Style::default().fg(t.text)
            } else {
                Style::default().fg(t.text_dim)
            };
            job_lines.push(Line::from(vec![
                Span::styled(format!("      {} ", j_icon), Style::default().fg(j_color)),
                Span::styled(job.name.clone(), job_style),
            ]));

            // Register click area (accounting for scroll)
            let visible_row = (line_idx as i32) - (scroll as i32);
            if visible_row >= 0 && (row_start + visible_row as u16) < detail_area.y + detail_area.height {
                job_areas.push((Rect {
                    x: detail_area.x,
                    y: row_start + visible_row as u16,
                    width: detail_area.width,
                    height: 1,
                }, job.id, job.name.clone()));
            }
            line_idx += 1;

            for sub in &job.sub_jobs {
                let (sj_icon, sj_color) = match sub.status.as_str() {
                    "success" | "passed" => ("✓", t.success),
                    "failed" => ("✗", t.error),
                    "running" => (spinner, t.warning),
                    "pending" => ("◌", t.info),
                    "canceled" | "skipped" => ("○", t.text_dim),
                    _ => ("○", t.border),
                };
                let is_sel = app.selected_job_id == Some(sub.id);
                let sub_style = if is_sel {
                    Style::default().fg(t.bg).bg(t.accent)
                } else {
                    Style::default().fg(t.text_dim)
                };
                job_lines.push(Line::from(vec![
                    Span::styled(format!("          {} ", sj_icon), Style::default().fg(sj_color)),
                    Span::styled(sub.name.clone(), sub_style),
                ]));
                let visible_row = (line_idx as i32) - (scroll as i32);
                if visible_row >= 0 && (row_start + visible_row as u16) < detail_area.y + detail_area.height {
                    job_areas.push((Rect {
                        x: detail_area.x,
                        y: row_start + visible_row as u16,
                        width: detail_area.width,
                        height: 1,
                    }, sub.id, sub.name.clone()));
                }
                line_idx += 1;
            }
        }
    }
    app.click_regions.pipeline_detail.job_areas = job_areas;

    // Render header + jobs as a combined scrollable paragraph
    let mut all_lines = header_lines;
    all_lines.extend(job_lines);
    render_scrollable_lines(frame, &mut app.pipeline_detail_scroll, all_lines, detail_area, t);

    // Render job log panel
    if let Some(log_area) = log_area {
        render_job_log(frame, app, log_area);
    }
}

fn render_job_log(frame: &mut Frame, app: &mut App, area: Rect) {
    use ansi_to_tui::IntoText;
    let t = app.theme;

    // Left border
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner: header row + content
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ]).split(inner);

    // Header row with full background
    let header_area = chunks[0];
    let job_name = app.selected_job_name.as_deref().unwrap_or("job log");
    let title = format!(" log: {}", job_name);
    let close = "[X]";
    let padding = (header_area.width as usize)
        .saturating_sub(title.len() + close.len());
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(title, Style::default().fg(t.text).bg(t.header_bg).add_modifier(Modifier::BOLD)),
            Span::styled(" ".repeat(padding), Style::default().bg(t.header_bg)),
            Span::styled(close, Style::default().fg(t.text_dim).bg(t.header_bg)),
        ])),
        header_area,
    );

    // Register close area (both regions — one will be active depending on context)
    let close_x = header_area.x + header_area.width.saturating_sub(3);
    let job_log_close = Rect { x: close_x, y: header_area.y, width: 3, height: 1 };
    app.click_regions.pipeline_detail.job_log_close = Some(job_log_close);
    app.click_regions.mr_detail.job_log_close = Some(job_log_close);

    let inner = chunks[1];

    if app.job_log_loading {
        frame.render_widget(
            Paragraph::new(Span::styled("Loading...", Style::default().fg(t.text_dim))),
            inner,
        );
        return;
    }

    // Parse ANSI codes into styled ratatui Text
    let parsed = app.job_log.as_str().into_text().unwrap_or_default();
    let total_lines = parsed.lines.len();

    // Calculate line number gutter width
    let gutter_width = (total_lines.max(1).to_string().len() as u16) + 1;

    // Split inner: gutter | content
    let cols = Layout::horizontal([
        Constraint::Length(gutter_width),
        Constraint::Min(0),
    ]).split(inner);

    let content_height = total_lines as u16;
    let visible_height = cols[1].height;
    let max_scroll = content_height.saturating_sub(visible_height);
    if app.job_log_scroll > max_scroll {
        app.job_log_scroll = max_scroll;
    }

    // Render gutter with line numbers
    let gutter_lines: Vec<Line> = (1..=total_lines)
        .map(|n| Line::from(Span::styled(
            format!("{:>width$}", n, width = (gutter_width - 1) as usize),
            Style::default().fg(t.border),
        )))
        .collect();
    frame.render_widget(
        Paragraph::new(gutter_lines).scroll((app.job_log_scroll, 0)),
        cols[0],
    );

    // Render log content with ANSI colors
    frame.render_widget(
        Paragraph::new(parsed).scroll((app.job_log_scroll, 0)),
        cols[1],
    );

    if content_height > visible_height {
        let mut state = ScrollbarState::new(max_scroll as usize).position(app.job_log_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None).end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let sb_area = Rect { x: area.x + area.width.saturating_sub(1), y: inner.y, width: 1, height: inner.height };
        frame.render_stateful_widget(scrollbar, sb_area, &mut state);
    }
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
