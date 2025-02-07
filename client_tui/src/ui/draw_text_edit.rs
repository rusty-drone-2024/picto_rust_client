use crate::state::ActiveComponent::TextEdit;
use crate::state::TUIState;
use crate::ui::draw_alert::draw_alert;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_text_edit(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = if let TextEdit = state.ui_data.active_component {
        Style::new().green()
    } else {
        Style::new()
    };

    if state.ui_data.current_log.is_some() {
        let text = state.ui_data.text_message_in_edit.clone();
        let text_clone = text.clone();

        let mut editor = Paragraph::new(text).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(Rounded)
                .border_style(border_style)
                .title("Write something!"),
        );

        let line_count = if !text_clone.is_empty() {
            editor.line_count(rect.width - 2) - 3
        } else {
            0
        };
        let overflow = if line_count > 4 { line_count - 4 } else { 0 };
        editor = editor.scroll((overflow as u16, 0));

        frame.render_widget(editor, rect);
    } else {
        frame.render_widget(
            Block::bordered()
                .border_type(Rounded)
                .border_style(border_style)
                .style(Style::new()),
            rect,
        );
        draw_alert(frame, rect, "Select a friend first!");
    }
}
