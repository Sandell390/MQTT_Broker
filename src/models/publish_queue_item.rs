use std::{sync::mpsc::Sender, time::Instant};

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
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

#[derive( Debug)]
#[allow(dead_code)]
pub enum PublishItemDirection {
    ToSubscriber,
    FromClient,
}

#[derive( Debug)]
pub struct PublishQueueItem {
    pub packet_id: usize,
    pub timestamp_sent: Instant,
    pub publish_packet: Vec<u8>,
    pub state: PublishItemState,
    pub qos_level: u8,
    pub flow_direction: PublishItemDirection,
    pub tx: Sender<PublishItemState>
}
