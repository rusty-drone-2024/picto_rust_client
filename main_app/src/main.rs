use client::Client;
use common_structs::leaf::Leaf;
use crossbeam_channel::unbounded;
use std::collections::HashMap;
use std::thread;

fn main() {
    let (controller_send1, _) = unbounded();
    let (_, controller_recv1) = unbounded();
    let (_, packet_recv1) = unbounded();
    let mut client1 = Client::new(
        1,
        controller_send1,
        controller_recv1,
        packet_recv1,
        HashMap::new(),
    );

    let (controller_send2, _) = unbounded();
    let (_, controller_recv2) = unbounded();
    let (_, packet_recv2) = unbounded();
    let mut client2 = Client::new(
        2,
        controller_send2,
        controller_recv2,
        packet_recv2,
        HashMap::new(),
    );
    thread::spawn(move || {
        client1.run();
    });
    thread::spawn(move || {
        client2.run();
    });
    loop {}
}
