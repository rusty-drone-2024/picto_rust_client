use crate::state::ActiveComponent::*;
use crate::state::NameSetAction::*;
use crate::state::TUIState;
use client_lib::communication::send_message;
use client_lib::communication::TUIEvent::RegisterToServer;
use client_lib::ClientError;
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
                handle_name_set_displaying_event(stream, &mut state, event)?;
            }
            ChangingName(s) => {}
        },
        RoomSelect => handle_room_select_event(stream, &mut state, event)?,
        ChatSelect => handle_chat_select_event(stream, &mut state, event)?,
        ChatView => handle_chat_view_event(stream, &mut state, event)?,
        _ => {}
    }
    Ok(())
}

fn handle_name_set_displaying_event(
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
                    //TODO: set active component to NameSet(ChangingName()) and handle them events
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

fn handle_chat_select_event(
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
                        state.ui_data.current_log = Some(l_id);
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
                _ => {}
            }
        } else {
            match key.code {
                _ => {}
            }
        }
    }
    Ok(())
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
    if let Some(id) = state.ui_data.selected_log {
        if let Some(r_id) = state.ui_data.current_room {
            let room = &state.chat_data.chat_rooms[r_id];
            if id < room.chats.len() - 1 {
                state.ui_data.selected_log = Some(id + 1);
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
