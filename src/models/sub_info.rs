pub struct SubInfo {
    pub packet_id: u16,
    pub topic_qos_pair: Vec<(String, u8)>,
    pub return_packet: Vec<u8>,
}
