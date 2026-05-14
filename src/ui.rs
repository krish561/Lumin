use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    let title = Block::default().title("Lumin").borders(Borders::ALL);

    let items: Vec<ListItem> = app
        .devices
        .iter()
        .enumerate()
        .map(|(index, device)| {
            let prefix = if index == app.selected { "> " } else { "  " };

            ListItem::new(format!("{}{}", prefix, device.name))
        })
        .collect();

    let device_list =
        List::new(items).block(Block::default().title("Devices").borders(Borders::ALL));

    let selected_device = &app.devices[app.selected];

    let gauge = Gauge::default()
        .block(Block::default().title("Brightness").borders(Borders::ALL))
        .percent(selected_device.brightness)
        .label(format!("{}%", selected_device.brightness));

    let footer = Block::default()
        .title("↑/↓ Select  ←/→ Adjust  •  q Quit")
        .borders(Borders::ALL);

    frame.render_widget(title, chunks[0]);
    frame.render_widget(device_list, content[0]);
    frame.render_widget(gauge, content[1]);
    frame.render_widget(footer, chunks[2]);
}
