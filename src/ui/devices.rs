use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Span},
};

use crate::app::{App, BrightnessBackend, Device};

use super::{
    bars,
    theme::{CharSet, Theme},
};

const ROW_HEIGHT: u16 = 3;
const ROW_SPACING: u16 = 2;

pub(crate) fn render(area: Rect, frame: &mut Frame, app: &App, theme: &Theme, char_set: &CharSet) {
    if app.devices.is_empty() {
        let line = Line::from("No brightness devices found").style(theme.detail);
        frame.render_widget(line.alignment(Alignment::Center), area);
        return;
    }

    let list = list_layout(area, app);

    if list.top > 0 {
        render_more(list.header, frame, theme, char_set);
    }

    if list.top + list.visible_rows < app.devices.len() {
        render_more(list.footer, frame, theme, char_set);
    }

    for ((index, device), row_area) in app
        .devices
        .iter()
        .enumerate()
        .skip(list.top)
        .take(list.visible_rows + 1)
        .zip(list.item_areas.iter())
    {
        render_device(
            *row_area,
            frame,
            device,
            index == list.selected,
            theme,
            char_set,
        );
    }
}

pub fn device_at(area: Rect, app: &App, column: u16, row: u16) -> Option<usize> {
    device_row_at(area, app, column, row).map(|(index, _)| index)
}

pub fn brightness_at(area: Rect, app: &App, column: u16, row: u16) -> Option<u16> {
    let (_, row_area) = device_row_at(area, app, column, row)?;
    let brightness_area = device_body_layout(row_area).brightness;
    bars::brightness_value_at(brightness_area, column, row)
}

fn render_more(area: Rect, frame: &mut Frame, theme: &Theme, char_set: &CharSet) {
    let line =
        Line::from(Span::styled(char_set.list_more, theme.list_more)).alignment(Alignment::Center);
    frame.render_widget(line, area);
}

fn render_device(
    area: Rect,
    frame: &mut Frame,
    device: &Device,
    selected: bool,
    theme: &Theme,
    char_set: &CharSet,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_selector(layout[0], frame, selected, theme, char_set);
    render_device_body(layout[1], frame, device, theme, char_set);
}

fn render_selector(
    area: Rect,
    frame: &mut Frame,
    selected: bool,
    theme: &Theme,
    char_set: &CharSet,
) {
    if !selected {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    frame.render_widget(Span::styled(char_set.selector_top, theme.selector), rows[0]);
    frame.render_widget(
        Span::styled(char_set.selector_middle, theme.selector),
        rows[1],
    );
    frame.render_widget(
        Span::styled(char_set.selector_bottom, theme.selector),
        rows[2],
    );
}

fn render_device_body(
    area: Rect,
    frame: &mut Frame,
    device: &Device,
    theme: &Theme,
    char_set: &CharSet,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Legacy)
        .split(area);

    render_device_header(layout[0], frame, device, theme);
    bars::render_brightness(layout[2], frame, device, theme, char_set);
}

fn render_device_header(area: Rect, frame: &mut Frame, device: &Device, theme: &Theme) {
    let backend = backend_label(&device.backend);
    let backend_width = backend.len().saturating_add(2) as u16;
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(backend_width)])
        .horizontal_margin(1)
        .spacing(1)
        .split(area);

    let title = Line::from(vec![
        Span::styled(&device.description, theme.title),
        Span::styled(format!("  {}", device.name), theme.detail),
    ]);
    let backend = Line::from(Span::styled(backend, theme.backend)).alignment(Alignment::Right);

    frame.render_widget(title, layout[0]);
    frame.render_widget(backend, layout[1]);
}

fn backend_label(backend: &BrightnessBackend) -> &'static str {
    match backend {
        BrightnessBackend::Laptop => "Laptop",
        BrightnessBackend::Ddc => "DDC",
        BrightnessBackend::Software => "Software",
    }
}

fn device_row_at(area: Rect, app: &App, column: u16, row: u16) -> Option<(usize, Rect)> {
    if app.devices.is_empty() {
        return None;
    }

    let devices_area = super::content_area(area);
    if !super::contains(devices_area, column, row) {
        return None;
    }

    let list = list_layout(devices_area, app);
    list.item_areas
        .iter()
        .enumerate()
        .find_map(|(offset, row_area)| {
            let index = list.top + offset;
            (index < app.devices.len() && super::contains(*row_area, column, row))
                .then_some((index, *row_area))
        })
}

struct DeviceBodyLayout {
    brightness: Rect,
}

fn device_body_layout(row_area: Rect) -> DeviceBodyLayout {
    let body_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(row_area)[1];
    let brightness = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Legacy)
        .split(body_area)[2];

    DeviceBodyLayout { brightness }
}

struct DeviceListLayout {
    top: usize,
    selected: usize,
    visible_rows: usize,
    header: Rect,
    footer: Rect,
    item_areas: Vec<Rect>,
}

fn list_layout(area: Rect, app: &App) -> DeviceListLayout {
    let selected = app.selected.min(app.devices.len().saturating_sub(1));
    let full_row_height = ROW_HEIGHT + ROW_SPACING;
    let visible_rows = (area.height / full_row_height).max(1) as usize;
    let top = selected
        .saturating_add(1)
        .saturating_sub(visible_rows)
        .min(app.devices.len().saturating_sub(visible_rows));

    let rows_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let mut constraints = vec![Constraint::Length(ROW_HEIGHT); visible_rows];
    constraints.push(Constraint::Max(ROW_HEIGHT));
    let item_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .spacing(ROW_SPACING)
        .split(rows_area[1])
        .to_vec();

    DeviceListLayout {
        top,
        selected,
        visible_rows,
        header: rows_area[0],
        footer: rows_area[2],
        item_areas,
    }
}
