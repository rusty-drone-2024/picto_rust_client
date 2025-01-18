use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

pub(super) fn draw_alert(frame: &mut Frame, rect: Rect, text: &str) {
    let p = Paragraph::new(text)
        .block(
            Block::bordered()
                .border_type(Rounded)
                .title("Alert")
                .border_style(Style::new().red()),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    let h_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 80) / 2),
            Constraint::Percentage(80),
            Constraint::Percentage((100 - 80) / 2),
        ])
        .split(rect);

    let mut height = p.line_count(h_split[1].width - 2) as u16;
    if height > rect.height {
        height = rect.height
    }

    let v_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(h_split[1]);

    frame.render_widget(p, v_split[1]);
}
