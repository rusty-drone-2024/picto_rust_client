use crate::state::Client;
use client_lib::communication::TUICommand::{UpdateChatRoom, UpdateName};
use client_lib::communication::TUIEvent::DeleteMessage;
use client_lib::communication::{receive_message, send_message, TUIEvent};
use client_lib::ClientError;
use client_lib::ClientError::LockError;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(crate) fn tui_event_receiver(state: Arc<Mutex<Client>>, mut stream: TcpStream) {
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
    state: &Arc<Mutex<Client>>,
    stream: &mut TcpStream,
    event: TUIEvent,
) -> Result<(), ClientError> {
    let mut state = state.lock().map_err(|_| LockError)?;
    match event {
        TUIEvent::SetName(s) => {
            //TODO: send new name to server;
            //TODO: wait for ack;
            send_message(stream, UpdateName(s))?;
        }
        TUIEvent::RegisterToServer(cr) => {
            //TODO: send register request to server;
            //TODO: wait for positive ack;
            send_message(stream, UpdateChatRoom(cr, Some(true), None))?;
        }
        TUIEvent::DeleteMessage(cr, cl, cm) => {
            //TODO: send delete request to server;
            //TODO: wait for positive ack;
        }
        _ => {}
    }
    Ok(())
}
