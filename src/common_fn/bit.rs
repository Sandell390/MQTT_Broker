// Function to decode the Remaining Length field from a packet of bytes
pub fn decode_remaining_length(mut bytes: &[u8]) -> Result<usize, &'static str> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;

    bytes = &bytes[1..]; // Skip first Byte, as this is the control packet / packet type

    loop {
        // Check if there are no more bytes in the packet
        if bytes.is_empty() {
            return Err("Unexpected end of packet");
        }

        let encoded_byte: u8 = bytes[0];

        bytes = &bytes[1..]; // Move to the next byte in the packet

        value += ((encoded_byte & 127) as u32) * multiplier;

        if multiplier > 128 * 128 * 128 {
            return Err("Malformed Remaining Length");
        }

        multiplier *= 128;

        if (encoded_byte & 128) == 0 {
            break; // Exit the loop when continuation bit is not set
        }
    }

    Ok(value.try_into().unwrap())
}
