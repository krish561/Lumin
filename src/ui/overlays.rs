use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding},
};

use crate::app::{ActiveSection, App, BackendReason, BrightnessBackend};

use super::theme::Theme;

pub(crate) fn render_section_window(
    area: Rect,
    frame: &mut Frame,
    app: &App,
    section: ActiveSection,
    theme: &Theme,
) {
    let (width, height) = section_size(section);
    let width = area.width.min(width);
    let height = area.height.min(height);

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

    match section {
        ActiveSection::Displays => render_displays(inner, frame, app, theme),
        ActiveSection::Gamma => render_gamma(inner, frame, app, theme),
        ActiveSection::Night => render_night(inner, frame, app, theme),
        ActiveSection::Profiles => render_placeholder(inner, frame, section, theme),
    }
}

// ── Displays ──────────────────────────────────────────────────────────────────

fn render_displays(area: Rect, frame: &mut Frame, app: &App, theme: &Theme) {
    let Some(device) = app.devices.get(app.selected) else {
        frame.render_widget(
            Line::from(Span::styled("No display selected", theme.detail)),
            area,
        );
        return;
    };

    // Rows:
    //   0: description + name
    //   1: spacer
    //   2: backend
    //   3: reason
    //   4: brightness
    //   5: spacer
    //   6: current mode
    //   7: available modes hint
    //   8: spacer
    //   9: actions
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 0 title
            Constraint::Length(1), // 1 spacer
            Constraint::Length(1), // 2 backend
            Constraint::Length(1), // 3 reason
            Constraint::Length(1), // 4 brightness
            Constraint::Length(1), // 5 spacer
            Constraint::Length(1), // 6 current mode
            Constraint::Length(1), // 7 modes hint
            Constraint::Length(1), // 8 spacer
            Constraint::Length(1), // 9 actions
        ])
        .split(area);

    // Row 0: title
    frame.render_widget(
        Line::from(vec![
            Span::styled(device.description.clone(), theme.title),
            Span::styled(format!("  {}", device.name), theme.detail),
        ]),
        rows[0],
    );

    // Row 2-4: backend info + brightness
    render_kv(
        rows[2],
        frame,
        "Backend",
        &backend_label(&device.backend),
        theme,
    );
    render_kv(
        rows[3],
        frame,
        "Reason",
        &device.backend_reason.describe(),
        theme,
    );
    render_brightness_row(rows[4], frame, device.brightness, theme);

    // Row 6: current mode
    let current_mode = format!(
        "{}x{}@{:.2}Hz  scale {:.1}",
        device.width, device.height, device.refresh_rate, device.scale
    );
    render_kv(rows[6], frame, "Mode", &current_mode, theme);

    // Row 7: available modes count + hint
    let modes_hint = format!(
        "{} modes available  Tab/Shift+Tab to cycle",
        device.available_modes.len()
    );
    frame.render_widget(Line::from(Span::styled(modes_hint, theme.detail)), rows[7]);

    // Row 9: backend actions
    render_display_actions(rows[9], frame, app, theme);
}

// ── Gamma ─────────────────────────────────────────────────────────────────────

fn render_gamma(area: Rect, frame: &mut Frame, app: &App, theme: &Theme) {
    // Rows:
    //   0: title
    //   1: spacer
    //   2: temperature bar
    //   3: gamma bar
    //   4: spacer
    //   5: hint
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 0 title
            Constraint::Length(1), // 1 spacer
            Constraint::Length(1), // 2 temperature
            Constraint::Length(1), // 3 gamma
            Constraint::Length(1), // 4 spacer
            Constraint::Length(1), // 5 hint
        ])
        .split(area);

    frame.render_widget(
        Line::from(Span::styled("Color adjustments", theme.title)),
        rows[0],
    );

    // Temperature: 2500K–6500K, shown as a bar
    render_range_row(
        rows[2],
        frame,
        "Temperature",
        app.temperature,
        2500,
        6500,
        &format!("{}K", app.temperature),
        theme,
    );

    // Gamma: 10–200%, shown as a bar
    render_range_row(
        rows[3],
        frame,
        "Gamma",
        app.gamma,
        10,
        200,
        &format!("{}%", app.gamma),
        theme,
    );

    frame.render_widget(
        Line::from(vec![
            Span::styled("←/→ temperature  ", theme.detail),
            Span::styled("g/G gamma  ", theme.detail),
            Span::styled("r reset", theme.detail),
        ]),
        rows[5],
    );
}

// ── Night ─────────────────────────────────────────────────────────────────────

fn render_night(area: Rect, frame: &mut Frame, app: &App, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 0 title
            Constraint::Length(1), // 1 spacer
            Constraint::Length(1), // 2 status
            Constraint::Length(1), // 3 temperature
            Constraint::Length(1), // 4 spacer
            Constraint::Length(1), // 5 hint
        ])
        .split(area);

    frame.render_widget(
        Line::from(Span::styled("Night light", theme.title)),
        rows[0],
    );

    let is_night = app.temperature <= 4500;
    let status = if is_night { "On" } else { "Off" };
    let status_style = if is_night {
        theme.tab_selected
    } else {
        theme.detail
    };
    render_kv_styled(rows[2], frame, "Status", status, status_style, theme);

    let temp_label = format!("{}K  (warmer ←  cooler →)", app.temperature);
    render_kv(rows[3], frame, "Temperature", &temp_label, theme);

    frame.render_widget(
        Line::from(vec![
            Span::styled("n ", theme.tab_marker),
            Span::styled("toggle  ", theme.detail),
            Span::styled("←/→ ", theme.tab_marker),
            Span::styled("adjust temperature", theme.detail),
        ]),
        rows[5],
    );
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn render_kv(area: Rect, frame: &mut Frame, key: &str, value: &str, theme: &Theme) {
    render_kv_styled(area, frame, key, value, theme.title, theme);
}

