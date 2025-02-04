use client::Client;
use common_structs::leaf::Leaf;
use crossbeam_channel::unbounded;
use std::collections::HashMap;

fn main() {
    let (controller_send, _) = unbounded();
    let (_, controller_recv) = unbounded();
    let (_, packet_recv) = unbounded();
    let mut client = Client::new(
        1,
        controller_send,
        controller_recv,
        packet_recv,
        HashMap::new(),
    );
    client.run();
}
