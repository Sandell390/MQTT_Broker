use crate::models::topicfilter::Topicfilter;

pub struct SubInfo{
    pub packet_id: u16,
    pub topic_qos_pair: Vec<Topicfilter>,
    pub return_packet: Vec<u8>
}

impl SubInfo {
    pub fn get_packet_id_bytes(&self) -> [u8; 2] {
        u16::to_be_bytes(self.packet_id)
    }
}