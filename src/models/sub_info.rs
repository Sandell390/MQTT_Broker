use crate::models::topic::Topic;

pub struct SubInfo {
    pub packet_id: u16,
    pub topic_qos_pair: Vec<(String, u8)>,
    pub return_packet: Vec<u8>,
}

impl SubInfo {
    pub fn get_packet_id_bytes(&self) -> [u8; 2] {
        u16::to_be_bytes(self.packet_id)
    }
}
