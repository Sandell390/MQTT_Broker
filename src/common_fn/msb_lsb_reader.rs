/// Retrieves values from the buffer, by decoding the MSB & LSB bytes, and reading the rest as UTF8 Encoded Strings.
///
/// # Arguments
///
/// * `buffer` - A reference to a byte slice containing the data to read.
/// * `current_index` - The current index within the buffer to start reading from.
/// * `read_string_value` - A boolean indicating whether to read a string value from the buffer.
///
/// # Returns
///
/// A Result containing a tuple with the following elements:
/// - The decimal value of MSB and LSB.
/// - The string value read from the buffer, if `read_string_value` is true.
/// - The stop index indicating the end of the read values.
///
/// # Errors
///
/// Returns an error if the buffer is too small to read the MSB and LSB or the string value.
///
/// # Description
///
/// This function retrieves values from the buffer according to the provided specifications.
/// It first checks if the buffer has enough bytes to read the MSB (most significant byte) and LSB (least significant byte).
/// Then, it calculates the decimal value from the MSB and LSB, representing the length of the subsequent string value.
/// If `read_string_value` is true, it checks if the buffer has enough bytes to read the string value and proceeds to
/// read the string value character by character. Finally, it returns a tuple containing the decimal value, the string value
/// (if read), and the stop index indicating the end of the read values.
///
/// # Examples
///
/// ```
/// let buffer = &[0x00, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]; // Byte slice with data
/// let (updated_index, string_value, stop_index) = get_values(buffer, 0, true).unwrap();
/// assert_eq!(updated_index, 7);
/// assert_eq!(string_value, "Hello");
/// assert_eq!(stop_index, 7);
/// ```
pub fn get_values(
    buffer: &[u8],
    mut current_index: usize,
    read_string_value: bool
) -> Result<(usize, String, usize), &'static str> {
    // Check if the buffer has enough bytes to read the MSB and LSB
    if current_index + 2 > buffer.len() {
        return Err("Buffer is too small to read MSB and LSB");
    }

    let msb: usize = buffer[current_index] as usize;
    let lsb: usize = buffer[current_index + 1] as usize;

    current_index += 2; // Move the pointer 2 bytes to the right, ready for a string to appear

    let decimal_value: usize = (msb << 8) + lsb;

    // Check if read_string_value is true and if the buffer has enough bytes to read the string value
    if read_string_value && current_index + decimal_value > buffer.len() {
        return Err("Buffer is too small to read the string value");
    }

    if read_string_value {
        let stop_at_index = current_index + decimal_value;
        let mut string_value: String = String::new();

        // Get length and value (Chars), and save in an array
        // stop_at_index == range (Number of bytes to read)
        for i in current_index..stop_at_index {
            string_value.push(buffer[i] as u8 as char);
        }

        return Ok((decimal_value, string_value, stop_at_index));
    }

    Ok((decimal_value, String::new(), current_index))
}
