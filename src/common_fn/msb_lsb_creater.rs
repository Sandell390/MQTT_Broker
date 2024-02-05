pub fn create_packet(string_value: &str) -> Result<Vec<u8>, &'static str> {
    // Check if the length of the string_value exceeds the maximum length that can be represented by two bytes (MSB and LSB)
    if string_value.len() > 0xffff {
        return Err("String is too long to create a packet");
    }

    // Get the length of the string_value
    let string_length = string_value.len();

    // Convert string_length to two bytes (MSB and LSB)
    let msb = ((string_length >> 8) & 0xff) as u8;
    let lsb = (string_length & 0xff) as u8;

    // Convert string_value to bytes
    let string_bytes = string_value.as_bytes();

    // Create a byte packet
    let mut byte_packet = Vec::new();

    // Append MSB and LSB to the byte packet
    byte_packet.push(msb);
    byte_packet.push(lsb);

    // Append string bytes to the byte packet
    byte_packet.extend_from_slice(string_bytes);

    Ok(byte_packet)
}
