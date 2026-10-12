use ratatui::style::{Color, Modifier, Style};

/// Colors the TUI draws with, named by what each color is for.
///
/// `App` owns the active theme and render code receives it as `&Theme`.
/// Composite styles are methods over the base roles, so changing a base role
/// also changes every composite built from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Theme {
    /// Main surface. `Color::Reset` inherits the terminal background.
    pub(crate) bg: Color,
    /// Raised surfaces: dialogs, overlays, kanban cards, sidebar headers.
    pub(crate) bg_raised: Color,
    /// Secondary panels and unfocused selection.
    pub(crate) bg_panel: Color,
    /// Focused selection.
    pub(crate) bg_selected: Color,
    /// Rows marked for a bulk action.
    pub(crate) bg_row_marked: Color,
    /// Group heading rows in task lists.
    pub(crate) bg_row_group: Color,
    /// Kanban lane.
    pub(crate) bg_lane: Color,
    /// Emphasized kanban lane and rows related to the selection.
    pub(crate) bg_lane_active: Color,
    pub(crate) lane_divider: Color,

    pub(crate) fg: Color,
    pub(crate) fg_muted: Color,
    pub(crate) fg_dim: Color,
    /// Text drawn on a colored fill, such as badges and inverse labels.
    pub(crate) inverse_fg: Color,

    /// Borders, separators, rules, and tree rails.
    pub(crate) border: Color,
    pub(crate) scrollbar_track: Color,
    pub(crate) table_header_bg: Color,
    /// Fill behind neutral badges and unselected chips.
    pub(crate) badge_bg: Color,
    /// Fill behind the changelog's installed-version badge.
    pub(crate) badge_installed_bg: Color,

    pub(crate) accent: Color,
    pub(crate) info: Color,
    pub(crate) success: Color,
    pub(crate) warning: Color,
    pub(crate) danger: Color,
    /// Due today, epics, marks, and other things that want a second look.
    pub(crate) attention: Color,
    pub(crate) conflict: Color,

    /// Foreground of inline code and commands inside plain text.
    pub(crate) code: Color,
    pub(crate) link: Color,
    pub(crate) status_todo: Color,
    pub(crate) priority_medium: Color,
    pub(crate) priority_none: Color,

    pub(crate) custom_command_name: Color,
    pub(crate) custom_command_tag: Color,

    /// Label of the focused field in the add-task form.
    pub(crate) field_label_active: Color,
    /// Validation errors inside form fields.
    pub(crate) field_error: Color,
    /// Task labels in the add-task summary.
    pub(crate) task_label: Color,

    /// Vertical gradient stops of the splash logo, top to bottom.
    pub(crate) splash_gradient: [Color; 3],
    /// Check mark in the splash logo.
    pub(crate) splash_check: Color,
    /// Check mark color when the splash is fully dimmed.
    pub(crate) splash_check_dimmed: Color,
    /// Highlight of the ring that sweeps over the splash logo.
    pub(crate) splash_afterglow: Color,

    /// Project colors, picked by a stable hash of the project key.
    pub(crate) project_palette: [Color; 14],

    /// Text inside markdown block quotes.
    pub(crate) quote: Color,
}

impl Theme {
    pub(crate) const fn dark() -> Self {
        let bg_raised = Color::Rgb(34, 35, 33);
        let border = Color::Rgb(88, 88, 83);
        let blue = Color::Rgb(70, 128, 203);
        Self {
            bg: Color::Reset,
            bg_raised,
            bg_panel: Color::Rgb(39, 40, 38),
            bg_selected: Color::Rgb(50, 45, 78),
            bg_row_marked: bg_raised,
            bg_row_group: bg_raised,
            bg_lane: Color::Rgb(24, 25, 24),
            bg_lane_active: Color::Rgb(27, 26, 32),
            lane_divider: Color::Rgb(49, 50, 47),

            fg: Color::Rgb(239, 238, 232),
            fg_muted: Color::Rgb(191, 188, 180),
            fg_dim: Color::Rgb(147, 145, 138),
            inverse_fg: Color::Rgb(18, 19, 18),

            border,
            scrollbar_track: border,
            table_header_bg: border,
            badge_bg: border,
            badge_installed_bg: Color::Rgb(55, 56, 52),

            accent: Color::Rgb(166, 139, 255),
            info: blue,
            success: Color::Rgb(137, 199, 82),
            warning: Color::Rgb(244, 166, 54),
            danger: Color::Rgb(239, 82, 86),
            attention: Color::Rgb(255, 207, 87),
            conflict: Color::Rgb(225, 91, 139),

            code: blue,
            link: blue,
            status_todo: blue,
            priority_medium: Color::Rgb(137, 124, 232),
            priority_none: border,

            custom_command_name: Color::Rgb(224, 151, 92),
            custom_command_tag: Color::Rgb(190, 126, 82),

            field_label_active: Color::Rgb(194, 174, 255),
            field_error: Color::Red,
            task_label: Color::Rgb(133, 222, 255),

            splash_gradient: [
                Color::Rgb(135, 40, 250),
                Color::Rgb(150, 63, 255),
                Color::Rgb(176, 108, 255),
            ],
            splash_check: Color::White,
            splash_check_dimmed: Color::Rgb(85, 85, 85),
            splash_afterglow: Color::Rgb(232, 213, 255),

            project_palette: [
                Color::Rgb(174, 127, 255),
                Color::Rgb(60, 203, 162),
                Color::Rgb(255, 177, 74),
                Color::Rgb(255, 116, 92),
                Color::Rgb(242, 112, 166),
                Color::Rgb(149, 213, 85),
                Color::Rgb(92, 181, 255),
                Color::Rgb(255, 207, 87),
                Color::Rgb(120, 223, 225),
                Color::Rgb(199, 143, 255),
                Color::Rgb(255, 139, 104),
                Color::Rgb(126, 220, 135),
                Color::Rgb(123, 156, 255),
                Color::Rgb(232, 128, 214),
            ],

            quote: Color::Rgb(137, 199, 82),
        }
    }

