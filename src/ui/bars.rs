use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
};

use crate::app::Device;

use super::theme::{CharSet, Theme};

pub(crate) fn render_brightness(
    area: Rect,
    frame: &mut Frame,
    device: &Device,
    theme: &Theme,
    char_set: &CharSet,
) {
    let layout = brightness_layout(area);
    let percent = device.brightness.min(100);
    let label = Line::from(Span::styled(format!("{percent}%"), theme.brightness))
        .alignment(Alignment::Right);
    frame.render_widget(label, layout.label);

    let filled_width = ((percent as f32 / 100.0) * layout.bar.width as f32).round() as usize;
    let filled = char_set.bar_filled.repeat(filled_width);
    let empty = char_set
        .bar_empty
        .repeat((layout.bar.width as usize).saturating_sub(filled_width));
    let bar = Line::from(vec![
        Span::styled(filled, theme.bar_filled),
        Span::styled(empty, theme.bar_empty),
    ]);
    frame.render_widget(bar, layout.bar);
}

pub(crate) fn brightness_value_at(area: Rect, column: u16, row: u16) -> Option<u16> {
    let bar_area = brightness_layout(area).bar;
    super::contains(bar_area, column, row).then(|| {
        let offset = column.saturating_sub(bar_area.x);
        let width = bar_area.width.max(1);
        ((offset as f32 / width as f32) * 100.0).round() as u16
    })
}

struct BrightnessLayout {
    label: Rect,
    bar: Rect,
}

fn brightness_layout(area: Rect) -> BrightnessLayout {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .horizontal_margin(2)
        .spacing(1)
        .split(area);

    BrightnessLayout {
        label: layout[0],
        bar: layout[1],
    }
}
