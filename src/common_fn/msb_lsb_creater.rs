/// Creates a packet from the given string value according to the MQTT protocol.
///
/// # Arguments
///
/// * `string_value` - A reference to the string value to be converted into a packet.
///
/// # Returns
///
/// A Result containing a vector of bytes representing the packet, or an error message if the string
/// value is too long to be represented by two bytes (MSB and LSB).
///
/// # Description
///
/// This function creates a packet from the given string value according to the MQTT protocol.
/// It first checks if the length of the string value exceeds the maximum length that can be represented
/// by two bytes (MSB and LSB).
///
/// If the length is within the allowed range, it calculates the length
/// in bytes, converts it to two bytes (MSB and LSB), converts the string value to bytes, and creates
/// a byte packet by appending the MSB, LSB, and string bytes.
///
/// # Errors
///
/// Returns an error if the length of the string value exceeds the maximum allowed length.
///
/// # Examples
///
/// ```
/// let string: &str = "MQTT";
/// let msb_lsb_packet = common_fn::msb_lsb_creater::create_packet(string).unwrap();
/// ```
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

/// Converts a usize value, to a 2 byte array, containing only MSB & LSB.
///
/// # Arguments
///
/// * `value` - The decimal value, as a uszie to convert.
///
/// # Returns
///
/// A 2 byte array, containing the MSB & LSB.
///
/// # Examples
///
/// ```
/// let usize_value: usize = 65;
/// let msb_lsb_array = common_fn::msb_lsb_creater::split_into_msb_lsb(string);
/// ```
pub fn split_into_msb_lsb(value: usize) -> [u8; 2] {
    let msb = ((value >> 8) & 0xFF) as u8;
    let lsb: u8 = (value & 0xFF) as u8;
    [msb, lsb]
}
