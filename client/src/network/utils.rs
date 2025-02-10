use crate::network::Network;
use common_structs::leaf::LeafEvent::ControllerShortcut;
use crossbeam_channel::Sender;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::Packet;

impl Network {
    pub fn get_id(&self) -> NodeId {
        self.id
    }
    pub fn check_queued(&mut self, leaf: NodeId) {
        let packs = self.queued_packs.remove(&leaf);
        if let Some(packs) = packs {
            for mut pack in packs.1 {
                let mut routing = pack.routing_header.clone();
                let route = self.paths_to_leafs.get(&leaf);
                if let Some(Some(route)) = route {
                    routing = SourceRoutingHeader::new(route.clone(), 1);
                }
                pack.routing_header = routing;
                let first_hop = pack.routing_header.hops.get(1);
                if let Some(first_hop) = first_hop {
                    let sender = self.packet_send.get(first_hop);
                    if let Some(sender) = sender {
                        self.send_packet(pack, &sender.clone(), packs.0);
                    }
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
}
