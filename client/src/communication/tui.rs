use crate::network::Network;
use client_lib::communication::TUICommand::{UpdateChatRoom, UpdateName};
use client_lib::communication::TUIEvent::{DeleteMessage, RegisterToServer, SetName};
use client_lib::communication::{receive_message, send_message, TUIEvent};
use client_lib::ClientError;
use client_lib::ClientError::LockError;
use std::cell::RefCell;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(crate) fn tui_event_receiver(state: Arc<Mutex<Network>>, mut stream: TcpStream) {
    loop {
        match receive_message::<TUIEvent>(&mut stream) {
            Ok(event) => {
                println!("Backend: received event: {:?}", event);
                match handle_tui_event(&state, &mut stream, event) {
                    Ok(_) => {
                        println!("Event handled correctly")
                    }
                    Err(_) => {
                        println!("Error in event handling")
                    }
                };
            }
            Err(e) => {
                println!("Backend: Error reading event: {:?}", e);
            }
        }
    }
}

fn handle_tui_event(
    state: &Arc<Mutex<Network>>,
    stream: &mut TcpStream,
    event: TUIEvent,
) -> Result<(), ClientError> {
    let state = state.lock().map_err(|_| LockError)?;
    match event {
        SetName(s) => {
            //TODO: send new name to server;
            //TODO: wait for ack;
            send_message(stream, UpdateName(s))?;
        }
        RegisterToServer(cr) => {
            //TODO: send register request to server;
            //TODO: wait for positive ack;
            send_message(stream, UpdateChatRoom(cr, Some(true), None))?;
        }
        DeleteMessage(cr, cl, cm) => {
            //TODO: send delete request to server;
            //TODO: wait for positive ack;
        }
        _ => {}
    }
    Ok(())
}
