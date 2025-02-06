mod communication;
mod helpers;
mod network;

use crate::communication::net::*;
use crate::communication::tui::tui_event_receiver;
use crate::helpers::{get_stream, new_listener, start_tui};
use crate::network::Network;
use client_lib::communication::send_message;
use client_lib::communication::TUICommand::UpdateName;
use client_lib::ClientError::{LockError, StreamError};
use common_structs::leaf::{Leaf, LeafCommand, LeafEvent};
use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

pub struct Client {
    controller_recv: Receiver<LeafCommand>,
    packet_recv: Receiver<Packet>,
    network: Arc<Mutex<Network>>,
}

impl Client {
    fn get_id(&self) -> NodeId {
        self.network.lock().unwrap().id
    }
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
        }
    }

    fn run(&mut self) {
        //START CLIENT TUI AND GET TCP CONNECTION TO IT
        let listener = new_listener().unwrap();
        start_tui(&listener).unwrap();
        let events_frontend_stream = get_stream(listener).unwrap();
        let mut net = self.network.lock().unwrap();
        net.frontend_stream = Some(
            events_frontend_stream
                .try_clone()
                .map_err(|_| StreamError)
                .unwrap(),
        );
        let id = self.get_id();
        if let Some(stream) = &mut net.frontend_stream {
            let _ = send_message(stream, UpdateName(format!("client_{}", id)));
        }
        drop(net);

        //INITIALIZE STATE
        let net_front = Arc::clone(&self.network);
        let net_back = Arc::clone(&self.network);

        //TUI EVENT RECEIVER THREAD
        thread::spawn(move || {
            tui_event_receiver(net_front, events_frontend_stream);
        });

        let mut exit = false;
        /*
        net_back
            .lock()
            .map_err(|_| LockError)
            .unwrap()
            .initiate_flood();
         */
        while !exit {
            select! {
                recv(self.controller_recv) -> msg =>{
                    let mut net_back = net_back.lock().map_err(|_| LockError).unwrap();
                    let Ok(comm) = msg else {continue;};
                    exit = handle_command(&mut net_back, comm);
                    drop(net_back);
                },
                recv(self.packet_recv) -> msg => {
                    let mut net_back = net_back.lock().map_err(|_| LockError).unwrap();
                    let Ok(packet) = msg else{continue;};
                    if let Some(error) = find_routing_error(net_back.id, &packet){
                        handle_routing_error(&mut net_back, packet, error);
                        continue;
                    }
                    net_back.handle_packet(packet);
                    drop(net_back);
                }
            }
        }
    }
}