    pub(crate) const fn selected(&self) -> Style {
        Style::new()
            .fg(self.fg)
            .bg(self.bg_selected)
            .add_modifier(Modifier::BOLD)
    }

    pub(crate) const fn selected_inactive(&self) -> Style {
        Style::new().fg(self.fg_muted).bg(self.bg_panel)
    }

    /// Rows related to the selected task.
    pub(crate) const fn related(&self) -> Style {
        Style::new().fg(self.fg).bg(self.bg_lane_active)
    }

    /// Inline code and commands inside plain text.
    pub(crate) const fn code(&self) -> Style {
        Style::new().fg(self.code).bg(self.bg_panel)
    }

    /// Drawn text cursor. `text_cursor_position` finds the caret by this exact
    /// foreground/background pair, so nothing else may use it.
    pub(crate) const fn cursor(&self) -> Style {
        Style::new().fg(self.bg_raised).bg(self.fg)
    }

    pub(crate) fn priority(&self, priority: &str) -> Style {
        let color = match priority {
            "urgent" => self.danger,
            "high" => self.warning,
            "medium" => self.priority_medium,
            "low" => self.fg_dim,
            _ => self.priority_none,
        };
        Style::new().fg(color)
    }

    pub(crate) fn status(&self, status: &str) -> Style {
        let color = match status {
            "active" => self.accent,
            "todo" => self.status_todo,
            "inbox" => self.fg_dim,
            "backlog" => self.fg_muted,
            "done" => self.success,
            "canceled" => self.danger,
            _ => self.fg_dim,
        };
        Style::new().fg(color)
    }

    pub(crate) fn project(&self, key: &str) -> Color {
        let hash = key
            .bytes()
            .fold(5381usize, |acc, byte| acc.wrapping_mul(33) ^ byte as usize);
        self.project_palette[hash % self.project_palette.len()]
    }
}

// Aliases of `Theme::dark()` kept while render code migrates to `&Theme`.
pub(crate) const INVERSE_FG: Color = Theme::dark().inverse_fg;
pub(crate) const BG_ALT: Color = Theme::dark().bg_raised;
pub(crate) const BG_PANEL: Color = Theme::dark().bg_panel;
pub(crate) const BG_LANE: Color = Theme::dark().bg_lane;
pub(crate) const BG_LANE_ACTIVE: Color = Theme::dark().bg_lane_active;
pub(crate) const LANE_DIVIDER: Color = Theme::dark().lane_divider;
pub(crate) const FG: Color = Theme::dark().fg;
pub(crate) const FG_MUTED: Color = Theme::dark().fg_muted;
pub(crate) const FG_DIM: Color = Theme::dark().fg_dim;
pub(crate) const BORDER: Color = Theme::dark().border;
pub(crate) const SELECTED_BG: Color = Theme::dark().bg_selected;
pub(crate) const ACCENT: Color = Theme::dark().accent;
pub(crate) const BLUE: Color = Theme::dark().info;
pub(crate) const ORANGE: Color = Theme::dark().warning;
pub(crate) const RED: Color = Theme::dark().danger;
pub(crate) const PINK: Color = Theme::dark().conflict;
pub(crate) const YELLOW: Color = Theme::dark().attention;
pub(crate) const GREEN: Color = Theme::dark().success;
pub(crate) const SELECTED: Style = Theme::dark().selected();
pub(crate) const RELATED: Style = Theme::dark().related();
pub(crate) const SELECTED_INACTIVE: Style = Theme::dark().selected_inactive();
pub(crate) const BG: Color = Theme::dark().bg;

pub(crate) fn priority_style(priority: &str) -> Style {
    Theme::dark().priority(priority)
}

pub(crate) fn status_style(status: &str) -> Style {
    Theme::dark().status(status)
}

pub(crate) fn project_color(key: &str) -> Color {
    Theme::dark().project(key)
}
