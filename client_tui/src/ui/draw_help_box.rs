use crate::state::ActiveComponent::*;
use crate::state::TextEditAction::*;
use crate::state::{NameSetAction, TUIState};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::cell::Ref;

pub(super) fn draw_help_box(frame: &mut Frame, rect: Rect, state: &Ref<TUIState>) {
    let mut text = "".to_string();
    match &state.ui_data.active_component {
        NameSet(a) => {
            if let NameSetAction::ChangingName(_) = a {
                text.push_str("Write your name!\n");
                text.push_str("<Esc>    : Cancel action\n");
                text.push_str("<Enter>  : Confirm new name\n");
            } else {
                text.push_str("<C-Down> : Go to chat Rooms\n");
                if let Some(c) = state.ui_data.current_room {
                    text.push_str("<C-Right>: Go to chats\n");
                }
                text.push_str("<Enter>  : Edit name\n");
            }
        }
        RoomSelect => {
            text.push_str("<C-Up>   : Select name\n");
            if let Some(cr_id) = state.ui_data.selected_room {
                let c = &state.chat_data.chat_rooms[cr_id];
                if state.ui_data.current_room.is_some() {
                    text.push_str("<C-Right>: Go to chats\n");
                }
                text.push_str("<Up|Down>: Navigate rooms\n");
                if c.net_reachable {
                    if c.registered_to {
                        text.push_str("<Enter>  : Select room\n");
                    } else {
                        text.push_str("<R>      : Register to room\n");
                    }
                }
            }
        }
        ChatSelect => {
            text.push_str("<C-Left> : Go to rooms\n");
            if state.ui_data.current_log.is_some() {
                text.push_str("<C-Right>: Go to chat view\n");
            }
            if let Some(r_id) = state.ui_data.current_room {
                let room = &state.chat_data.chat_rooms[r_id];
                if !room.chats.is_empty() {
                    text.push_str("<Up|Down>: Navigate chats\n");
                    text.push_str("<Enter>  : Select chat\n");
                }
            }
        }
        ChatView => {
            text.push_str("<C-Down> : Create text message\n");
            text.push_str("<C-Left> : Go to chats\n");
            if let Some(r_id) = state.ui_data.current_room {
                let room = &state.chat_data.chat_rooms[r_id];
                if let Some(l_id) = state.ui_data.current_log {
                    let log = &room.chats[l_id];
                    if !log.messages.is_empty() {
                        text.push_str("<Up|Down>: Scroll chat\n");
                        //TODO: determine if selected message is mine or not
                    }
                }
            }
        }
        TextEdit(a) => {
            text.push_str("<C-Up>   : Go to chat view\n");
            if let Editing = a {
                text.push_str("<C-Left> : Go to chats\n");
                text.push_str("<C-Right>: Send button")
            } else {
                text.push_str("<C-Left  : Edit text message\n");
                text.push_str("<Enter>  : Send message\n");
            }
        }
        _ => {}
    }

    let b = Block::bordered()
        .border_type(Rounded)
        .border_style(Style::new().magenta())
        .title("Help");
    let p = Paragraph::new(text)
        .block(b)
        .alignment(Alignment::Left)
        .style(Style::new());

    frame.render_widget(p, rect);
}
