#[cfg(test)]
mod tests {
    use crate::control_packet::unsubcribe::handle;

    #[test]
    fn test_handle_unsubscribe_packet() {
        // Create a simple unsubscribe packet
        // Header: 0xA2 (unsubscribe packet)
        // Length: 0x07 (7 remaining bytes)
        // Packet ID: 0x0001
        // Topic: "a" (length: 0x0001)
        let buffer = [0xA2, 0x05, 0x00, 0x01, 0x00, 0x01, b'a'];
        let packet_length = 7;

        // Test the handle function with the unsubscribe packet
        let result = handle(&buffer, packet_length);

        // Check that the result is Ok
        assert!(result.is_ok());

        // Check the contents of the result
        let sub_info = result.unwrap();
        assert_eq!(sub_info.packet_id, 1);
        assert_eq!(sub_info.topic_qos_pair, vec![("a".to_string(), 0)]);
        // Check the return_packet as needed
    }

    #[test]
    fn test_handle_unsubscribe_packet_multiple_topics() {
        // Create an unsubscribe packet with multiple topics
        // Header: 0xA2 (unsubscribe packet)
        // Length: 0x0D (13 remaining bytes)
        // Packet ID: 0x0001
        // Topics: "a" (length: 0x0001), "b" (length: 0x0001)
        let buffer = [0xA2, 0x08, 0x00, 0x01, 0x00, 0x01, b'a', 0x00, 0x01, b'b'];
        let packet_length = 10;

        // Test the handle function with the unsubscribe packet
        let result = handle(&buffer, packet_length);

        // Check that the result is Ok
        assert!(result.is_ok());

        // Check the contents of the result
        let unsub_info = result.unwrap();
        assert_eq!(unsub_info.packet_id, 1);
        assert_eq!(unsub_info.topic_qos_pair, vec![("a".to_string(),0 ), ("b".to_string(), 0)]);
    }

    #[test]
    fn test_handle_unsubscribe_packet_invalid_length() {
        // Create an unsubscribe packet with an invalid length
        // Header: 0xA2 (unsubscribe packet)
        // Length: 0x07 (7 remaining bytes)
        // Packet ID: 0x0001
        // Topic: "a" (length: 0x0001)
        let buffer = [0xA2, 0x07, 0x00, 0x01, 0x00, 0x01, b'a'];
        let packet_length = 6; // Incorrect packet length

        // Test the handle function with the unsubscribe packet
        let result = handle(&buffer, packet_length);

        // Check that the result is an error
        assert!(result.is_err());
    }

}