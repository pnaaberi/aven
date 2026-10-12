use crate::render::format_bytes;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

use super::super::dialog::{Dialog, dialog_hint_line};
use super::super::scroll::{clamp_scroll_start, render_vertical_scrollbar};
use super::super::timestamps::optional_local_timestamp_display;
use crate::tui::config_overlay::DATABASE_STATS_TITLE;
use crate::tui::store::TuiDatabaseStats;
use crate::tui::theme::Theme;

pub(in crate::tui::ui) fn render_database_stats(
    frame: &mut Frame,
    theme: &Theme,
    stats: &TuiDatabaseStats,
    scroll: u16,
) {
    let width = frame.area().width.saturating_sub(8).clamp(72, 86);
    let lines = database_stats_lines(theme, stats);
    let height = frame.area().height.saturating_sub(1).clamp(12, 30);
    let content_rows = height.saturating_sub(2);
    let visible_rows = content_rows.saturating_sub(2) as usize;
    let start = clamp_scroll_start(scroll, lines.len(), visible_rows);
    let visible = lines
        .iter()
        .skip(start)
        .take(visible_rows)
        .cloned()
        .collect::<Vec<_>>();
    let dialog = if let Some(title) = scroll_title(scroll, lines.len(), visible_rows) {
        Dialog::new(theme, DATABASE_STATS_TITLE, width, height).right_title(Line::from(
            Span::styled(title, Style::new().fg(theme.fg_muted)),
        ))
    } else {
        Dialog::new(theme, DATABASE_STATS_TITLE, width, height)
    };
    let content = dialog.render_block(frame);
    let stats_area = Rect {
        height: content.height.saturating_sub(2),
        ..content
    };
    let hint_area = Rect {
        y: content.y + content.height.saturating_sub(1),
        height: 1,
        ..content
    };

    frame.render_widget(
        Paragraph::new(Text::from(visible)).style(Style::new().fg(theme.fg).bg(theme.bg_raised)),
        stats_area,
    );
    frame.render_widget(
        Paragraph::new(dialog_hint_line(
            theme,
            &[("j/k", "scroll"), ("Enter/Esc", "close")],
        ))
        .style(Style::new().fg(theme.fg).bg(theme.bg_raised)),
        hint_area,
    );
    render_vertical_scrollbar(frame, theme, stats_area, lines.len(), scroll);
}

