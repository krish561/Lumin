use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
};

use crate::app::{ActiveSection, App};

use super::theme::{CharSet, Theme};

pub(crate) fn render(area: Rect, frame: &mut Frame, app: &App, theme: &Theme, char_set: &CharSet) {
    let layout = footer_layout(area);
    render_display_sections(layout.sections, frame, app, theme, char_set);

    let line = Line::from("1-5 sections   ↑/↓ select   ←/→ adjust   q quit")
        .style(theme.help)
        .alignment(Alignment::Right);
    frame.render_widget(line, layout.help);
}

pub fn section_at(area: Rect, column: u16, row: u16) -> Option<ActiveSection> {
    let footer_area = super::footer_area(area);
    if row != footer_area.y || column < footer_area.x || column >= footer_area.right() {
        return None;
    }

    let layout = footer_layout(footer_area);
    let section_areas = section_areas(layout.sections);

    ActiveSection::ALL
        .iter()
        .zip(section_areas.iter())
        .find_map(|(section, area)| super::contains(*area, column, row).then_some(*section))
}

fn render_display_sections(
    area: Rect,
    frame: &mut Frame,
    app: &App,
    theme: &Theme,
    char_set: &CharSet,
) {
    let areas = section_areas(area);

    for (section, area) in ActiveSection::ALL.iter().zip(areas.iter()) {
        let selected = app.ui.active_section == Some(*section);
        let line = if selected {
            Line::from(vec![
                Span::styled(char_set.tab_marker_left, theme.tab_marker),
                Span::styled(section.title(), theme.tab_selected),
                Span::styled(char_set.tab_marker_right, theme.tab_marker),
            ])
        } else {
            Line::from(Span::styled(format!(" {} ", section.title()), theme.tab))
        };

        frame.render_widget(line, *area);
    }
}

struct FooterLayout {
    sections: Rect,
    help: Rect,
}

fn footer_layout(area: Rect) -> FooterLayout {
    // Widen help area to fit the longer hint text
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(52)])
        .split(area);

    FooterLayout {
        sections: layout[0],
        help: layout[1],
    }
}

fn section_areas(area: Rect) -> Vec<Rect> {
    let constraints: Vec<_> = ActiveSection::ALL
        .iter()
        .map(|section| Constraint::Length(section.title().len() as u16 + 2))
        .collect();
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
        .to_vec()
}
