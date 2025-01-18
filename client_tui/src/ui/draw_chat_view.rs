use crate::state::ActiveComponent::*;
use crate::state::TUIState;
use crate::ui::draw_alert::draw_alert;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType::Rounded;
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_chat_view(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let border_style = if let ChatView = state.ui_data.active_component {
        Style::new().green()
    } else {
        Style::new()
    };

    if let Some(l_id) = state.ui_data.current_log {
        if let Some(r_id) = state.ui_data.current_room {
            let curr_room = &state.chat_data.chat_rooms[r_id];
            let curr_log = &curr_room.chats[l_id];
            frame.render_widget(
                Block::bordered()
                    .border_type(Rounded)
                    .border_style(border_style)
                    .title(format!("{}", curr_log.peer_name))
                    .style(Style::new()),
                rect,
            );
        }
    } else {
        frame.render_widget(
            Block::bordered()
                .border_type(Rounded)
                .border_style(border_style)
                .title("Chat view")
                .style(Style::new()),
            rect,
        );
        draw_alert(frame, rect, "Select a friend first!")
    }
}
