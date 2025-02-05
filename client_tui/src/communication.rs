use crate::state::ActiveComponent::*;
use crate::state::{ChatLog, ChatMessage, ChatRoom, TUIState};
use client_lib::communication::MessageContent::TextMessage;
use client_lib::communication::TUICommand::*;
use client_lib::communication::TUIEvent::ReadMessage;
use client_lib::communication::{
    receive_message, send_message, ChatClientID, ChatServerID, MessageContent, MessageID,
    MessageStatus, Reaction, TUICommand,
};
use client_lib::ClientError;
use client_lib::ClientError::{LockError, TUICommandHandlingError};
use std::cell::{RefCell, RefMut};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(crate) fn backend_command_receiver(
    state: Arc<Mutex<RefCell<TUIState>>>,
    mut stream: TcpStream,
) {
    loop {
        match receive_message::<TUICommand>(&mut stream) {
            Ok(command) => {
                //println!("Received command: {:?}", command);
                match handle_backend_command(&state, &mut stream, command) {
                    Ok(_) => {
                        /*println!(
                            "Command handled correctly: {:#?}\n\n",
                            state.lock().unwrap().chat_data
                        );*/
                    }
                    Err(_) => {
                        //println!("Error in command handling\n\n")
                    }
                };
            }
            Err(e) => {
                //println!("Error reading command: {:#?}\n\n", e);
            }
        }
    }
}

fn handle_backend_command(
    state: &Arc<Mutex<RefCell<TUIState>>>,
    stream: &mut TcpStream,
    command: TUICommand,
) -> Result<(), ClientError> {
    let state = state.lock().map_err(|_| LockError)?;
    let mut state = state.borrow_mut();
    match command {
        UpdateName(s) => {
            state.chat_data.current_name = s.clone();
            state.ui_data.change_window_title = true;
            state.ui_data.new_window_title = format!("{}'s chat client", s);
        }
        UpdateChatRoom(id, registered, reachable) => {
            handle_chat_room_update(state, id, registered, reachable)?;
        }
        UpdatePeerName(room_id, log_id, name) => {
            handle_peer_name_update(state, room_id, log_id, name)?;
        }
        UpdatePeerLastSeen(room_id, log_id) => {
            handle_peer_last_seen_update(state, room_id, log_id)?;
        }
        UpdatePeerStatus(room_id, log_id, status) => {
            handle_peer_status_update(state, room_id, log_id, status)?;
        }
        UpdateMessageContent(room_id, log_id, msg_id, content) => {
            handle_message_content_update(state, stream, room_id, log_id, msg_id, content)?;
        }
        UpdateMessageStatus(room_id, log_id, msg_id, reachable) => {
            handle_message_status_update(state, room_id, log_id, msg_id, reachable)?;
        }
        UpdateMessageReaction(room_id, log_id, msg_id, reaction) => {
            handle_message_reaction_update(state, room_id, log_id, msg_id, reaction)?;
        }
        DeleteMessage(room_id, log_id, msg_id) => {
            handle_message_delete(state, room_id, log_id, msg_id)?;
        }
        Kill => {
            state.kill = true;
        }
    }
    Ok(())
}

