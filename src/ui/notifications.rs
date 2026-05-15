use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Clear, Padding},
};

use super::theme::Theme;

pub(crate) fn render(area: Rect, frame: &mut Frame, message: &str, theme: &Theme) {
    let width = (message.len() as u16).saturating_add(4).min(area.width);
    let height = area.height.min(3);
    if width == 0 || height == 0 {
        return;
    }

    let x = area.right().saturating_sub(width);
    let popup_area = Rect::new(x, area.y, width, height);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Notice")
        .borders(Borders::ALL)
        .border_style(theme.notification_border)
        .padding(Padding::horizontal(1));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let line = Line::from(message).style(theme.notification_text);
    frame.render_widget(line, inner);
}
