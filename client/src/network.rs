use crate::communication::net::{new_ack, new_flood_resp};
use client_lib::communication::TUICommand::*;
use client_lib::communication::{send_message, MessageStatus, TUICommand};
use common_structs::leaf::LeafEvent;
use common_structs::leaf::LeafEvent::{ControllerShortcut, PacketSend};
use common_structs::message::{Message, ServerType};
use common_structs::types::{Routing, Session};
use crossbeam_channel::Sender;
use petgraph::algo::astar;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;
use std::net::TcpStream;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NodeType::{Client, Drone};
use wg_2024::packet::{
    Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType,
};

pub type Path = Vec<NodeId>;

pub(super) struct Network {
    pub id: NodeId,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    controller_send: Sender<LeafEvent>,
    topology: DiGraphMap<NodeId, i32>,
    packs_waiting_for_ack: HashMap<u64, (NodeId, Option<NodeId>, Vec<Packet>)>,
    pub messages_waiting_for_ack: HashMap<u64, Message>,
    queued_packs: HashMap<NodeId, (Option<NodeId>, Vec<Packet>)>,
    paths_to_leafs: HashMap<NodeId, Option<Vec<NodeId>>>,
    leaf_types: HashMap<NodeId, Option<ServerType>>,
    partially_received: HashMap<u64, Vec<Option<Fragment>>>,
    current_session: Session,
    pub frontend_stream: Option<TcpStream>,
}

