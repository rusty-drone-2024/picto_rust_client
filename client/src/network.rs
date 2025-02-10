mod pack_in;
mod pack_out;
mod topology;
mod utils;

use common_structs::leaf::LeafEvent;
use common_structs::message::{Message, ServerType};
use common_structs::types::Session;
use crossbeam_channel::Sender;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;
use std::net::TcpStream;
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, Packet};

pub(super) struct Network {
    pub id: NodeId,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    controller_send: Sender<LeafEvent>,
    topology: DiGraphMap<NodeId, i32>,
    //HM<session, (serverId, OtherClientId, Waiting)>
    packs_waiting_for_ack: HashMap<u64, (NodeId, Option<NodeId>, Vec<Packet>)>,
    //HM<session, message>
    pub messages_waiting_for_ack: HashMap<u64, Message>,
    //HM<serverId, (OtherClientId, Waiting)>
    queued_packs: HashMap<NodeId, (Option<NodeId>, Vec<Packet>)>,
    //HM<serverId (Path)>
    paths_to_leafs: HashMap<NodeId, Option<Vec<NodeId>>>,
    leaf_types: HashMap<NodeId, Option<ServerType>>,
    partially_received: HashMap<u64, Vec<Option<Fragment>>>,
    current_session: Session,
    pub frontend_stream: Option<TcpStream>,
}

impl Network {
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
}
