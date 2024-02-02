pub fn get_values(
    buffer: &[u8],
    mut current_index: usize,
    read_string_value: bool
) -> Result<(usize, String, usize), &'static str> {
    let msb: usize = buffer[current_index] as usize;
    let lsb: usize = buffer[current_index + 1] as usize;

    current_index += 2; // Move the pointer 2 bytes to the right, ready for a string to appear

    let decimal_value: usize = (msb << 8) + lsb;

    if read_string_value {
        let stop_at_index = current_index + decimal_value;
        let mut string_value: String = "".to_owned();
        string_value.push_str("");

        // Get length and value (Chars), and save in an array
        // stop_at_index == range (Number of bytes to read)
        for i in current_index..stop_at_index {
            string_value.push_str(&(buffer[i] as u8 as char).to_string());
            current_index += 1;
        }

        return Ok((decimal_value, string_value, current_index));
    }

    return Ok((decimal_value, String::new(), current_index));
}
