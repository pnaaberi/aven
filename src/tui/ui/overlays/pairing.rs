use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::super::dialog::{Dialog, dialog_hint_line, dialog_hint_width};
use super::super::inline_code::{wrap_with_code, wrapped_code_rows};
use crate::pairing::{PairingPresentation, PairingQr};
use crate::tui::overlay::{PairingOverlay, dialog_area, dialog_inner_area};
use crate::tui::theme::Theme;

pub(crate) const NETWORK_REQUIREMENT: &str = "Anyone with this code can access all synced data.";
const PAIRING_TITLE: &str = "Sync › Add device";
const HANDOFF: &str = "On the other device: Join existing sync, or `aven sync join`.";
const DIALOG_CHROME_COLUMNS: u16 = 4;
const DIALOG_CHROME_ROWS: u16 = 2;
const QR_GAP_ROWS: u16 = 0;
const FOOTER_ROWS: u16 = 1;
const FALLBACK_WIDTH: u16 = 64;
const READY_HINTS: &[(&str, &str)] = &[("c", "copy invitation"), ("Esc", "back")];
/// Black modules on a white field regardless of theme, since scanners read
/// dark-on-light codes most reliably. Explicit RGB, because terminal themes can
/// remap the named black and white.
const QR_STYLE: Style = Style::new()
    .fg(Color::Rgb(0, 0, 0))
    .bg(Color::Rgb(255, 255, 255));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PairingLayout {
    pub(crate) area: Rect,
    pub(crate) inner: Rect,
    pub(crate) content: Rect,
    pub(crate) qr: Option<Rect>,
    pub(crate) footer: Rect,
}

pub(crate) fn pairing_layout(terminal: Rect, presentation: &PairingPresentation) -> PairingLayout {
    let qr_width = u16::try_from(presentation.qr().width()).unwrap_or(u16::MAX);
    let qr_height = u16::try_from(presentation.qr().rows().len()).unwrap_or(u16::MAX);
    let available_width = terminal.width.saturating_sub(2);
    // The dialog is as wide as the other Sync pages so the copy reads well,
    // widens to hold the QR, and never cuts off the footer hints.
    let min_width = qr_width
        .max(hint_width(READY_HINTS))
        .saturating_add(DIALOG_CHROME_COLUMNS);
    let width = min_width.max(FALLBACK_WIDTH).min(available_width);
    let inner_width = width.saturating_sub(DIALOG_CHROME_COLUMNS);
    let header_height = qr_header_height(presentation, inner_width);
    let required_height = header_height
        .saturating_add(QR_GAP_ROWS)
        .saturating_add(qr_height)
        .saturating_add(QR_GAP_ROWS)
        .saturating_add(FOOTER_ROWS)
        .saturating_add(DIALOG_CHROME_ROWS);
    let fits = min_width <= available_width && required_height <= terminal.height.saturating_sub(2);

    if fits {
        let area = dialog_area(terminal, width, required_height);
        let inner = dialog_inner_area(area);
        let content = Rect::new(inner.x, inner.y, inner.width, header_height);
        let qr = Rect::new(
            inner.x + inner.width.saturating_sub(qr_width) / 2,
            content.bottom().saturating_add(QR_GAP_ROWS),
            qr_width,
            qr_height,
        );
        return PairingLayout {
            area,
            inner,
            content,
            qr: Some(qr),
            footer: footer_rect(inner),
        };
    }

    let width = FALLBACK_WIDTH.min(terminal.width.saturating_sub(2));
    let inner_width = width.saturating_sub(DIALOG_CHROME_COLUMNS);
    let body_height = wrapped_height(&fallback_text(), inner_width);
    let height = body_height
        .saturating_add(FOOTER_ROWS)
        .saturating_add(DIALOG_CHROME_ROWS);
    let area = dialog_area(terminal, width, height);
    let inner = dialog_inner_area(area);
    PairingLayout {
        area,
        inner,
        content: Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(FOOTER_ROWS),
        ),
        qr: None,
        footer: footer_rect(inner),
    }
}

