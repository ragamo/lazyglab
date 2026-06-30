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
    pub tick_frame: u8,
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn spinner_char(tick: u8) -> &'static str {
    SPINNER_FRAMES[(tick as usize) % SPINNER_FRAMES.len()]
}

/// Build display lines for a single pipeline card.
/// Does not include the trailing separator — caller adds it.
pub fn pipeline_card_lines<'a>(p: &PipelineView<'_>, t: &'a Theme, area_width: u16) -> Vec<Line<'a>> {
    let _ = area_width;
    let mut lines: Vec<Line> = Vec::new();
    let spinner = spinner_char(p.tick_frame);

    let (icon, status_color) = if p.status == "running" {
        (spinner, t.warning)
    } else {
        status_style(p.status, t)
    };

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
            let (s_icon, s_color) = if stage.status == "running" {
                (spinner, t.warning)
            } else {
                status_style(&stage.status, t)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", s_icon), Style::default().fg(s_color)),
                Span::styled(stage.name.clone(), Style::default().fg(t.text)),
            ]));
            for job in &stage.jobs {
                let (j_icon, j_color) = if job.status == "running" {
                    (spinner, t.warning)
                } else {
                    status_style_job(&job.status, t)
                };
                let is_bridge = !job.sub_jobs.is_empty();
                lines.push(Line::from(vec![
                    Span::styled(format!("      {} ", j_icon), Style::default().fg(j_color)),
                    Span::styled(job.name.clone(), Style::default().fg(if is_bridge { t.text } else { t.text_dim })),
                ]));
                for sub in &job.sub_jobs {
                    let (sj_icon, sj_color) = if sub.status == "running" {
                        (spinner, t.warning)
                    } else {
                        status_style_job(&sub.status, t)
                    };
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
    let ts = parse_iso_to_unix(date_str);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = now - ts;
    if diff < 60 { "just now".to_string() }
    else if diff < 3600 {
        let m = diff / 60;
        if m == 1 { "1 min ago".to_string() } else { format!("{} mins ago", m) }
    } else if diff < 86400 {
        let h = diff / 3600;
        if h == 1 { "1 hour ago".to_string() } else { format!("{} hours ago", h) }
    } else if diff < 86400 * 30 {
        let d = diff / 86400;
        if d == 1 { "yesterday".to_string() } else { format!("{} days ago", d) }
    } else if diff < 86400 * 365 {
        let m = diff / (86400 * 30);
        if m == 1 { "1 month ago".to_string() } else { format!("{} months ago", m) }
    } else {
        let y = diff / (86400 * 365);
        if y == 1 { "1 year ago".to_string() } else { format!("{} years ago", y) }
    }
}

/// Parse ISO 8601 timestamp to Unix seconds without external deps.
/// Handles "2026-06-30T14:05:23.000Z" and "2026-06-30T14:05:23+00:00".
fn parse_iso_to_unix(s: &str) -> i64 {
    // Expect at least "YYYY-MM-DDTHH:MM:SS"
    if s.len() < 19 { return 0; }
    let b = s.as_bytes();
    let get = |i: usize, j: usize| -> Option<i64> {
        std::str::from_utf8(&b[i..j]).ok()?.parse().ok()
    };
    let (y, mo, d, h, mi, sec) = match (get(0,4), get(5,7), get(8,10), get(11,13), get(14,16), get(17,19)) {
        (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) => (y, mo, d, h, mi, s),
        _ => return 0,
    };
    // Days since epoch using Julian day number approach
    let days = days_from_ymd(y, mo, d);
    let mut unix = days * 86400 + h * 3600 + mi * 60 + sec;
    // Parse timezone offset if present (e.g. +02:00 or -05:00)
    if let Some(tz_start) = s[19..].find(|c| c == '+' || c == '-') {
        let tz = &s[19 + tz_start..];
        if tz.len() >= 6 {
            let sign: i64 = if tz.starts_with('+') { 1 } else { -1 };
            let th: i64 = tz[1..3].parse().unwrap_or(0);
            let tm: i64 = tz[4..6].parse().unwrap_or(0);
            unix -= sign * (th * 3600 + tm * 60);
        }
    }
    unix
}

fn days_from_ymd(y: i64, m: i64, d: i64) -> i64 {
    // Algorithm: days from 1970-01-01
    let m = if m <= 2 { m + 9 } else { m - 3 };
    let y = if m > 9 { y - 1 } else { y };
    let c = y / 100;
    let yr = y - 100 * c;
    (146097 * c) / 4 + (1461 * yr) / 4 + (153 * m + 2) / 5 + d - 719469
}
