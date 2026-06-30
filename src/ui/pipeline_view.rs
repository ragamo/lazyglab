use ratatui::prelude::*;

use crate::provider::types::StageStatus;
use crate::theme::Theme;

pub struct PipelineView<'a> {
    pub id: u64,
    pub status: &'a str,
    pub r#ref: &'a str,
    pub created_at: &'a str,
    pub duration: Option<u64>,
    pub user: Option<&'a str>,
    pub mr_iid: Option<u64>,
    pub mr_title: Option<&'a str>,
    pub stages: &'a [StageStatus],
}

/// Build display lines for a single pipeline card.
/// Does not include the trailing separator — caller adds it.
pub fn pipeline_card_lines<'a>(p: &PipelineView<'_>, t: &'a Theme, area_width: u16) -> Vec<Line<'a>> {
    let _ = area_width;
    let mut lines: Vec<Line> = Vec::new();

    let (icon, status_color) = status_style(p.status, t);

    let duration_str = p.duration
        .map(|d| {
            let mins = d / 60;
            let secs = d % 60;
            if mins > 0 { format!("{}m {}s", mins, secs) } else { format!("{}s", secs) }
        })
        .unwrap_or_else(|| "—".to_string());

    let time_ago = format_time_ago(p.created_at);

    // Line 1: status | duration | time ago
    lines.push(Line::from(vec![
        Span::styled(format!("{} {}", icon, p.status), Style::default().fg(status_color)),
        Span::styled("  │  ", Style::default().fg(t.text_dim)),
        Span::styled(format!("⏱ {}", duration_str), Style::default().fg(t.text)),
        Span::styled("  │  ", Style::default().fg(t.text_dim)),
        Span::styled(time_ago, Style::default().fg(t.text_dim)),
    ]));

    // Line 2: ID + ref + user
    let user_str = p.user
        .map(|u| format!("@{}", u))
        .unwrap_or_else(|| "…".to_string());
    let mut id_line = vec![
        Span::styled(format!("#{}", p.id), Style::default().fg(t.accent)),
        Span::styled(" — ", Style::default().fg(t.text_dim)),
        Span::styled(p.r#ref.to_string(), Style::default().fg(t.info)),
        Span::styled(" by ", Style::default().fg(t.text_dim)),
        Span::styled(user_str, Style::default().fg(t.warning)),
    ];
    if let (Some(iid), Some(title)) = (p.mr_iid, p.mr_title) {
        id_line.push(Span::styled("  │  ", Style::default().fg(t.text_dim)));
        id_line.push(Span::styled(format!("!{}", iid), Style::default().fg(t.accent)));
        id_line.push(Span::styled(format!(" {}", title), Style::default().fg(t.text)));
    }
    lines.push(Line::from(id_line));

    // Stages with jobs
    if !p.stages.is_empty() {
        for stage in p.stages {
            let (s_icon, s_color) = status_style(&stage.status, t);
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", s_icon), Style::default().fg(s_color)),
                Span::styled(stage.name.clone(), Style::default().fg(t.text)),
            ]));
            for job in &stage.jobs {
                let (j_icon, j_color) = status_style_job(&job.status, t);
                let is_bridge = !job.sub_jobs.is_empty();
                lines.push(Line::from(vec![
                    Span::styled(format!("      {} ", j_icon), Style::default().fg(j_color)),
                    Span::styled(job.name.clone(), Style::default().fg(if is_bridge { t.text } else { t.text_dim })),
                ]));
                for sub in &job.sub_jobs {
                    let (sj_icon, sj_color) = status_style_job(&sub.status, t);
                    lines.push(Line::from(vec![
                        Span::styled(format!("          {} ", sj_icon), Style::default().fg(sj_color)),
                        Span::styled(sub.name.clone(), Style::default().fg(t.text_dim)),
                    ]));
                }
            }
        }
    }

    lines
}

fn status_style(status: &str, t: &Theme) -> (&'static str, Color) {
    match status {
        "success" | "passed" => ("✓", t.success),
        "failed" => ("✗", t.error),
        "running" => ("●", t.warning),
        "pending" => ("◌", t.info),
        "canceled" | "skipped" => ("○", t.text_dim),
        _ => ("?", t.border),
    }
}

fn status_style_job(status: &str, t: &Theme) -> (&'static str, Color) {
    match status {
        "success" | "passed" => ("✓", t.success),
        "failed" => ("✗", t.error),
        "running" => ("●", t.warning),
        "pending" => ("◌", t.info),
        "canceled" | "skipped" => ("○", t.text_dim),
        _ => ("○", t.border),
    }
}

fn format_time_ago(date_str: &str) -> String {
    if date_str.len() < 10 { return date_str.to_string(); }
    let date_part = &date_str[..10];
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 { return date_part.to_string(); }
    let (y, m, d) = match (parts[0].parse::<i64>(), parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
        (Ok(y), Ok(m), Ok(d)) => (y, m, d),
        _ => return date_part.to_string(),
    };
    let date_days = y * 365 + m * 30 + d;
    let now_days = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (now / 86400) as i64
    };
    let epoch_offset = 1970 * 365 + 1 * 30 + 1;
    let diff = now_days - (date_days - epoch_offset);
    if diff <= 0 { "today".to_string() }
    else if diff == 1 { "yesterday".to_string() }
    else if diff < 30 { format!("{} days ago", diff) }
    else if diff < 365 {
        let m = diff / 30;
        if m == 1 { "1 month ago".to_string() } else { format!("{} months ago", m) }
    } else {
        let y = diff / 365;
        if y == 1 { "1 year ago".to_string() } else { format!("{} years ago", y) }
    }
}
