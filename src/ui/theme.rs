use ratatui::style::{Color, Style};

pub(crate) struct Theme {
    pub(crate) selector: Style,
    pub(crate) title: Style,
    pub(crate) detail: Style,
    pub(crate) backend: Style,
    pub(crate) brightness: Style,
    pub(crate) bar_empty: Style,
    pub(crate) bar_filled: Style,
    pub(crate) list_more: Style,
    pub(crate) tab: Style,
    pub(crate) tab_selected: Style,
    pub(crate) tab_marker: Style,
    pub(crate) overlay_border: Style,
    pub(crate) notification_border: Style,
    pub(crate) notification_text: Style,
    pub(crate) help: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            selector: Style::default().fg(Color::LightCyan),
            title: Style::default(),
            detail: Style::default().fg(Color::DarkGray),
            backend: Style::default().fg(Color::DarkGray),
            brightness: Style::default(),
            bar_empty: Style::default().fg(Color::DarkGray),
            bar_filled: Style::default().fg(Color::LightBlue),
            list_more: Style::default().fg(Color::DarkGray),
            tab: Style::default(),
            tab_selected: Style::default().fg(Color::LightCyan),
            tab_marker: Style::default().fg(Color::LightCyan),
            overlay_border: Style::default(),
            notification_border: Style::default().fg(Color::LightCyan),
            notification_text: Style::default(),
            help: Style::default().fg(Color::DarkGray),
        }
    }
}

pub(crate) struct CharSet {
    pub(crate) selector_top: &'static str,
    pub(crate) selector_middle: &'static str,
    pub(crate) selector_bottom: &'static str,
    pub(crate) tab_marker_left: &'static str,
    pub(crate) tab_marker_right: &'static str,
    pub(crate) list_more: &'static str,
    pub(crate) bar_empty: &'static str,
    pub(crate) bar_filled: &'static str,
}

impl Default for CharSet {
    fn default() -> Self {
        Self {
            selector_top: "░",
            selector_middle: "▒",
            selector_bottom: "░",
            tab_marker_left: "[",
            tab_marker_right: "]",
            list_more: "•••",
            bar_empty: "╌",
            bar_filled: "━",
        }
    }
}
