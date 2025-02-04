use common_structs::message::Message;
use wg_2024::network::NodeId;
pub type Path = Vec<NodeId>;

pub(super) struct Network {
    topology: (),
    pending: Vec<Message>,
}

impl Network {
    fn compute_path() -> Result<Path, ()> {
        todo!()
    }
    pub fn new() -> Self {
        //TODO:
        Network {
            topology: (),
            pending: Vec::new(),
        }
    }
    pub fn initiate_flood() {
        todo!()
    }
    pub fn update_topology(subgraph: ()) {
        todo!()
    }
    pub fn forward_message(msg: Message, target: NodeId) -> Result<(), ()> {
        todo!()
    }
}