fn render_kv_styled(
    area: Rect,
    frame: &mut Frame,
    key: &str,
    value: &str,
    value_style: ratatui::style::Style,
    theme: &Theme,
) {
    let key_width = 14u16;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(key_width), Constraint::Min(0)])
        .split(area);

    frame.render_widget(
        Line::from(Span::styled(key.to_string(), theme.detail)),
        cols[0],
    );
    frame.render_widget(
        Line::from(Span::styled(value.to_string(), value_style)),
        cols[1],
    );
}

fn render_brightness_row(area: Rect, frame: &mut Frame, brightness: u16, theme: &Theme) {
    let key_width = 14u16;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(key_width), Constraint::Min(0)])
        .split(area);

    frame.render_widget(
        Line::from(Span::styled("Brightness", theme.detail)),
        cols[0],
    );

    let percent = brightness.min(100);
    let label = format!("{percent}%");
    let label_width = label.len() as u16 + 1;

    let bar_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(label_width)])
        .split(cols[1]);

    let filled_width = ((percent as f32 / 100.0) * bar_cols[0].width as f32).round() as usize;
    let filled = "━".repeat(filled_width);
    let empty = "╌".repeat((bar_cols[0].width as usize).saturating_sub(filled_width));

    frame.render_widget(
        Line::from(vec![
            Span::styled(filled, theme.bar_filled),
            Span::styled(empty, theme.bar_empty),
        ]),
        bar_cols[0],
    );
    frame.render_widget(
        Line::from(Span::styled(label, theme.brightness)),
        bar_cols[1],
    );
}

fn render_range_row(
    area: Rect,
    frame: &mut Frame,
    key: &str,
    value: u32,
    min: u32,
    max: u32,
    label: &str,
    theme: &Theme,
) {
    let key_width = 14u16;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(key_width), Constraint::Min(0)])
        .split(area);

    frame.render_widget(
        Line::from(Span::styled(key.to_string(), theme.detail)),
        cols[0],
    );

    let label_width = label.len() as u16 + 1;
    let bar_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(label_width)])
        .split(cols[1]);

    let fraction = (value.saturating_sub(min)) as f32 / (max - min) as f32;
    let filled_width = (fraction * bar_cols[0].width as f32).round() as usize;
    let filled = "━".repeat(filled_width);
    let empty = "╌".repeat((bar_cols[0].width as usize).saturating_sub(filled_width));

    frame.render_widget(
        Line::from(vec![
            Span::styled(filled, theme.bar_filled),
            Span::styled(empty, theme.bar_empty),
        ]),
        bar_cols[0],
    );
    frame.render_widget(
        Line::from(Span::styled(label.to_string(), theme.brightness)),
        bar_cols[1],
    );
}

fn render_display_actions(area: Rect, frame: &mut Frame, app: &App, theme: &Theme) {
    let Some(device) = app.devices.get(app.selected) else {
        return;
    };

    match device.backend {
        BrightnessBackend::Laptop => {
            frame.render_widget(
                Line::from(Span::styled(
                    "Laptop display — no backend actions",
                    theme.detail,
                )),
                area,
            );
        }
        BrightnessBackend::Software => {
            let hint = match &device.backend_reason {
                BackendReason::DdcFailed { .. } => "r retry DDC  s force software",
                _ => "r retry DDC",
            };
            frame.render_widget(
                Line::from(vec![
                    Span::styled("[ Retry DDC ]", theme.tab_selected),
                    Span::styled(format!("  {hint}"), theme.detail),
                ]),
                area,
            );
        }
        BrightnessBackend::Ddc => {
            frame.render_widget(
                Line::from(vec![
                    Span::styled("[ Force Software ]", theme.tab_selected),
                    Span::styled("  s force software", theme.detail),
                ]),
                area,
            );
        }
    }
}

fn backend_label(backend: &BrightnessBackend) -> &'static str {
    match backend {
        BrightnessBackend::Laptop => "Laptop",
        BrightnessBackend::Ddc => "DDC",
        BrightnessBackend::Software => "Software",
    }
}

// ── Placeholder ───────────────────────────────────────────────────────────────

fn render_placeholder(area: Rect, frame: &mut Frame, section: ActiveSection, theme: &Theme) {
    let lines: Vec<Line> = match section {
        ActiveSection::Profiles => vec![
            Line::from(Span::styled("Brightness profiles", theme.title)),
            Line::from(""),
            Line::from(Span::styled(
                "Coming soon: indoor, outdoor, gaming, night, custom",
                theme.detail,
            )),
        ],
        _ => vec![Line::from(Span::styled("Coming soon", theme.detail))],
    };

    let constraints = lines
        .iter()
        .map(|_| Constraint::Length(1))
        .collect::<Vec<_>>();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (line, area) in lines.into_iter().zip(rows.iter()) {
        frame.render_widget(line, *area);
    }
}

fn section_size(section: ActiveSection) -> (u16, u16) {
    match section {
        ActiveSection::Displays => (64, 13),
        ActiveSection::Gamma => (60, 8),
        ActiveSection::Night => (60, 8),
        ActiveSection::Profiles => (54, 6),
    }
}
