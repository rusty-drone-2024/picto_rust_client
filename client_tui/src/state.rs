use crate::state::ActiveComponent::{NameSet, RoomSelect, Startup};
use client_lib::communication::{
    ChatClientID, ChatServerID, MessageContent, MessageID, MessageStatus, Reaction, TimeStamp,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use tui_scrollview::ScrollViewState;

#[derive(Debug)]
pub(crate) struct EditableContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: ChatServerID,
    pub chats: Vec<ChatLog>,
    pub pending: u32,
    pub registered_to: bool,
    pub net_reachable: bool,
}

impl Eq for ChatRoom {}

impl PartialEq<Self> for ChatRoom {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd<Self> for ChatRoom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Equal)
        } else if self.net_reachable && !other.net_reachable {
            Some(Less)
        } else if !self.net_reachable && other.net_reachable {
            Some(Greater)
        } else if self.registered_to && !other.registered_to {
            Some(Less)
        } else {
            Some(Greater)
        }
    }
}

impl Ord for ChatRoom {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(o) = self.partial_cmp(other) {
            return o;
        }
        Equal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatLog {
    pub id: ChatClientID,
    pub messages: Vec<ChatMessage>,
    pub peer_name: String,
    pub last_seen: TimeStamp,
    pub currently_creating: MessageContent,
    pub pending: u32,
    pub net_reachable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: MessageID,
    pub content: Option<MessageContent>,
    pub timestamp: TimeStamp,
    pub status: Option<MessageStatus>,
    pub reaction: Option<Reaction>,
    pub edited: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum ActiveComponent {
    Startup,
    RoomSelect,
    ChatSelect,
    ChatView,
    TextEdit,
    ReactionSend,
    NameSet(NameSetAction),
}
#[derive(Debug, Clone)]
pub(crate) enum NameSetAction {
    Displaying,
    ChangingName(String),
}

#[derive(Debug, Clone)]
pub(crate) struct TUIState<'a> {
    pub chat_data: ChatData,
    pub ui_data: UIData<'a>,
    pub kill: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ChatData {
    pub chat_rooms: Vec<ChatRoom>,
    pub current_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UIData<'a> {
    pub active_component: ActiveComponent,
    pub current_room: Option<usize>,
    pub selected_room: Option<usize>,
    pub current_log: Option<usize>,
    pub selected_log: Option<usize>,
    pub text_message_in_edit: String,
    pub reacting_to: Option<usize>,
    pub selected_reaction: Option<&'a Reaction>,
    pub name_in_editing: Option<String>,
    pub scroll_view_state: RefCell<ScrollViewState>,
    pub go_to_chat_bottom: RefCell<bool>,
    pub selected_message: RefCell<Option<usize>>,
}

impl TUIState<'_> {
    pub(crate) fn new() -> Self {
        TUIState {
            chat_data: ChatData {
                chat_rooms: vec![],
                current_name: "".to_string(),
            },
            ui_data: UIData {
                active_component: RoomSelect,
                current_room: None,
                selected_room: None,
                current_log: None,
                selected_log: None,
                text_message_in_edit: "".to_string(),
                reacting_to: None,
                selected_reaction: None,
                name_in_editing: None,
                scroll_view_state: RefCell::new(ScrollViewState::default()),
                go_to_chat_bottom: RefCell::new(false),
                selected_message: RefCell::new(None),
            },
            kill: false,
        }
    }
}
