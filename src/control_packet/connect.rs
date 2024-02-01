pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> [u8; 4] {
    println!("MQTT Connection is being validated");

    // Validate packet

    // Assemble return packet

    // Return newly assembled return packet

    // Convert received bytes to binary representation and print
    let mut binary_repr: String = String::new();
    for byte in &buffer[..bytes_read] {
        binary_repr.push_str(&format!("{:08b} ", byte));
    }

    println!("{}", binary_repr);

    let connack_byte: [u8; 4] = [32, 2, 0, 0];
    return connack_byte;
}
