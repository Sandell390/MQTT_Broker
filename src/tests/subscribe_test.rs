#[cfg(test)]
mod tests {
    use crate::control_packet::subcribe::handle;

    #[test]
    fn test_handle_subscribe_packet() {
        // Create a simple subscribe packet
        // Header: 0x82 (subscribe packet, QoS 1)
        // Length: 0x07 (7 remaining bytes)
        // Packet ID: 0x0001
        // Topic: "a" (length: 0x0001, QoS: 0x00)
        let buffer = [0x82, 0x06, 0x00, 0x01, 0x00, 0x01, b'a', 0x00];
        let packet_length = 8;

        // Test the handle function with the subscribe packet
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
    fn test_handle_subscribe_packet_multiple_topics() {
        // Create a subscribe packet with multiple topics
        // Header: 0x82 (subscribe packet, QoS 1)
        // Length: 0x0D (13 remaining bytes)
        // Packet ID: 0x0001
        // Topics: "a" (length: 0x0001, QoS: 0x00), "b" (length: 0x0001, QoS: 0x01)
        let buffer = [0x82, 0x0A, 0x00, 0x01, 0x00, 0x01, b'a', 0x00, 0x00, 0x01, b'b', 0x01];
        let packet_length = 12;

        // Test the handle function with the subscribe packet
        let result = handle(&buffer, packet_length);

        // Check that the result is Ok
        assert!(result.is_ok());

        // Check the contents of the result
        let sub_info = result.unwrap();
        assert_eq!(sub_info.packet_id, 1);
        assert_eq!(sub_info.topic_qos_pair, vec![("a".to_string(), 0), ("b".to_string(), 1)]);
    }

    #[test]
    fn test_handle_subscribe_packet_invalid_qos() {
        // Create a subscribe packet with an invalid QoS
        // Header: 0x82 (subscribe packet, QoS 1)
        // Length: 0x07 (7 remaining bytes)
        // Packet ID: 0x0001
        // Topic: "a" (length: 0x0001, QoS: 0x03)
        let buffer = [0x82, 0x06, 0x00, 0x01, 0x00, 0x01, b'a', 0x03];
        let packet_length = 8;

        // Test the handle function with the subscribe packet
        let result = handle(&buffer, packet_length);

        // Check that the result is an error
        assert!(result.is_ok());
    }
}