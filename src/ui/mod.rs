mod bars;
mod devices;
mod footer;
mod notifications;
mod overlays;
mod theme;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
};

use crate::app::{App, WindowMode};

use self::theme::{CharSet, Theme};

pub use devices::{brightness_at, device_at};
pub use footer::section_at;

pub fn mode_confirm_button_at(
    area: Rect,
    column: u16,
    row: u16,
) -> Option<crate::app::ModeConfirmChoice> {
    overlays::mode_confirm_button_at(area, column, row)
}

pub fn render(frame: &mut Frame, app: &App) {
    let theme = Theme::default();
    let char_set = CharSet::default();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(layout[0], frame, app, &theme);
    devices::render(layout[1], frame, app, &theme, &char_set);
    footer::render(layout[2], frame, app, &theme, &char_set);

    if let Some(section) = app.ui.active_section {
        overlays::render_section_window(frame.area(), frame, app, section, &theme);
    }

    if app.pending_mode_revert.is_some() {
        overlays::render_mode_confirm_popup(frame.area(), frame, app, &theme);
    }
    // Collect up to 3 messages from the front of the queue, newest first
    let stack: Vec<&str> = app
        .ui
        .notifications
        .iter()
        .rev()
        .take(3)
        .map(|n| n.message.as_str())
        .collect();

    if !stack.is_empty() {
        notifications::render_stack(frame.area(), frame, &stack, &theme);
    }
} // <-- this closes render()

fn render_header(area: Rect, frame: &mut Frame, app: &App, theme: &Theme) {
    let mode = match app.ui.window_mode {
        WindowMode::Inline => "inline",
        WindowMode::Floating => "floating",
    };
    let line = Line::from(vec![
        Span::styled("Lumin", theme.title),
        Span::styled("  brightness controller", theme.detail),
        Span::styled(format!("  {mode}"), theme.detail),
    ]);
    frame.render_widget(line, area);
}

pub(crate) fn content_area(area: Rect) -> Rect {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area)[1]
}

pub(crate) fn footer_area(area: Rect) -> Rect {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area)[2]
}

pub(crate) fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x && column < area.right() && row >= area.y && row < area.bottom()
}