impl Network {
    fn update_reachable_paths(&mut self) {
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            if self.paths_to_leafs.get(&leaf).is_some() {
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                if let Some((_, path)) = path {
                    self.paths_to_leafs.insert(leaf, Some(path));
                } else {
                    self.paths_to_leafs.insert(leaf, None);
                }
            }
        }
    }

    fn update_unreachable_paths(&mut self) {
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            if self.paths_to_leafs.get(&leaf).is_none() {
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                if let Some((_, path)) = path {
                    self.paths_to_leafs.insert(leaf, Some(path));
                    self.check_queued(leaf);
                }
            }
        }
    }
    pub fn new(
        id: NodeId,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        controller_send: Sender<LeafEvent>,
    ) -> Self {
        let mut topology = DiGraphMap::new();
        topology.add_node(id);
        Network {
            id,
            packet_send,
            controller_send,
            topology,
            packs_waiting_for_ack: HashMap::new(),
            messages_waiting_for_ack: Default::default(),
            queued_packs: HashMap::new(),
            paths_to_leafs: HashMap::new(),
            leaf_types: Default::default(),
            partially_received: HashMap::new(),
            current_session: 0,
            frontend_stream: None,
        }
    }
    pub fn initiate_flood(&mut self) {
        let packet = Packet::new_flood_request(
            Routing::empty_route(),
            0,
            FloodRequest::initialize(self.current_session, self.id, NodeType::Client),
        );
        self.current_session += 1;
        let mut senders = Vec::new();
        for sender in self.packet_send.values() {
            senders.push(sender.clone());
        }
        for sender in senders {
            self.send_packet(packet.clone(), &sender, None);
        }
    }
    pub fn update_topology(&mut self, subgraph: Vec<(NodeId, NodeType)>) {
        for node in 0..subgraph.len() - 1 {
            let curr = subgraph[node];
            let next = subgraph[node + 1];
            if let Drone = curr.1 {
                self.topology.add_edge(curr.0, next.0, 1);
            }
            if let Drone = next.1 {
                self.topology.add_edge(next.0, curr.0, 1);
            }
        }
        self.update_unreachable_paths();
    }

    pub fn send_message(&mut self, message: Message, target: NodeId, session: Option<Session>) {
        let session = if let Some(session) = session {
            session
        } else {
            self.current_session
        };
        self.messages_waiting_for_ack
            .insert(session, message.clone());
        let recipient = match message {
            Message::ReqChatSend { to: recipient, .. } => Some(recipient),
            _ => None,
        };
        let frags = message.into_fragments();
        let mut send_now = false;
        let mut routing = Routing::empty_route();
        let path = self.paths_to_leafs.get(&target);
        if let Some(Some(path)) = path {
            routing = Routing::new(path.clone(), 1);
            send_now = true;
        }
        for frag in frags {
            let pack = Packet::new_fragment(routing.clone(), session, frag.clone());
            let queue_data = self.queued_packs.remove(&target);
            if let Some(queue_data) = queue_data {
                let mut queue = queue_data.1;
                queue.push(pack.clone());
                self.queued_packs.insert(target, (recipient, queue));
            } else {
                self.queued_packs.insert(target, (recipient, vec![pack]));
            }
        }
        if send_now {
            self.check_queued(target);
        }

        self.current_session += 1;
    }

    pub fn send_packet(
        &mut self,
        pack: Packet,
        sender: &Sender<Packet>,
        recipient: Option<NodeId>,
    ) {
        let send_res = sender.send(pack.clone());
        if let Err(_) = send_res {
            match &pack.pack_type {
                PacketType::MsgFragment(_) => {
                    let pack = pack.clone();
                    let leaf = *pack.routing_header.hops.last().unwrap();
                    let queue_data = self.queued_packs.remove(&leaf);
                    if let Some(queue_data) = queue_data {
                        let mut queue = queue_data.1;
                        queue.push(pack);
                        self.queued_packs.insert(leaf, (recipient, queue));
                    } else {
                        self.queued_packs.insert(leaf, (recipient, vec![pack]));
                    }
                }
                PacketType::FloodRequest(_) => {
                    //do nothing
                }
                _ => {
                    self.controller_shortcut(pack.clone());
                }
            }
        } else {
            let _ = self.controller_send.send(PacketSend(pack.clone()));
            if let PacketType::MsgFragment(_) = &pack.pack_type {
                let waiting_for_ack_session_data =
                    self.packs_waiting_for_ack.remove(&pack.session_id);
                let server = *pack.routing_header.hops.last().unwrap();
                if let Some(mut waiting_for_ack_session_data) = waiting_for_ack_session_data {
                    let mut waiting_for_ack_session = waiting_for_ack_session_data.2.clone();
                    waiting_for_ack_session.push(pack.clone());
                    self.packs_waiting_for_ack.insert(
                        pack.session_id,
                        (server, recipient, waiting_for_ack_session),
                    );
                } else {
                    self.packs_waiting_for_ack
                        .insert(pack.session_id, (server, recipient, vec![pack.clone()]));
                }
            }
        }
    }

    pub fn check_queued(&mut self, leaf: NodeId) {
        let packs = self.queued_packs.remove(&leaf);
        if let Some(packs) = packs {
            for mut pack in packs.1 {
                let mut routing = pack.routing_header;
                if routing.is_empty() {
                    let route = self.paths_to_leafs.get(&leaf);
                    if let Some(Some(route)) = route {
                        routing = SourceRoutingHeader::new(route.clone(), 1);
                    }
                }
                pack.routing_header = routing;
                let first_hop = pack.routing_header.hops[1];
                let sender = self.packet_send.get(&first_hop);
                if let Some(sender) = sender {
                    self.send_packet(pack, &sender.clone(), packs.0);
                }
            }
        }
    }

    pub fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
        self.topology.add_edge(self.id, id, 1);
        self.topology.add_edge(id, self.id, 1);
        self.update_unreachable_paths();
    }
    pub fn remove_sender(&mut self, id: &NodeId) {
        self.packet_send.remove(id);
        self.topology.remove_edge(self.id, *id);
        self.topology.remove_edge(*id, self.id);
        self.update_reachable_paths();
    }
    pub fn controller_shortcut(&mut self, packet: Packet) {
        let _ = self.controller_send.send(ControllerShortcut(packet));
    }

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
                            let _ = send_message(stream, UpdateChatRoom(server, Some(true), None));
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
    }
    fn handle_flood_request_receive(&mut self, session: Session, flood_request: FloodRequest) {
        let flood_res = new_flood_resp(self.id, Client, session, flood_request);
        let sender = flood_res.routing_header.hops[1];
        let sender = self.packet_send.get(&sender).unwrap().clone();
        self.send_packet(flood_res, &sender, None);
    }
    fn handle_flood_response_receive(&mut self, flood_response: FloodResponse) {
        if flood_response.flood_id != self.current_session {
            return;
        }
        let last = flood_response.path_trace.last().copied();
        if let Some((id, node_type)) = last {
            let path = flood_response
                .path_trace
                .into_iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>();

            // Only add last as only leaf are valid destination (which are always at end)
            if node_type == NodeType::Drone {
                let _ = self.add_path(&path, false);
                return;
            }

            let _ = self.add_path(&path, true);
            self.check_queued(id);
        }
    }

    fn add_path(&mut self, path: &[NodeId], is_last_leaf: bool) -> Result<(), String> {
        if Some(self.id) != path.first().copied() {
            return Err("Path does not start with this node".to_string());
        }

        let windows = path.windows(2);
        let last_index = windows.len() - 1;

        for (i, window) in windows.enumerate() {
            let a = window[0];
            let b = window[1];

            self.topology.add_edge(a, b, 1);
            if i != 0 && !(is_last_leaf && i == last_index) {
                self.topology.add_edge(b, a, 1);
            }
        }

        Ok(())
    }
}
