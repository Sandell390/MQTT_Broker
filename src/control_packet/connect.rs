use crate::common_fn;

pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> Result<[u8; 4], &'static str> {
    println!("MQTT Connection is being validated");

    // Validate packet
    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    let mut has_valid_protocol_length_and_name = true;
    let mut current_index: usize = bytes_read - remaining_length;

    // Control protocol length & name
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
        Ok(response) => {
            // Match conditions
            if response.0 != 4 && response.1 != "MQTT" {
                has_valid_protocol_length_and_name = false;
            }

            current_index = response.2;
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    if !has_valid_protocol_length_and_name {
        return Err("Invalid protocol name");
    }

    // Control protocol level should be 4 (3.1.1)
    if (buffer[current_index] as u8) != 4 {
        return Ok([32, 2, 0, 1]);
    }

    current_index += 1; // Move the pointer 1 byte to the right

    // Control the connect flags
    let byte: u8 = buffer[current_index];

    // Extracting individual flags
    let flag_0: bool = (byte & 0b00000001) != 0; // Reserved, should be 0
    let flag_1: bool = (byte & 0b00000010) != 0; // Clean Session
    let flag_2: bool = (byte & 0b00000100) != 0; // Will Flag
    let flag_3: bool = (byte & 0b00001000) != 0; // QoS 1 (Note: Both can not be true)
    let flag_4: bool = (byte & 0b00010000) != 0; // QoS 2 (Note: Both can not be true)
    let flag_5: bool = (byte & 0b00100000) != 0; // Will Retain
    let flag_6: bool = (byte & 0b01000000) != 0; // Password Flag
    let flag_7: bool = (byte & 0b10000000) != 0; // Username Flag

    // current_index += 1;

    // Assemble return packet

    let connack_byte: [u8; 4] = [32, 2, 0, 0];

    // Return newly assembled return packet
    return Ok(connack_byte);
}
