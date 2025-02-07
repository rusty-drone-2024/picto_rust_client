use crate::state::ActiveComponent::*;
use crate::state::NameSetAction::*;
use crate::state::{ChatMessage, TUIState};
use client_lib::communication::MessageContent::TextMessage;
use client_lib::communication::MessageStatus::SentToServer;
use client_lib::communication::TUIEvent::{DeleteMessage, RegisterToServer, SendMessage, SetName};
use client_lib::communication::{send_message, ChatClientID, ChatServerID};
use client_lib::ClientError;
use rand::Rng;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use std::cell::RefMut;
use std::net::TcpStream;

pub(super) fn handle_event(
    stream: &mut TcpStream,
    mut state: RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    match &state.ui_data.active_component {
        NameSet(action) => match action {
            Displaying => {
                handle_name_set_displaying_event(&mut state, event)?;
            }
            ChangingName => {
                handle_name_set_changing_event(stream, &mut state, event)?;
            }
        },
        RoomSelect => handle_room_select_event(stream, &mut state, event)?,
        ChatSelect => handle_chat_select_event(&mut state, event)?,
        ChatView => handle_chat_view_event(stream, &mut state, event)?,
        TextEdit => handle_text_area_event(stream, &mut state, event)?,
        _ => {}
    }
    Ok(())
}

