use crate::network::Network;
use client_lib::communication::send_message;
use client_lib::communication::TUIEvent::Kill;
use common_structs::leaf::LeafCommand;
use common_structs::types::*;
use std::net::TcpStream;
use std::sync::MutexGuard;
use wg_2024::network::NodeId;
use wg_2024::packet::NackType::{ErrorInRouting, UnexpectedRecipient};
use wg_2024::packet::PacketType::{FloodRequest, MsgFragment};
use wg_2024::packet::{
    FloodRequest as FloodRequestData, FloodResponse, Nack, NackType, NodeType, Packet,
};

pub(crate) fn handle_command(
    net: &mut MutexGuard<Network>,
    c: LeafCommand,
    stream: &mut Option<TcpStream>,
) -> bool {
    match c {
        LeafCommand::RemoveSender(conn_id) => {
            net.remove_sender(&conn_id);
        }
        LeafCommand::AddSender(conn_id, sender) => {
            net.add_sender(conn_id, sender);
        }
        LeafCommand::Kill => {
            if let Some(stream) = stream {
                send_message(stream, Kill);
            }
            return true;
        }
    }

    false
}

pub(crate) fn find_routing_error(id: NodeId, packet: &Packet) -> Option<NackType> {
    if let FloodRequest(_) = &packet.pack_type {
        return None;
    }

    let routing = &packet.routing_header;

    if Some(id) != routing.current_hop() {
        return Some(UnexpectedRecipient(id));
    } else if let Some(next) = routing.next_hop() {
        return Some(ErrorInRouting(next));
    }

    None
}

pub(crate) fn handle_routing_error(
    net: &mut MutexGuard<Network>,
    packet: Packet,
    nack_type: NackType,
) {
    let MsgFragment(fragment) = &packet.pack_type else {
        return net.controller_shortcut(packet);
    };

    let id = net.id;
    net.send_packet(&new_nack(
        id,
        packet.routing_header,
        packet.session_id,
        fragment.fragment_index,
        nack_type,
    ));
}

fn new_ack(mut routing: Routing, session: Session, fragment_id: FragmentIdx) -> Packet {
    routing.reverse();
    routing.increase_hop_index();

    Packet::new_ack(routing, session, fragment_id)
}

fn new_nack(
    self_id: NodeId,
    routing: Routing,
    session: Session,
    fragment_id: FragmentIdx,
    nack_type: NackType,
) -> Packet {
    let hops = routing
        .hops
        .into_iter()
        .take(routing.hop_index)
        .chain(Some(self_id))
        .rev()
        .collect();

    let nack = Nack {
        fragment_index: fragment_id,
        nack_type,
    };

    let routing = Routing::new(hops, 1);
    Packet::new_nack(routing, session, nack)
}

fn new_flood_resp(
    self_id: NodeId,
    self_type: NodeType,
    session: Session,
    flood: FloodRequestData,
) -> Packet {
    let flood_id = flood.flood_id;
    let mut path_trace = flood.path_trace;

    path_trace.push((self_id, self_type));
    let hops = path_trace.iter().map(|(id, _)| *id).rev().collect();

    Packet::new_flood_response(
        Routing::with_first_hop(hops),
        session,
        FloodResponse {
            flood_id,
            path_trace,
        },
    )
}
