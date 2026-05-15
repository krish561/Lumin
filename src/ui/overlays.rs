use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, Borders, Clear, Padding},
};

use crate::app::{ActiveSection, App};

use super::theme::Theme;

pub(crate) fn render_section_window(
    area: Rect,
    frame: &mut Frame,
    app: &App,
    section: ActiveSection,
    theme: &Theme,
) {
    let width = area.width.min(54);
    let height = area.height.min(10);
    let [window_area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [window_area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(window_area);

    frame.render_widget(Clear, window_area);

    let block = Block::default()
        .title(section.title())
        .borders(Borders::ALL)
        .border_style(theme.overlay_border)
        .padding(Padding::horizontal(2));
    let inner = block.inner(window_area);
    frame.render_widget(block, window_area);

    let lines = section_lines(app, section);
    let constraints = lines
        .iter()
        .map(|_| Constraint::Length(1))
        .collect::<Vec<_>>();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (line, area) in lines.into_iter().zip(rows.iter()) {
        frame.render_widget(line, *area);
    }
}

fn section_lines(app: &App, section: ActiveSection) -> Vec<Line<'static>> {
    let selected = app.devices.get(app.selected);

    match section {
        ActiveSection::Displays => vec![
            Line::from("Connected displays"),
            Line::from(format!("Detected: {}", app.devices.len())),
            Line::from(format!(
                "Selected: {}",
                selected
                    .map(|device| device.description.as_str())
                    .unwrap_or("none")
            )),
            Line::from("Placeholder: per-display enable, mirror, and layout controls"),
        ],
        ActiveSection::Profiles => vec![
            Line::from("Brightness profiles"),
            Line::from("Placeholder: indoor, outdoor, battery, and custom presets"),
            Line::from("Placeholder: assign profile per display"),
        ],
        ActiveSection::Gamma => vec![
            Line::from("Gamma controls"),
            Line::from("Placeholder: red, green, and blue channel curves"),
            Line::from("Placeholder: contrast and white-point tuning"),
        ],
        ActiveSection::Night => vec![
            Line::from("Night light"),
            Line::from("Placeholder: warm temperature schedule"),
            Line::from("Placeholder: sunrise/sunset and manual override"),
        ],
    }
}