fn handle_chat_room_update(
    mut state: RefMut<TUIState>,
    id: ChatServerID,
    registered: Option<bool>,
    reachable: Option<bool>,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&id));
    match room_pos_result {
        Ok(room_pos) => {
            let old_room = &mut state.chat_data.chat_rooms[room_pos];
            if let Some(b) = registered {
                old_room.registered_to = b;
            }
            if let Some(b) = reachable {
                old_room.net_reachable = b;
            }
        }
        Err(_) => {
            let registered = registered.unwrap_or_default();
            let reachable = reachable.unwrap_or_default();
            let new_room = ChatRoom {
                id,
                chats: Vec::new(),
                pending: 0,
                registered_to: registered,
                net_reachable: reachable,
            };
            state.chat_data.chat_rooms.push(new_room);
            if let RoomSelect = state.ui_data.active_component {
                if state.chat_data.chat_rooms.len() == 1 {
                    state.ui_data.selected_room = Some(0);
                }
            }
        }
    }
    Ok(())
}
fn handle_peer_name_update(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
    name: String,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        match log_pos_result {
            Ok(log_pos) => {
                let log = &mut room.chats[log_pos];
                log.peer_name = name;
            }
            Err(_) => {
                let log = ChatLog {
                    id: log_id,
                    messages: Vec::new(),
                    peer_name: name,

                    //TODO: set last seen to now
                    last_seen: 0,
                    currently_creating: TextMessage("".to_string()),
                    pending: 0,
                    net_reachable: true,
                };
                room.chats.push(log);
            }
        }
        return Ok(());
    }

    Err(TUICommandHandlingError)
}
fn handle_peer_last_seen_update(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            //TODO: set last seen to now
            log.last_seen += 100;
            return Ok(());
        }
    }

    Err(TUICommandHandlingError)
}
fn handle_peer_status_update(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
    reachable: bool,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            log.net_reachable = reachable;
            return Ok(());
        }
    }

    Err(TUICommandHandlingError)
}
fn handle_message_content_update(
    mut state: RefMut<TUIState>,
    stream: &mut TcpStream,
    room_id: ChatServerID,
    log_id: ChatClientID,
    msg_id: MessageID,
    content: MessageContent,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            let msg_pos_result = log.messages.binary_search_by(|msg| msg.id.cmp(&msg_id));
            match msg_pos_result {
                Ok(msg_pos) => {
                    let msg = &mut log.messages[msg_pos];
                    msg.content = Some(content);
                    msg.edited = true;
                }
                Err(_) => {
                    log.messages.push(ChatMessage {
                        id: msg_id,
                        content: Some(content),
                        //TODO set ts to now
                        timestamp: 0,
                        status: None,
                        reaction: None,
                        edited: false,
                        deleted: false,
                    });
                    room.pending += 1;
                    log.pending += 1;
                }
            }
            match state.ui_data.current_room {
                Some(r) if state.chat_data.chat_rooms[r].id == room_id => {
                    match state.ui_data.current_log {
                        Some(l) if state.chat_data.chat_rooms[r].chats[l].id == log_id => {
                            send_message(stream, ReadMessage(room_id, log_id, msg_id))?;
                        }
                        _ => {}
                    };
                }
                _ => {}
            };
            return Ok(());
        }
    }

    Err(TUICommandHandlingError)
}
fn handle_message_status_update(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
    msg_id: MessageID,
    status: MessageStatus,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            let msg_pos_result = log.messages.binary_search_by(|msg| msg.id.cmp(&msg_id));
            if let Ok(msg_pos) = msg_pos_result {
                let msg = &mut log.messages[msg_pos];
                msg.status = Some(status);
                return Ok(());
            }
        }
    }

    Err(TUICommandHandlingError)
}
fn handle_message_reaction_update(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
    msg_id: MessageID,
    reaction: Option<Reaction>,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            let msg_pos_result = log.messages.binary_search_by(|msg| msg.id.cmp(&msg_id));
            if let Ok(msg_pos) = msg_pos_result {
                let msg = &mut log.messages[msg_pos];
                msg.reaction = reaction;
                return Ok(());
            }
        }
    }

    Err(TUICommandHandlingError)
}
fn handle_message_delete(
    mut state: RefMut<TUIState>,
    room_id: ChatServerID,
    log_id: ChatClientID,
    msg_id: MessageID,
) -> Result<(), ClientError> {
    let room_pos_result = state
        .chat_data
        .chat_rooms
        .binary_search_by(|cr| cr.id.cmp(&room_id));

    if let Ok(room_pos) = room_pos_result {
        let room = &mut state.chat_data.chat_rooms[room_pos];
        let log_pos_result = room.chats.binary_search_by(|cl| cl.id.cmp(&log_id));
        if let Ok(log_pos) = log_pos_result {
            let log = &mut room.chats[log_pos];
            let msg_pos_result = log.messages.binary_search_by(|msg| msg.id.cmp(&msg_id));
            if let Ok(msg_pos) = msg_pos_result {
                let msg = &mut log.messages[msg_pos];
                msg.deleted = true;
                msg.content = None;
                msg.reaction = None;
                msg.status = None;
                return Ok(());
            }
        }
    }

    Err(TUICommandHandlingError)
}
