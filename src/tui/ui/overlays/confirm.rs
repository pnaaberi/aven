use ratatui::Frame;
use ratatui::text::{Line, Text};

use super::super::dialog::{Dialog, dialog_hint_line};
use crate::tui::overlay::{ConfirmView, confirm_width};
use crate::tui::text::cell_width_ranges;
use crate::tui::theme::Theme;

pub(in crate::tui::ui) fn render_confirm(frame: &mut Frame, theme: &Theme, state: &ConfirmView) {
    render_confirm_with_hint_line(frame, theme, state, confirm_hint_line(theme));
}

pub(in crate::tui::ui) fn render_confirm_with_hints(
    frame: &mut Frame,
    theme: &Theme,
    state: &ConfirmView,
    hints: &[(&str, &str)],
) {
    render_confirm_with_hint_line(frame, theme, state, dialog_hint_line(theme, hints));
}

fn render_confirm_with_hint_line(
    frame: &mut Frame,
    theme: &Theme,
    state: &ConfirmView,
    hints: Line<'static>,
) {
    let width = confirm_width(frame.area().width, &state.prompt)
        .max(confirm_width(frame.area().width, &hints.to_string()));
    let prompt_rows = cell_width_ranges(&state.prompt, width.saturating_sub(4) as usize).len();
    let height = prompt_rows.saturating_add(4) as u16;
    let text = Text::from(vec![
        Line::from(state.prompt.as_str()),
        Line::from(""),
        hints,
    ]);
    Dialog::new(theme, &state.title, width, height)
        .wrap()
        .render_text(frame, text);
}

pub(in crate::tui::ui) fn confirm_hint_line(theme: &Theme) -> ratatui::text::Line<'static> {
    dialog_hint_line(theme, &[("y", "yes"), ("n", "no"), ("Esc", "cancel")])
}
