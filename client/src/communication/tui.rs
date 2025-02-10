use crate::network::Network;
use client_lib::communication::MessageStatus::ReadByPeer;
use client_lib::communication::TUICommand::{
    UpdateMessageContent, UpdateMessageReaction, UpdateMessageStatus, UpdateName,
};
use client_lib::communication::TUIEvent::*;
use client_lib::communication::{receive_message, send_message, TUICommand, TUIEvent};
use client_lib::ClientError;
use client_lib::ClientError::LockError;
use common_structs::message::Message::{ReqChatClients, ReqChatRegistration, ReqChatSend};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub(crate) fn tui_event_receiver(state: Arc<Mutex<Network>>, mut stream: TcpStream) {
    loop {
        match receive_message::<TUIEvent>(&mut stream) {
            Ok(event) => {
                //println!("Backend: received event: {:?}", event);
                match handle_tui_event(&state, &mut stream, event) {
                    Ok(_) => {
                        //println!("Event handled correctly")
                    }
                    Err(_) => {
                        //println!("Error in event handling")
                    }
                };
            }
            Err(e) => {
                //println!("Backend: Error reading event: {:?}", e);
                break;
            }
        }
    }
}

fn handle_tui_event(
    state: &Arc<Mutex<Network>>,
    stream: &mut TcpStream,
    event: TUIEvent,
) -> Result<(), ClientError> {
    let mut state = state.lock().map_err(|_| LockError)?;
    match event {
        SetName(s) => {
            //TODO: send new name to each known peer;
            send_message(stream, UpdateName(s))?;
        }
        RegisterToServer(cr) => {
            state.send_message(ReqChatRegistration, cr, None);
        }
        DeleteMessage(cr, cl, cm) => {
            let command = TUICommand::DeleteMessage(cr, state.id, cm);
            let content = serde_json::to_string(&command)
                .unwrap()
                .as_bytes()
                .to_owned();
            let message = ReqChatSend {
                to: cl,
                chat_msg: content,
            };
            state.send_message(message, cr, Some(cm));
        }
        SendMessage(cr, cl, cm, mc) => {
            let command = UpdateMessageContent(cr, state.id, cm, mc);
            let content = serde_json::to_string(&command)
                .unwrap()
                .as_bytes()
                .to_owned();
            let message = ReqChatSend {
                to: cl,
                chat_msg: content,
            };
            state.send_message(message, cr, Some(cm));
        }
        ReadMessage(cr, cl, cm) => {
            let command = UpdateMessageStatus(cr, state.id, cm, ReadByPeer);
            let content = serde_json::to_string(&command)
                .unwrap()
                .as_bytes()
                .to_owned();
            let message = ReqChatSend {
                to: cl,
                chat_msg: content,
            };
            state.send_message(message, cr, Some(cm));
        }
        ReactToMessage(cr, cl, cm, reaction) => {
            let command = UpdateMessageReaction(cr, state.id, cm, Some(reaction));
            let content = serde_json::to_string(&command)
                .unwrap()
                .as_bytes()
                .to_owned();
            let message = ReqChatSend {
                to: cl,
                chat_msg: content,
            };
            state.send_message(message, cr, Some(cm));
        }
        RequestRoomList(cr) => {
            let message = ReqChatClients;
            state.send_message(message, cr, None);
        }
        Dead => {}
    }
    Ok(())
}
