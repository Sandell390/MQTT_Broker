#[cfg(test)]
mod tests {
    use crate::control_packet::disconnect::handle;

    #[test]
    fn test_handle_disconnect_packets() {
        // Test with a normal disconnect packet
        let buffer = [0xE0, 0x00];
        let packet_length = 2;
        assert_eq!(handle(&buffer, packet_length), Ok("Reserved bits not set"));

        // Test with a disconnect packet with extra data
        let buffer = [0xE0, 0x00, 0x01];
        let packet_length = 3;
        assert_eq!(handle(&buffer, packet_length), Ok("Reserved bits not set"));

        // Test with a disconnect packet with an incorrect packet length
        let buffer = [0xE0, 0x01];
        let packet_length = 2;
        assert_eq!(handle(&buffer, packet_length), Err("Reserved bits are set"));
    }
}