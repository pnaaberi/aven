use ratatui::style::Style;
use ratatui::text::Span;

use crate::tui::theme::Theme;

pub(super) fn linked_task_ref_spans(
    theme: &Theme,
    display_ref: &str,
    project_key: &str,
) -> Vec<Span<'static>> {
    if let Some((prefix, suffix)) = display_ref.split_once('-') {
        vec![
            Span::styled(
                prefix.to_string(),
                Style::new().fg(theme.project(project_key)),
            ),
            Span::styled("-", theme.dim()),
            Span::styled(suffix.to_string(), theme.dim()),
        ]
    } else {
        vec![Span::styled(display_ref.to_string(), theme.dim())]
    }
}

pub(super) fn labels_display(labels: &[String], separator: &str) -> String {
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(separator)
    }
}

pub(super) fn description_or_placeholder(description: &str) -> String {
    if description.is_empty() {
        "(no description)".to_string()
    } else {
        description.to_string()
    }
}
