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
