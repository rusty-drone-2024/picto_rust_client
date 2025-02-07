use crate::network::Network;
use common_structs::leaf::LeafEvent::PacketSend;
use common_structs::message::Message;
use common_structs::types::{Routing, Session};
use crossbeam_channel::Sender;
use wg_2024::network::NodeId;
use wg_2024::packet::{FloodRequest, NodeType, Packet, PacketType};

impl Network {
    pub fn initiate_flood(&mut self) {
        //construct flood request packet
        let packet = Packet::new_flood_request(
            Routing::empty_route(),
            0,
            FloodRequest::initialize(self.current_session, self.id, NodeType::Client),
        );
        //increment session
        self.current_session += 1;
        //for every neighbor
        let mut senders = Vec::new();
        for sender in self.packet_send.values() {
            senders.push(sender.clone());
        }
        //send flood request
        for sender in senders {
            self.send_packet(packet.clone(), &sender, None);
        }
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
        if send_res.is_err() {
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
                if let Some(waiting_for_ack_session_data) = waiting_for_ack_session_data {
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
}
