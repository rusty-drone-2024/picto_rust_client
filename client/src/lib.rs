mod communication;
mod helpers;
mod state;

use crate::communication::tui::tui_event_receiver;
use crate::helpers::{get_stream, new_listener, start_tui};
use crate::state::Client;
use client_lib::communication::send_message;
use client_lib::communication::MessageContent::TextMessage;
use client_lib::communication::Reaction::Skull;
use client_lib::communication::TUICommand::{
    UpdateChatRoom, UpdateMessageContent, UpdateMessageReaction, UpdateName, UpdatePeerName,
};
use client_lib::communication::TUIEvent::{DeleteMessage, Kill};
use client_lib::ClientError;
use client_lib::ClientError::StreamError;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::{thread, time};

pub fn run() -> Result<(), ClientError> {
    //START CLIENT TUI AND GET TCP CONNECTION TO IT
    let listener = new_listener()?;
    start_tui(&listener)?;
    let mut stream = get_stream(listener)?;
    //debug
    println!("TUI port: {}", stream.peer_addr().unwrap().port());

    //INITIALIZE STATE
    let state = Arc::new(Mutex::new(Client::new(1)));
    let mut state_clone = Arc::clone(&state);
    let mut tui_events_stream = stream.try_clone().map_err(|_| StreamError)?;

    //TUI EVENT RECEIVER THREAD
    thread::spawn(move || {
        tui_event_receiver(state_clone, tui_events_stream);
    });

    send_message(&mut stream, UpdateName("brokenhouse".to_string()))?;
    for i in 1..31 {
        send_message(
            &mut stream,
            UpdateChatRoom(i, Some(i % 3 != 0), Some(i % 4 != 0)),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(1, 1, "Gabry".to_string()))?;
    for i in 0..5 {
        send_message(
            &mut stream,
            UpdateMessageContent(1, 1, i, TextMessage("Test".to_string())),
        )?;
    }

    send_message(&mut stream, UpdatePeerName(1, 2, "Andreea".to_string()))?;
    for i in 0..8 {
        send_message(
            &mut stream,
            UpdateMessageContent(1, 2, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(1, 3, "Ace".to_string()))?;
    for i in 0..15 {
        send_message(
            &mut stream,
            UpdateMessageContent(1, 3, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(1, 4, "Marco".to_string()))?;
    for i in 0..2 {
        send_message(
            &mut stream,
            UpdateMessageContent(1, 4, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(2, 1, "Marty".to_string()))?;
    for i in 0..3 {
        send_message(
            &mut stream,
            UpdateMessageContent(2, 1, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(4, 1, "Roger".to_string()))?;
    for i in 0..12 {
        send_message(
            &mut stream,
            UpdateMessageContent(4, 1, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(4, 2, "Laura".to_string()))?;
    for i in 0..7 {
        send_message(
            &mut stream,
            UpdateMessageContent(4, 2, i, TextMessage("Test".to_string())),
        )?;
    }
    send_message(&mut stream, UpdatePeerName(8, 1, "Bianca".to_string()))?;
    for i in 0..20 {
        send_message(
            &mut stream,
            UpdateMessageContent(8, 1, i, TextMessage("Test".to_string())),
        )?;
    }

    send_message(&mut stream, UpdateMessageReaction(1, 1, 1, Some(Skull)))?;
    send_message(&mut stream, DeleteMessage(1, 1, 1))?;
    send_message(&mut stream, Kill)?;
    println!("sent all");
    loop {}

    Ok(())
}
