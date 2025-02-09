use crate::communication::net::{new_ack, new_flood_resp};
use crate::network::Network;
use client_lib::communication::TUICommand::{
    DeleteMessage, UpdateChatRoom, UpdateMessageContent, UpdateMessageReaction,
    UpdateMessageStatus, UpdatePeerLastSeen, UpdatePeerName,
};
use client_lib::communication::{send_message, MessageStatus, TUICommand};
use common_structs::message::{Message, ServerType};
use common_structs::types::{Routing, Session};
use wg_2024::packet::NodeType::{Client, Server};
use wg_2024::packet::{
    Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType,
};

impl Network {
    pub fn handle_packet(&mut self, packet: Packet) {
        let (routing, session, pack_type) =
            (packet.routing_header, packet.session_id, packet.pack_type);
        match pack_type {
            PacketType::MsgFragment(f) => self.handle_fragment_receive(routing, session, f),
            PacketType::Ack(a) => self.handle_ack_receive(session, a),
            PacketType::Nack(n) => self.handle_nack_receive(routing, session, n),
            PacketType::FloodRequest(f_req) => self.handle_flood_request_receive(session, f_req),
            PacketType::FloodResponse(f_res) => self.handle_flood_response_receive(f_res),
        }
    }
    fn handle_fragment_receive(&mut self, routing: Routing, session: Session, fragment: Fragment) {
        let mut partial_message = Vec::with_capacity(fragment.total_n_fragments as usize);
        if let Some(stored_partial_message) = self.partially_received.remove(&session) {
            partial_message = stored_partial_message;
        }
        partial_message[fragment.fragment_index as usize] = Some(fragment.clone());

        let mut complete = true;
        for frag in &partial_message {
            if frag.is_none() {
                complete = false;
            }
        }

        if complete {
            let mut message = Vec::new();
            for frag in partial_message.iter().flatten() {
                message.push(frag.clone());
            }
            let message = Message::from_fragments(message);
            //if message makes sense
            if let Ok(message) = message {
                match message {
                    Message::RespServerType(server_type) => {
                        let server = routing.hops[0];
                        self.leaf_types.insert(server, Some(server_type.clone()));
                        if server_type == ServerType::Chat {
                            self.check_queued(server);
                        }
                    }
                    Message::RespClientList(peers) => {
                        let server = routing.hops[0];
                        for peer in peers {
                            if let Some(ref mut stream) = self.frontend_stream {
                                let _ = send_message(stream, UpdatePeerName(server, peer, None));
                            }
                        }
                    }
                    Message::RespChatFrom { chat_msg, .. } => {
                        if let Some(ref mut stream) = self.frontend_stream {
                            let content: Result<TUICommand, _> = serde_json::from_slice(&chat_msg);
                            if let Ok(content) = content {
                                match content {
                                    UpdatePeerName(_, _, _) //after SetName
                                    | UpdatePeerLastSeen(_, _) //when interacting with room
                                    | UpdateMessageStatus(_, _, _, _) //DONE: after read message or received
                                    | UpdateMessageReaction(_, _, _, _)
                                    | DeleteMessage(_, _, _) => { //DONE: after DeleteMessage
                                        let _ = send_message(stream, content);
                                    }
                                    UpdateMessageContent(_, _, _, _) => {
                                        let _ = send_message(stream, content);
                                        //send received to peer
                                    }//DONE: after SendMessage
                                    _ => {}
                                }
                            }
                        }
                    }
                    Message::ErrUnsupportedRequestType | Message::ErrNotExistentClient => {}
                    _ => {}
                }
            }
        } else {
            self.partially_received.insert(session, partial_message);
        }
        let ack = new_ack(routing, session, fragment.fragment_index);
        let sender = ack.routing_header.hops[1];
        let sender = self.packet_send.get(&sender).unwrap().clone();
        self.send_packet(ack, &sender, None);
    }
    fn handle_ack_receive(&mut self, session: Session, ack: Ack) {
        let waiting_for_ack_session = self.packs_waiting_for_ack.remove(&session);
        if let Some(mut waiting_for_ack_session) = waiting_for_ack_session {
            let mut remove = None;
            for (i, pack) in waiting_for_ack_session.2.iter().enumerate() {
                if let PacketType::MsgFragment(f) = &pack.pack_type {
                    if f.fragment_index == ack.fragment_index {
                        remove = Some(i);
                    }
                }
            }
            if let Some(i) = remove {
                waiting_for_ack_session.2.remove(i);
            }
            if waiting_for_ack_session.2.is_empty() {
                let message = self.messages_waiting_for_ack.remove(&session);
                let server = waiting_for_ack_session.0;
                let recipient = waiting_for_ack_session.1;
                if let Some(ref mut stream) = &mut self.frontend_stream {
                    if let Some(recipient) = recipient {
                        let _ = send_message(
                            stream,
                            UpdateMessageStatus(
                                server,
                                recipient,
                                session,
                                MessageStatus::ReceivedByServer,
                            ),
                        );
                        if let Some(Message::ReqChatRegistration) = message {
                            let _ = send_message(
                                stream,
                                UpdateChatRoom(server, Some(true), Some(true)),
                            );
                        }
                    }
                }
            } else {
                self.packs_waiting_for_ack
                    .insert(session, waiting_for_ack_session);
            }
        }
    }
    fn handle_nack_receive(&mut self, routing: Routing, session: Session, nack: Nack) {
        if let NackType::ErrorInRouting(node_id) = nack.nack_type {
            self.topology.remove_node(node_id);
            self.update_reachable_paths();
        }

        let packets_waiting_for_session = self.packs_waiting_for_ack.get(&session);
        let leaf = routing.hops[0];
        if let Some(packs) = packets_waiting_for_session {
            let pack = packs.2.iter().find(|p| {
                if let PacketType::MsgFragment(f) = &p.pack_type {
                    return f.fragment_index == nack.fragment_index;
                }
                false
            });
            if let Some(pack) = pack {
                //requeue packet
                let queue_data = self.queued_packs.remove(&leaf);
                if let Some(queue_data) = queue_data {
                    let mut queue = queue_data.1;
                    queue.push(pack.clone());
                    self.queued_packs.insert(leaf, (packs.1, queue));
                } else {
                    self.queued_packs
                        .insert(leaf, (packs.1, vec![pack.clone()]));
                }

                //recheck queue for leaf
                self.check_queued(leaf);
            }
        }
        self.initiate_flood();
    }
    fn handle_flood_request_receive(&mut self, session: Session, flood_request: FloodRequest) {
        let flood_res = new_flood_resp(self.id, Client, session, flood_request);
        let sender = flood_res.routing_header.hops[1];
        let sender = self.packet_send.get(&sender).unwrap().clone();
        self.send_packet(flood_res, &sender, None);
    }
    fn handle_flood_response_receive(&mut self, flood_response: FloodResponse) {
        let last = flood_response.path_trace.last().copied();
        if let Some((id, node_type)) = last {
            let path = flood_response
                .path_trace
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>();

            // Only add last as only leaf are valid destination (which are always at end)
            match node_type {
                Client => {}
                NodeType::Drone => {
                    let _ = self.add_path(&path, false);
                    return;
                }
                Server => {
                    self.leaf_types.insert(id, None);
                    self.paths_to_leafs.insert(id, None);
                    self.update_unreachable_paths();
                    if let Some(stream) = &mut self.frontend_stream {
                        let _ = send_message(stream, UpdateChatRoom(id, None, Some(true)));
                    }
                }
            }

            let _ = self.add_path(&path, true);
            if node_type == Server {
                self.send_message(Message::ReqServerType, id, None);
            }
            self.check_queued(id);
        }
    }
}
