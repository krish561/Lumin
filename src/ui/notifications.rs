use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Clear, Padding},
};

use super::theme::Theme;

const MAX_VISIBLE: usize = 3;
const POPUP_HEIGHT: u16 = 3;
const POPUP_SPACING: u16 = 1;

pub(crate) fn render_stack(area: Rect, frame: &mut Frame, messages: &[&str], theme: &Theme) {
    let visible: Vec<&str> = messages.iter().copied().take(MAX_VISIBLE).collect();

    for (i, message) in visible.iter().enumerate() {
        let y_offset = i as u16 * (POPUP_HEIGHT + POPUP_SPACING);
        if area.y + y_offset + POPUP_HEIGHT > area.bottom() {
            break;
        }

        let width = (message.len() as u16).saturating_add(4).min(area.width);
        let x = area.right().saturating_sub(width);
        let popup_area = Rect::new(x, area.y + y_offset, width, POPUP_HEIGHT);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title("Notice")
            .borders(Borders::ALL)
            .border_style(theme.notification_border)
            .padding(Padding::horizontal(1));
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let line = Line::from(*message).style(theme.notification_text);
        frame.render_widget(line, inner);
    }
}

