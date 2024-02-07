use std::net::SocketAddr;

pub enum PublishItemState {
    AwaitingPuback,
    PubackRecieved,
    AwaitingPubrec,
    PubrecRecieved,
    AwaitingPubrel,
    PubrelRecieved,
    AwaitingPubcomp,
    PubcompRecieved,
}

pub enum PublishItemDirection {
    ToSubscriber,
    FromClient,
}

pub struct PublishQueueItem {
    pub packet_id: usize,
    pub publish_packet: Vec<u8>,
    pub state: PublishItemState,
    pub qos_level: u8,
    pub flow_direction: PublishItemDirection,
    pub socket_addr: SocketAddr,
}
