use common_structs::leaf::LeafEvent;
use common_structs::message::Message;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, Packet};

pub type Path = Vec<NodeId>;

pub(super) struct Network {
    id: NodeId,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    controller_send: Sender<LeafEvent>,
    topology: (),
    frags_waiting_for_ack: Vec<Fragment>,
    queued_frags: HashMap<NodeId, Vec<Fragment>>,
    unreachable_leafs: Vec<NodeId>,
}

impl Network {
    fn compute_path(&self, target: NodeId) -> Result<Path, ()> {
        todo!()
        //if target is reachable
        //  path
        //else
        //  error
    }
    pub fn new(
        id: NodeId,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        controller_send: Sender<LeafEvent>,
    ) -> Self {
        //TODO: implement topology
        Network {
            id,
            packet_send,
            controller_send,
            topology: (),
            frags_waiting_for_ack: Vec::new(),
            queued_frags: HashMap::new(),
            unreachable_leafs: Vec::new(),
        }
    }
    pub fn initiate_flood(&mut self) {
        todo!()
        //create flood packet
        //forward packets to packet_send's channels
    }
    pub fn update_topology(&mut self, subgraph: ()) {
        todo!()
        //add subgraph to path
        //call check_pending
    }

    pub fn send_message(&mut self, message: Message, target: NodeId) {
        todo!()
        //serialize msg into frags
        //queue each frag
        //if path exists
        //  send frag
    }

    pub fn send_fragment(&mut self, fragment: Fragment, target: NodeId) {
        todo!()
        //remove frag from queue
        //add frag to frags_waiting_for_ack
        //forward frag
        //notify SC
    }

    pub fn check_pending(&mut self) {
        todo!()
        //foreach node in unreachable
        //  if path to node exists
        //    foreach fragment in queue for that node
        //      call send_frag
    }
}