/// The dialog area for any state of the page, for hit testing.
pub(crate) fn pairing_area(terminal: Rect, pairing: &PairingOverlay) -> Rect {
    match pairing {
        PairingOverlay::Ready(presentation) => pairing_layout(terminal, presentation).area,
        _ => status_layout(terminal, status_rows(pairing, FALLBACK_WIDTH)).0,
    }
}

pub(in crate::tui::ui) fn render_pairing(
    frame: &mut Frame,
    theme: &Theme,
    pairing: &PairingOverlay,
) {
    match pairing {
        PairingOverlay::Ready(presentation) => render_presentation(frame, theme, presentation),
        _ => render_status(frame, theme, pairing),
    }
}

/// Lines for the page before its QR code is available.
fn status_lines(theme: &Theme, pairing: &PairingOverlay, width: u16) -> Vec<Line<'static>> {
    let inner = usize::from(width.saturating_sub(DIALOG_CHROME_COLUMNS));
    match pairing {
        PairingOverlay::Creating { started_at } => vec![Line::from(vec![
            Span::styled(
                format!("{} ", super::sync_dialog::spinner(*started_at)),
                Style::new().fg(theme.accent),
            ),
            Span::styled("Creating invitation…", Style::new().fg(theme.fg)),
        ])],
        PairingOverlay::Failed(message) => {
            let mut lines = vec![Line::from(Span::styled(
                "! Couldn't create the invitation",
                Style::new().fg(theme.danger),
            ))];
            lines.extend(wrap_with_code(
                theme,
                message,
                Style::new().fg(theme.fg),
                inner,
            ));
            lines
        }
        PairingOverlay::Ready(_) => Vec::new(),
    }
}

/// Row count of `status_lines`, which hit testing needs without a theme.
fn status_rows(pairing: &PairingOverlay, width: u16) -> usize {
    let inner = usize::from(width.saturating_sub(DIALOG_CHROME_COLUMNS));
    match pairing {
        PairingOverlay::Creating { .. } => 1,
        PairingOverlay::Failed(message) => 1 + wrapped_code_rows(message, inner),
        PairingOverlay::Ready(_) => 0,
    }
}

fn status_layout(terminal: Rect, rows: usize) -> (Rect, u16) {
    let width = FALLBACK_WIDTH.min(terminal.width.saturating_sub(2));
    let body = u16::try_from(rows).unwrap_or(u16::MAX);
    let height = body
        .saturating_add(FOOTER_ROWS)
        .saturating_add(DIALOG_CHROME_ROWS);
    (dialog_area(terminal, width, height), body)
}

fn render_status(frame: &mut Frame, theme: &Theme, pairing: &PairingOverlay) {
    let width = FALLBACK_WIDTH.min(frame.area().width.saturating_sub(2));
    let lines = status_lines(theme, pairing, width);
    let (area, body) = status_layout(frame.area(), lines.len());
    let inner =
        Dialog::new(theme, PAIRING_TITLE, area.width, area.height).render_block_at(frame, area);
    frame.render_widget(
        Paragraph::new(lines).style(Style::new().bg(theme.bg_raised)),
        Rect::new(inner.x, inner.y, inner.width, body.min(inner.height)),
    );
    let hints: &[(&str, &str)] = match pairing {
        PairingOverlay::Failed(_) => &[("Enter", "retry"), ("Esc", "back")],
        _ => &[("Esc", "back")],
    };
    render_hints(frame, theme, footer_rect(inner), hints);
}

fn render_presentation(frame: &mut Frame, theme: &Theme, presentation: &PairingPresentation) {
    let layout = pairing_layout(frame.area(), presentation);
    let inner = Dialog::new(theme, PAIRING_TITLE, layout.area.width, layout.area.height)
        .render_block_at(frame, layout.area);
    debug_assert_eq!(inner, layout.inner);

    if let Some(qr) = layout.qr {
        render_qr_header(frame, theme, layout.content, presentation);
        render_qr(frame, qr, presentation.qr());
    } else {
        render_text(frame, theme, layout.content, &fallback_text(), theme.fg);
    }
    render_hints(frame, theme, layout.footer, READY_HINTS);
}