fn handle_name_set_changing_event(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        match key.code {
            KeyCode::Esc => {
                state.ui_data.name_in_editing = None;
                state.ui_data.active_component = NameSet(Displaying);
            }
            KeyCode::Enter => {
                if let Some(name) = &state.ui_data.name_in_editing {
                    if !name.is_empty() {
                        send_message(stream, SetName(name.clone()))?;
                        state.ui_data.name_in_editing = None;
                        state.ui_data.active_component = NameSet(Displaying);
                    }
                }
            }
            KeyCode::Char(c) => {
                if let Some(name) = &mut state.ui_data.name_in_editing {
                    name.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(name) = &mut state.ui_data.name_in_editing {
                    if !name.is_empty() {
                        name.pop();
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn handle_name_set_displaying_event(
    state: &mut RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Down => {
                    go_to_room_select(state);
                }
                KeyCode::Right => {
                    go_to_chat_select(state);
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Enter => {
                    state.ui_data.name_in_editing = Some(state.chat_data.current_name.to_string());
                    state.ui_data.active_component = NameSet(ChangingName);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
fn handle_room_select_event(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Up => {
                    state.ui_data.active_component = NameSet(Displaying);
                    state.ui_data.selected_room = None;
                }
                KeyCode::Right => {
                    go_to_chat_select(state);
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    room_select_go_up(state);
                }
                KeyCode::Down => {
                    room_select_go_down(state);
                }
                KeyCode::Enter => {
                    if let Some(r_id) = state.ui_data.selected_room {
                        let room = &state.chat_data.chat_rooms[r_id];
                        if room.registered_to {
                            if let Some(curr_room_id) = &state.ui_data.current_room {
                                if r_id != *curr_room_id {
                                    state.ui_data.current_log = None;
                                    let mut selected_message =
                                        state.ui_data.selected_message.borrow_mut();
                                    *selected_message = None;
                                }
                            }
                            state.ui_data.current_room = Some(r_id);
                            go_to_chat_select(state);
                        }
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Some(r_id) = state.ui_data.selected_room {
                        let room = &state.chat_data.chat_rooms[r_id];
                        if !room.registered_to && room.net_reachable {
                            send_message(stream, RegisterToServer(room.id))?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_chat_select_event(state: &mut RefMut<TUIState>, event: Event) -> Result<(), ClientError> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Left => {
                    go_to_room_select(state);
                }
                KeyCode::Right => {
                    go_to_chat_view(state);
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    chat_select_go_up(state);
                }
                KeyCode::Down => {
                    chat_select_go_down(state);
                }
                KeyCode::Enter => {
                    if let Some(l_id) = state.ui_data.selected_log {
                        let mut go_to_chat_bottom = state.ui_data.go_to_chat_bottom.borrow_mut();
                        if let Some(c_id) = state.ui_data.current_log {
                            if l_id != c_id {
                                *go_to_chat_bottom = true;
                                drop(go_to_chat_bottom);
                                state.ui_data.current_log = Some(l_id);
                                select_last_message(state)?;
                            } else {
                                drop(go_to_chat_bottom);
                            }
                        } else {
                            *go_to_chat_bottom = true;
                            drop(go_to_chat_bottom);
                            state.ui_data.current_log = Some(l_id);
                            select_last_message(state)?;
                        }

                        //state.ui_data.current_log = Some(l_id);
                        go_to_chat_view(state);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_chat_view_event(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Left => {
                    go_to_chat_select(state);
                }
                KeyCode::Down => {
                    go_to_text_area(state);
                }
                _ => {}
            }
        } else if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Up => {
                    message_select_go_up(state);
                }

                KeyCode::Down => {
                    message_select_go_down(state);
                }
                KeyCode::Char('d') => {
                    delete_selected_message(stream, state)?;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    state.ui_data.scroll_view_state.borrow_mut().scroll_up();
                }
                KeyCode::Down => {
                    state.ui_data.scroll_view_state.borrow_mut().scroll_down();
                }
                KeyCode::Char('d') => {
                    delete_selected_message(stream, state)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_text_area_event(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
    event: Event,
) -> Result<(), ClientError> {
    let text = &mut state.ui_data.text_message_in_edit;
    if let Event::Key(key) = event {
        if key.kind == event::KeyEventKind::Release {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Left => {
                    go_to_chat_select(state);
                }
                KeyCode::Up => {
                    go_to_chat_view(state);
                }
                KeyCode::Char('s') => {
                    send_current_text_message(stream, state)?;
                    select_last_message(state)?;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char(c) => {
                    text.push(c);
                }
                KeyCode::Backspace => {
                    if !text.is_empty() {
                        text.pop();
                    }
                }
                KeyCode::Enter => {
                    if !text.is_empty() {
                        let lines: Vec<&str> = text.split("\n").collect();
                        let last_line = lines.last().unwrap();
                        if !last_line.is_empty() {
                            text.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn go_to_text_area(state: &mut RefMut<TUIState>) {
    state.ui_data.active_component = TextEdit;
}

fn go_to_room_select(state: &mut RefMut<TUIState>) {
    state.ui_data.active_component = RoomSelect;
    state.ui_data.selected_log = None;
    if let Some(r_id) = state.ui_data.current_room {
        state.ui_data.selected_room = Some(r_id);
    } else {
        if !state.chat_data.chat_rooms.is_empty() {
            state.ui_data.selected_room = Some(0);
        } else {
            state.ui_data.selected_room = None;
        }
    }
}
fn go_to_chat_select(state: &mut RefMut<TUIState>) {
    if let Some(chat_room_id) = state.ui_data.current_room {
        let room = &state.chat_data.chat_rooms[chat_room_id];
        if let Some(l_id) = state.ui_data.current_log {
            state.ui_data.selected_log = Some(l_id);
        } else if !room.chats.is_empty() {
            state.ui_data.selected_log = Some(0);
        }
        state.ui_data.active_component = ChatSelect;
    }
}

fn go_to_chat_view(state: &mut RefMut<TUIState>) {
    if let Some(r_id) = state.ui_data.current_room {
        if let Some(l_id) = state.ui_data.current_log {
            state.ui_data.active_component = ChatView;
            state.ui_data.selected_log = None;
            read_log(state, r_id, l_id);
        }
    }
}

fn room_select_go_up(state: &mut RefMut<TUIState>) {
    if let Some(id) = state.ui_data.selected_room {
        if id > 0 {
            state.ui_data.selected_room = Some(id - 1);
        }
    }
}

fn room_select_go_down(state: &mut RefMut<TUIState>) {
    if let Some(id) = state.ui_data.selected_room {
        if id < state.chat_data.chat_rooms.len() - 1 {
            state.ui_data.selected_room = Some(id + 1);
        }
    }
}

fn chat_select_go_up(state: &mut RefMut<TUIState>) {
    if let Some(id) = state.ui_data.selected_log {
        if id > 0 {
            state.ui_data.selected_log = Some(id - 1);
        }
    }
}

fn chat_select_go_down(state: &mut RefMut<TUIState>) {
    if let Some(r_id) = state.ui_data.current_room {
        let room = &state.chat_data.chat_rooms[r_id];
        if let Some(id) = state.ui_data.selected_log {
            if id < room.chats.len() - 1 {
                state.ui_data.selected_log = Some(id + 1);
            }
        }
    }
}

fn message_select_go_up(state: &mut RefMut<TUIState>) {
    let mut selected_message = state.ui_data.selected_message.borrow_mut();
    if let Some(id) = *selected_message {
        if id > 0 {
            *selected_message = Some(id - 1);
        }
    }
}

fn message_select_go_down(state: &mut RefMut<TUIState>) {
    if let Some(r_id) = state.ui_data.current_room {
        let room = &state.chat_data.chat_rooms[r_id];
        if let Some(l_id) = state.ui_data.current_log {
            let log = &room.chats[l_id];
            let mut selected_message = state.ui_data.selected_message.borrow_mut();
            if let Some(id) = *selected_message {
                if id < log.messages.len() - 1 {
                    *selected_message = Some(id + 1);
                }
            }
        }
    }
}

fn read_log(state: &mut RefMut<TUIState>, room_id: usize, log_id: usize) {
    let room = &mut state.chat_data.chat_rooms[room_id];
    let log = &mut room.chats[log_id];
    room.pending -= log.pending;
    log.pending = 0;
}

fn send_current_text_message(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
) -> Result<(), ClientError> {
    if let Some(room_pos) = state.ui_data.current_room {
        if let Some(log_pos) = state.ui_data.current_log {
            let room = &state.chat_data.chat_rooms[room_pos];
            let log = &room.chats[log_pos];
            let room_id = room.id;
            let log_id = log.id;
            let mut rng = rand::rng();
            let msg_id = rng.random();
            let mut msg = state.ui_data.text_message_in_edit.clone();
            let mut cleaned = false;
            while !cleaned {
                if let Some(c) = msg.chars().last() {
                    if c == '\n' || c == ' ' {
                        msg.pop();
                    } else {
                        cleaned = true;
                    }
                } else {
                    cleaned = true;
                }
            }
            if !msg.is_empty() {
                let content = TextMessage(msg);
                send_message(
                    stream,
                    SendMessage(
                        room_id as ChatServerID,
                        log_id as ChatClientID,
                        msg_id,
                        content.clone(),
                    ),
                )?;
                state.chat_data.chat_rooms[room_pos].chats[log_pos]
                    .messages
                    .push(ChatMessage {
                        id: msg_id,
                        content: Some(content),
                        //TODO set to now
                        timestamp: 0,
                        status: Some(SentToServer),
                        reaction: None,
                        edited: false,
                        deleted: false,
                    });
                state.ui_data.text_message_in_edit = "".to_string();
                let mut go_to_chat_bottom = state.ui_data.go_to_chat_bottom.borrow_mut();
                *go_to_chat_bottom = true;
            }
        }
    }
    Ok(())
}

fn select_last_message(state: &mut RefMut<TUIState>) -> Result<(), ClientError> {
    if let Some(r_id) = state.ui_data.current_room {
        let room = &state.chat_data.chat_rooms[r_id];
        if let Some(l_id) = state.ui_data.current_log {
            let log = &room.chats[l_id];
            let mut selected_message = state.ui_data.selected_message.borrow_mut();
            if !log.messages.is_empty() {
                *selected_message = Some(log.messages.len() - 1);
            } else {
                *selected_message = None;
            }
        }
    }
    Ok(())
}

fn delete_selected_message(
    stream: &mut TcpStream,
    state: &mut RefMut<TUIState>,
) -> Result<(), ClientError> {
    if let Some(r_id) = state.ui_data.current_room {
        if let Some(l_id) = state.ui_data.current_log {
            let selected_message = state.ui_data.selected_message.borrow_mut();
            if let Some(m_id) = *selected_message {
                drop(selected_message);
                let room = &mut state.chat_data.chat_rooms[r_id];
                let log = &mut room.chats[l_id];
                let msg = &mut log.messages[m_id];
                if msg.status.is_some() {
                    send_message(stream, DeleteMessage(room.id, log.id, msg.id))?;
                    msg.content = None;
                }
            }
        }
    }
    Ok(())
}
