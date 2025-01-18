use crate::ClientError;
use crate::ClientError::{SerializationError, StreamError};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

pub type ChatServerID = u32;
pub type ChatClientID = u32;
pub type MessageID = u32;
pub type TimeStamp = u32;
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Reaction {
    Like,
    Heart,
    Skull,
    Crying,
    Star,
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    SentToServer,
    ReceivedByServer,
    ReceivedByPeer,
    ReadByPeer,
    MessageFromPeer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TUIEvent {
    SendMessage(ChatServerID, ChatClientID, MessageID, MessageContent),
    ReadMessage(ChatServerID, ChatClientID, MessageID),
    DeleteMessage(ChatServerID, ChatClientID, MessageID),
    ReactToMessage(ChatServerID, ChatClientID, MessageID, Reaction),

    SetName(String),

    // Update with NodeID type defined in WGL
    RegisterToServer(ChatServerID),
    RequestRoomList(ChatServerID),

    Kill,
}

pub type MessageContent = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TUICommand {
    // Name
    UpdateName(String),

    // ChatRoom
    // ChatServerID, registered_to, net_reachable
    UpdateChatRoom(ChatServerID, Option<bool>, Option<bool>),

    // ChatLog
    UpdatePeerName(ChatServerID, ChatClientID, String),
    UpdatePeerLastSeen(ChatServerID, ChatClientID),
    UpdatePeerStatus(ChatServerID, ChatClientID, bool),

    // Message
    UpdateMessageContent(ChatServerID, ChatClientID, MessageID, MessageContent),
    UpdateMessageStatus(ChatServerID, ChatClientID, MessageID, MessageStatus),
    UpdateMessageReaction(ChatServerID, ChatClientID, MessageID, Option<Reaction>),
    DeleteMessage(ChatClientID, ChatClientID, MessageID),

    Kill,
}

pub fn send_message<T: Serialize>(stream: &mut TcpStream, message: T) -> Result<(), ClientError> {
    let serialized = serde_json::to_string(&message).map_err(|_| SerializationError)?;
    let len = serialized.len() as u32;
    stream
        .write_all(&len.to_be_bytes())
        .map_err(|_| StreamError)?;
    stream
        .write_all(serialized.as_bytes())
        .map_err(|_| StreamError)?;
    stream.flush().map_err(|_| StreamError)?;
    Ok(())
}

pub fn receive_message<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
) -> Result<T, ClientError> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).map_err(|_| StreamError)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).map_err(|_| StreamError)?;

    serde_json::from_slice(&buffer).map_err(|_| SerializationError)
}
