/// Decodes the Remaining Length field from a packet of bytes according to the MQTT protocol.
///
/// # Arguments
///
/// * `bytes` - A mutable reference to a slice of bytes representing the remaining length field.
///
/// # Returns
///
/// A Result containing the decoded remaining length as usize or an error message.
///
/// # Description
///
/// This function decodes the Remaining Length field from a packet of bytes according to the MQTT protocol.
/// The remaining length field is represented by one or more bytes with each byte using the least
/// significant seven bits to represent the data, and the most significant bit indicating whether there
/// are more bytes to read. The function reads bytes from the provided slice and calculates the value
/// of the remaining length field accordingly.
///
/// # Errors
///
/// Returns an error if the remaining length field is malformed or if there are unexpected end of packet.
///
/// # Examples
///
/// ```
/// let buffer: [u8; 8192]; // Read from a tcp stream
///
/// match common_fn::bit_operations::decode_remaining_length(&buffer) {
///         Ok(value) => {
///             remaining_length = value;
///         }
///         Err(err) => println!("Error: {}", err),
///     }
/// ```
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

/// Splits a byte at the defined split index and returns both parts as an array of u8.
///
/// # Arguments
///
/// * `byte` - A reference to a byte to be split.
/// * `split_index` - The index at which to split the byte (0 to 7).
///
/// # Returns
///
/// A Result containing an array of u8 with two elements representing the split parts of the byte,
/// or an error message if the split index is invalid or parsing fails.
///
/// # Description
///
/// This function splits a byte at the defined split index and returns both parts as an array of u8.
/// It first converts the byte into an 8-bit string representation, then splits the string at the
/// specified index, and finally converts each part back to u8 values.
///
/// The resulting array contains the first and second parts of the split byte.
///
/// # Errors
///
/// Returns an error if the split index is greater than 7 or if parsing fails.
///
/// # Examples
///
/// ```
/// let buffer: [u8; 8192]; // Read from a tcp stream
///
/// // Convert first 4 bits to decimal value
/// let packet_type: u8 = common_fn::bit_operations::split_byte(&buffer[0], 4).expect("")[0];
///
/// // Convert last 4 bits to decimal value
/// let packet_type: u8 = common_fn::bit_operations::split_byte(&buffer[0], 4).expect("")[1];
/// ```
pub fn split_byte(byte: &u8, split_index: usize) -> Result<[u8; 2], &'static str> {
    if split_index > 7 {
        return Err("split_index is not allowed to more than 7");
    }

    if split_index == 0 {
        return Err("split_index is not allowed to be 0");
    }

    // Convert byte to a 8-bit string
    let bits_string: String = format!("{:08b}", byte);

    let first_half: u8;
    let second_half: u8;

    // Split the bit_string in half &
    let string_split: (&str, &str) = bits_string.split_at(split_index);

    //convert the first bits to decimal value
    match u8::from_str_radix(string_split.0, 2) {
        Ok(value) => {
            first_half = value;
        }
        Err(_) => {
            return Err("Could not parse the first half byte to u8");
        }
    }

    //convert the last bits to decimal value
    match u8::from_str_radix(string_split.1, 2) {
        Ok(value) => {
            second_half = value;
        }
        Err(_) => {
            return Err("Could not parse the second half byte to u8");
        }
    }

    Ok([first_half, second_half])
}
