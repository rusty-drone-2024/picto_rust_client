use common_structs::leaf::LeafEvent;
use common_structs::leaf::LeafEvent::{ControllerShortcut, PacketSend};
use common_structs::message::Message;
use common_structs::types::Routing;
use crossbeam_channel::Sender;
use petgraph::algo::astar;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;
use wg_2024::network::NodeId;
use wg_2024::packet::NodeType::Drone;
use wg_2024::packet::{FloodRequest, Fragment, NodeType, Packet, PacketType};

pub type Path = Vec<NodeId>;

pub(super) struct Network {
    pub id: NodeId,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    controller_send: Sender<LeafEvent>,
    topology: DiGraphMap<NodeId, i32>,
    packs_waiting_for_ack: Vec<Packet>,
    queued_packs: HashMap<NodeId, Vec<Packet>>,
    paths_to_leafs: HashMap<NodeId, Option<Vec<NodeId>>>,
    partially_received: HashMap<u64, (u64, Vec<Option<Fragment>>)>,
    current_flood: u64,
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
                    self.check_pending(leaf);
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
            packs_waiting_for_ack: Vec::new(),
            queued_packs: HashMap::new(),
            paths_to_leafs: HashMap::new(),
            partially_received: HashMap::new(),
            current_flood: 0,
        }
    }
    pub fn initiate_flood(&mut self) {
        let packet = Packet::new_flood_request(
            Routing::empty_route(),
            0,
            FloodRequest::initialize(self.current_flood, self.id, NodeType::Client),
        );
        self.current_flood += 1;
        self.send_packet(&packet);
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

    pub fn send_message(&mut self, message: Message, target: NodeId) {
        let frags = message.into_fragments();
        //TODO:
        // queue each frag
        // if path to bastardo
        // remove frag from queue
        // send frag
    }

    pub fn send_packet(&mut self, pack: &Packet) {
        let mut senders = Vec::new();
        for sender in self.packet_send.values() {
            senders.push(sender.clone());
        }
        for sender in senders {
            let send_res = sender.send(pack.clone());
            if let Err(_) = send_res {
                match &pack.pack_type {
                    PacketType::MsgFragment(_) => {
                        let pack = pack.clone();
                        let leaf = *pack.routing_header.hops.last().unwrap();
                        let queue = self.queued_packs.remove(&leaf);
                        if let Some(mut queue) = queue {
                            queue.push(pack);
                            self.queued_packs.insert(leaf, queue);
                        } else {
                            let new_queue = vec![pack];
                            self.queued_packs.insert(leaf, new_queue);
                        }
                    }
                    PacketType::FloodRequest(_) => {
                        //TODO: wtf? shouldn't happen
                    }
                    _ => {
                        self.controller_shortcut(pack.clone());
                    }
                }
            } else {
                let _ = self.controller_send.send(PacketSend(pack.clone()));
                if let PacketType::MsgFragment(_) = &pack.pack_type {
                    self.packs_waiting_for_ack.push(pack.clone());
                }
            }
        }
    }

    pub fn check_pending(&mut self, leaf: NodeId) {
        let packs = &self.queued_packs.remove(&leaf);
        if let Some(packs) = packs {
            for pack in packs {
                self.send_packet(pack);
            }
        }
    }

    pub fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
        self.topology.add_edge(self.id, id, 1);
        self.update_unreachable_paths();
    }
    pub fn remove_sender(&mut self, id: &NodeId) {
        self.packet_send.remove(id);
        self.topology.remove_edge(self.id, *id);
        self.update_reachable_paths();
    }
    pub fn controller_shortcut(&mut self, packet: Packet) {
        let _ = self.controller_send.send(ControllerShortcut(packet));
    }

    pub fn handle_packet(&mut self, packet: &Packet) {}
}