fn database_stats_lines(theme: &Theme, stats: &TuiDatabaseStats) -> Vec<Line<'static>> {
    let sections = [
        section(
            theme,
            "workspace",
            vec![
                value_row(
                    theme,
                    "name",
                    stats.workspace_name.clone(),
                    Style::new().fg(theme.fg),
                ),
                value_row(
                    theme,
                    "key",
                    stats.workspace_key.clone(),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "tasks",
            vec![
                value_row(
                    theme,
                    "total",
                    stats.total_tasks.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "open",
                    stats.open_tasks.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "deleted",
                    stats.deleted_tasks.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "statuses",
            vec![
                value_row(
                    theme,
                    "inbox",
                    stats.statuses.inbox.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "backlog",
                    stats.statuses.backlog.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "todo",
                    stats.statuses.todo.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "active",
                    stats.statuses.active.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "done",
                    stats.statuses.done.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "canceled",
                    stats.statuses.canceled.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "priorities",
            vec![
                value_row(
                    theme,
                    "none",
                    stats.priorities.none.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "low",
                    stats.priorities.low.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "medium",
                    stats.priorities.medium.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "high",
                    stats.priorities.high.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "urgent",
                    stats.priorities.urgent.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "related rows",
            vec![
                value_row(
                    theme,
                    "projects",
                    stats.projects.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "labels",
                    stats.labels.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "notes",
                    stats.notes.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "task labels",
                    stats.task_labels.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "sync history",
            vec![
                count_row(theme, "change rows", stats.sync_history.total_change_rows),
                count_row(theme, "pending", stats.sync_history.pending_change_rows),
                count_row(theme, "synced", stats.sync_history.synced_change_rows),
                value_row(
                    theme,
                    "min server_seq",
                    format_optional_i64(stats.sync_history.min_server_seq),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "max server_seq",
                    format_optional_i64(stats.sync_history.max_server_seq),
                    Style::new().fg(theme.fg_muted),
                ),
                count_row(theme, "payload bytes", stats.sync_history.payload_bytes),
                count_row(theme, "conflicts", stats.conflicts),
            ],
        ),
        section(
            theme,
            "sqlite",
            vec![
                value_row(
                    theme,
                    "page size",
                    format!("{} bytes", stats.sqlite_page_size),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "pages",
                    stats.sqlite_page_count.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "free pages",
                    stats.sqlite_freelist_count.to_string(),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "main db size",
                    format_bytes(stats.sqlite_page_size * stats.sqlite_page_count),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "free",
                    format_bytes(stats.sqlite_page_size * stats.sqlite_freelist_count),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
        section(
            theme,
            "latest task timestamps",
            vec![
                value_row(
                    theme,
                    "created",
                    optional_local_timestamp_display(stats.latest_created_at.as_deref(), "none"),
                    Style::new().fg(theme.fg_muted),
                ),
                value_row(
                    theme,
                    "updated",
                    optional_local_timestamp_display(stats.latest_updated_at.as_deref(), "none"),
                    Style::new().fg(theme.fg_muted),
                ),
            ],
        ),
    ];

    let mut lines = Vec::new();
    for (pair_index, pair) in sections.chunks(2).enumerate() {
        let left = &pair[0];
        let right = pair.get(1);
        let rows = left
            .lines
            .len()
            .max(right.map_or(0, |section| section.lines.len()));
        for index in 0..rows {
            lines.push(two_column_line(
                left.lines.get(index),
                right.and_then(|section| section.lines.get(index)),
            ));
        }
        if pair_index + 1 < sections.len().div_ceil(2) {
            lines.push(Line::from(""));
        }
    }
    lines
}

struct StatsSection {
    lines: Vec<Line<'static>>,
}

fn section(theme: &Theme, title: &str, rows: Vec<Line<'static>>) -> StatsSection {
    let mut lines = vec![super::shared::section_line(theme, title)];
    lines.extend(rows);
    StatsSection { lines }
}

fn two_column_line(left: Option<&Line<'static>>, right: Option<&Line<'static>>) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(line) = left {
        spans.extend(line.spans.clone());
    }
    spans.push(Span::raw(
        " ".repeat(34usize.saturating_sub(line_width(left))),
    ));
    if let Some(line) = right {
        spans.extend(line.spans.clone());
    }
    Line::from(spans)
}

fn line_width(line: Option<&Line<'static>>) -> usize {
    line.map_or(0, Line::width)
}

const LABEL_WIDTH: usize = 14;

fn count_row(theme: &Theme, label: &str, value: i64) -> Line<'static> {
    super::shared::count_row(theme, LABEL_WIDTH, label, value)
}

fn value_row(
    theme: &Theme,
    label: &str,
    value: impl Into<String>,
    value_style: Style,
) -> Line<'static> {
    super::shared::value_row(theme, LABEL_WIDTH, label, value, value_style)
}

fn format_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn scroll_title(scroll: u16, content_height: usize, visible_rows: usize) -> Option<String> {
    if content_height <= visible_rows {
        return None;
    }
    let total = content_height
        .saturating_sub(visible_rows)
        .saturating_add(1);
    let current = clamp_scroll_start(scroll, content_height, visible_rows)
        .saturating_add(1)
        .min(total);
    Some(format!(" {current}/{total} "))
}

pub(crate) fn database_stats_scroll_cap(frame_height: u16, theme: &Theme) -> u16 {
    let content_rows = frame_height
        .saturating_sub(1)
        .clamp(12, 30)
        .saturating_sub(2);
    let visible_rows = content_rows.saturating_sub(2) as usize;
    database_stats_lines(theme, &TuiDatabaseStats::default())
        .len()
        .saturating_sub(visible_rows) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_width_counts_wide_value_cells() {
        let line = Line::from(vec![Span::raw("name          "), Span::raw("한글")]);

        assert_eq!(line_width(Some(&line)), 18);
    }
}
