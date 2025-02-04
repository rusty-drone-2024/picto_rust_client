mod communication;
mod helpers;
mod network;

use crate::communication::tui::tui_event_receiver;
use crate::helpers::{get_stream, new_listener, start_tui};
use crate::network::Network;
use client_lib::ClientError::StreamError;
use common_structs::leaf::{Leaf, LeafCommand, LeafEvent};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use wg_2024::network::NodeId;
use wg_2024::packet::{Fragment, Packet};

pub struct Client {
    controller_recv: Receiver<LeafCommand>,
    packet_recv: Receiver<Packet>,
    network: Arc<Mutex<Network>>,
    partially_received: HashMap<u64, Vec<Option<Fragment>>>,
}
impl Leaf for Client {
    fn new(
        id: NodeId,
        controller_send: Sender<LeafEvent>,
        controller_recv: Receiver<LeafCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self
    where
        Self: Sized,
    {
        Client {
            controller_recv,
            packet_recv,
            network: Arc::new(Mutex::new(Network::new(id, packet_send, controller_send))),
            partially_received: HashMap::new(),
        }
    }

    fn run(&mut self) {
        //START CLIENT TUI AND GET TCP CONNECTION TO IT
        let listener = new_listener().unwrap();
        start_tui(&listener).unwrap();
        let stream = get_stream(listener).unwrap();

        //INITIALIZE STATE
        let state_clone_front = Arc::clone(&self.network);
        let state_clone_back = Arc::clone(&self.network);
        let tui_events_stream = stream.try_clone().map_err(|_| StreamError).unwrap();

        //TUI EVENT RECEIVER THREAD
        thread::spawn(move || {
            tui_event_receiver(state_clone_front, tui_events_stream);
        });

        loop {}
    }
}
