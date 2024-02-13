#[cfg(test)]
mod tests {
    use crate::control_packet;
    use crate::control_packet::connect::handle;
    use std::sync::mpsc::channel;

    #[test]
    fn test_handle_valid_packet() {
        let mut buffer: [u8; 8192] = [0; 8192];
        // Fill buffer with valid data

        let packet = [
            0b0001_0000, // CONNECT
            16, // Remaining length
            0x00,
            0x04, // Protocol name length
            b'M',
            b'Q',
            b'T',
            b'T', // Protocol name
            0x04, // Protocol level (4 for MQTT 3.1.1)
            0x02, // Connect flags (Clean session)
            0x00,
            0x3c, // Keep alive (60 seconds)
            0x00,
            0x04, // Client ID length
            b't',
            b'e',
            b's',
            b't', // Client ID
        ];
        buffer[..packet.len()].copy_from_slice(&packet);

        let packet_length = packet.len().clone(); // Set to valid packet length
        let socket_addr = "127.0.0.1:12345".parse().unwrap();
        let mut clients = Vec::new();
        let (tx, _rx) = channel();

        let result = control_packet::connect::handle(
            buffer,
            packet_length,
            socket_addr,
            &mut clients,
            tx
        );

        assert!(result.is_ok());
        // Further assertions based on expected Response
        // ...
    }

    #[test]
    fn test_handle_invalid_protocol() {
        let mut buffer: [u8; 8192] = [0; 8192];
        // Fill buffer with invalid data
        let packet = [
            0b0001_0000, // CONNECT
            0x0f, // Remaining length
            0x00,
            0x04, // Protocol name length
            b'M',
            b'Q',
            b'T',
            b'T', // Protocol name
            0x05, // Invalid protocol level (5 instead of 4 for MQTT 3.1.1)
            0x02, // Connect flags (Clean session)
            0x00,
            0x3c, // Keep alive (60 seconds)
            0x00,
            0x04, // Client ID length
            b't',
            b'e',
            b's',
            b't', // Client ID
        ];
        buffer[..packet.len()].copy_from_slice(&packet);

        let packet_length = packet.len(); // Set to invalid packet length
        let socket_addr = "127.0.0.1:12345".parse().unwrap();
        let mut clients = Vec::new();
        let (tx, _rx) = channel();

        let result = handle(buffer, packet_length, socket_addr, &mut clients, tx);

        assert!(result.is_err());
        // Further assertions based on expected error
        // ...
    }

    #[test]
    fn test_handle_invalid_remaining_lenght() {
        let mut buffer: [u8; 8192] = [0; 8192];
        // Fill buffer with invalid data
        let packet = [
            0b0001_0000, // CONNECT
            0x05, // Incorrect remaining length
            0x00,
            0x04, // Protocol name length
            b'M',
            b'Q',
            b'T',
            b'T', // Protocol name
            0x04, // Protocol level (4 for MQTT 3.1.1)
            0x02, // Connect flags (Clean session)
            0x00,
            0x3c, // Keep alive (60 seconds)
            0x00,
            0x04, // Client ID length
            b't',
            b'e',
            b's',
            b't', // Client ID
        ];
        buffer[..packet.len()].copy_from_slice(&packet);

        let packet_length = packet.len();
        let socket_addr = "127.0.0.1:12345".parse().unwrap();
        let mut clients = Vec::new();
        let (tx, _rx) = channel();

        let result = handle(buffer, packet_length, socket_addr, &mut clients, tx);

        assert!(result.is_err());
        // Further assertions based on expected error
        // ...
    }

    #[test]
    fn test_handle_invalid_packet_type() {
        let mut buffer: [u8; 8192] = [0; 8192];
        // Fill buffer with invalid data
        let packet = [
            0b0000_0000, // Invalid packet type
            0x0f, // Remaining length
            0x00,
            0x04, // Protocol name length
            b'M',
            b'Q',
            b'T',
            b'T', // Protocol name
            0x04, // Protocol level (4 for MQTT 3.1.1)
            0x02, // Connect flags (Clean session)
            0x00,
            0x3c, // Keep alive (60 seconds)
            0x00,
            0x04, // Client ID length
            b't',
            b'e',
            b's',
            b't', // Client ID
        ];
        buffer[..packet.len()].copy_from_slice(&packet);

        let packet_length = packet.len();
        let socket_addr = "127.0.0.1:12345".parse().unwrap();
        let mut clients = Vec::new();
        let (tx, _rx) = channel();

        let result = handle(buffer, packet_length, socket_addr, &mut clients, tx);

        assert!(result.is_err());
        // Further assertions based on expected error
        // ...
    }

    #[test]
    fn test_handle_reserved_flag() {
        let mut buffer: [u8; 8192] = [0; 8192];
        let socket_addr = "127.0.0.1:12345".parse().unwrap();
        let mut clients = Vec::new();
        let (tx, _rx) = channel();

        let packet = [
            0b0001_0000, // CONNECT
            0x0e, // Remaining length
            0x00,
            0x04, // Protocol name length
            b'M',
            b'Q',
            b'T',
            b'T', // Protocol name
            0x04, // Protocol level (4 for MQTT 3.1.1)
            0b0000_0001, // Connect flags with Reserved flag set
            0x00,
            0x3c, // Keep alive (60 seconds)
            0x00,
            0x04, // Client ID length
            b't',
            b'e',
            b's',
            b't', // Client ID
        ];

        buffer[..packet.len()].copy_from_slice(&packet);

        let packet_length = packet.len();
        let result = handle(buffer, packet_length, socket_addr, &mut clients, tx.clone());

        assert!(result.is_err(), "Expected error for Reserved flag");
    }
}
