use crate::common_fn;

pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> Result<[u8; 4], &'static str> {
    println!("MQTT Connection is being validated");

    // Validate packet

    let mut remaining_length: usize = 0;

    match common_fn::bit::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    let mut has_valid_protocol_length_and_name = true;
    let mut current_index: usize = bytes_read - remaining_length;
    let start_at_index: usize = bytes_read - remaining_length;
    let stop_at_index: usize = start_at_index + 6;
    let expected_protocol_length_and_name: [&str; 6] = ["0", "4", "M", "Q", "T", "T"];
    let mut protocol_length_and_name: [String; 6] = [
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
    ];
    let mut iterator: usize = 0;

    // Get protocol length and name, and save in an array
    // stop_at_index == range (Number of bytes to read)
    for i in start_at_index..stop_at_index {
        if i < stop_at_index - 4 {
            protocol_length_and_name[iterator] = (buffer[i] as u8).to_string();
        } else {
            protocol_length_and_name[iterator] = (buffer[i] as u8 as char).to_string();
        }

        iterator += 1;
        current_index += 1;
    }

    for i in 0..protocol_length_and_name.len() {
        if expected_protocol_length_and_name[i] != protocol_length_and_name[i] {
            has_valid_protocol_length_and_name = false;
        }
    }

    if !has_valid_protocol_length_and_name {
        return Err("Invalid protocol name");
    }

    // Control protocol level
    if (buffer[current_index] as u8) != 4 {
        return Ok([32, 2, 0, 1]);
    }

    // current_index += 1;

    // Assemble return packet

    // Return newly assembled return packet

    // Convert received bytes to binary representation and print
    let mut binary_repr: String = String::new();
    for byte in &buffer[..bytes_read] {
        binary_repr.push_str(&format!("{:08b} ", byte));
    }

    println!("{}", binary_repr);

    let connack_byte: [u8; 4] = [32, 2, 0, 0];
    return Ok(connack_byte);
}