fn waiting_text(presentation: &PairingPresentation) -> String {
    let remaining = presentation
        .expires_at()
        .saturating_sub(crate::sync::encrypted::unix_now().unwrap_or_default());
    format!(
        "Waiting for a device · {}:{:02} left",
        remaining / 60,
        remaining % 60
    )
}

fn qr_header_height(presentation: &PairingPresentation, width: u16) -> u16 {
    wrapped_height(&waiting_text(presentation), width)
        .saturating_add(wrapped_height(
            &format!("Server: {}", presentation.server_identity()),
            width,
        ))
        .saturating_add(wrapped_height(HANDOFF, width))
        .saturating_add(wrapped_height(NETWORK_REQUIREMENT, width))
}

fn render_qr_header(
    frame: &mut Frame,
    theme: &Theme,
    area: Rect,
    presentation: &PairingPresentation,
) {
    let waiting = waiting_text(presentation);
    let waiting_height = wrapped_height(&waiting, area.width);
    render_text(
        frame,
        theme,
        Rect::new(area.x, area.y, area.width, waiting_height.min(area.height)),
        &waiting,
        theme.fg,
    );

    let server = format!("Server: {}", presentation.server_identity());
    let server_height = wrapped_height(&server, area.width);
    let server_y = area.y.saturating_add(waiting_height);
    render_text(
        frame,
        theme,
        Rect::new(
            area.x,
            server_y,
            area.width,
            server_height.min(area.bottom().saturating_sub(server_y)),
        ),
        &server,
        theme.fg_muted,
    );

    let handoff_height = wrapped_height(HANDOFF, area.width);
    let handoff_y = server_y.saturating_add(server_height);
    render_text(
        frame,
        theme,
        Rect::new(
            area.x,
            handoff_y,
            area.width,
            handoff_height.min(area.bottom().saturating_sub(handoff_y)),
        ),
        HANDOFF,
        theme.fg_muted,
    );

    let requirement_y = handoff_y.saturating_add(handoff_height);
    render_text(
        frame,
        theme,
        Rect::new(
            area.x,
            requirement_y,
            area.width,
            area.bottom().saturating_sub(requirement_y),
        ),
        NETWORK_REQUIREMENT,
        theme.fg_muted,
    );
}

fn render_qr(frame: &mut Frame, area: Rect, qr: &PairingQr) {
    let lines = qr
        .rows()
        .iter()
        .map(|row| Line::from(Span::styled(row.as_str(), QR_STYLE)))
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines).style(QR_STYLE), area);
}

fn render_hints(frame: &mut Frame, theme: &Theme, area: Rect, hints: &[(&str, &str)]) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(dialog_hint_line(theme, hints)).style(Style::new().bg(theme.bg_raised)),
        area,
    );
}

fn hint_width(hints: &[(&str, &str)]) -> u16 {
    u16::try_from(dialog_hint_width(hints)).unwrap_or(u16::MAX)
}

fn render_text(frame: &mut Frame, theme: &Theme, area: Rect, text: &str, color: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let style = Style::new().fg(color).bg(theme.bg_raised);
    frame.render_widget(
        Paragraph::new(wrap_text(theme, text, area.width, style)).style(style),
        area,
    );
}

fn fallback_text() -> String {
    format!(
        "Terminal too small for QR. Press c to copy, enlarge it, or run `aven sync invite`.\nOn other device: Join existing sync or `aven sync join`.\n{NETWORK_REQUIREMENT}"
    )
}

fn footer_rect(inner: Rect) -> Rect {
    Rect::new(
        inner.x,
        inner.bottom().saturating_sub(FOOTER_ROWS),
        inner.width,
        u16::from(inner.height >= FOOTER_ROWS),
    )
}

/// Number of lines `wrap_text` produces.
fn wrapped_height(text: &str, width: u16) -> u16 {
    let rows = text
        .split('\n')
        .map(|paragraph| wrapped_code_rows(paragraph, usize::from(width)))
        .sum::<usize>();
    u16::try_from(rows).unwrap_or(u16::MAX)
}

fn wrap_text(theme: &Theme, text: &str, width: u16, style: Style) -> Vec<Line<'static>> {
    text.split('\n')
        .flat_map(|paragraph| wrap_with_code(theme, paragraph, style, usize::from(width)))
        .collect()
}
