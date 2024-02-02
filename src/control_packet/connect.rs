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
    let stop_at_index: usize = current_index + 6;
    let expected_protocol_length_and_name: [&str; 6] = ["0", "4", "M", "Q", "T", "T"];
    let mut protocol_length_and_name: [String; 6] = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    let mut iterator: usize = 0;

    // Get protocol length and name, and save in an array
    // stop_at_index == range (Number of bytes to read)
    for i in current_index..stop_at_index {
        if i < stop_at_index - 4 {
            protocol_length_and_name[iterator] = (buffer[i] as u8).to_string();
        } else {
            protocol_length_and_name[iterator] = String::from(buffer[i] as u8 as char);
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

    // Control protocol level should be 4 (3.1.1)
    if (buffer[current_index] as u8) != 4 {
        return Ok([32, 2, 0, 1]);
    }

    /*fn main() {
        let byte: u8 = 0b01101001;
    
        // Extracting individual flags
        let flag_0 = (byte & 0b00000001) != 0;
        let flag_1 = (byte & 0b00000010) != 0;
        let flag_2 = (byte & 0b00000100) != 0;
        let flag_3 = (byte & 0b00001000) != 0;
        let flag_4 = (byte & 0b00010000) != 0;
        let flag_5 = (byte & 0b00100000) != 0;
        let flag_6 = (byte & 0b01000000) != 0;
        let flag_7 = (byte & 0b10000000) != 0;
    
        // Printing the flags
        println!("Flag 0: {}", flag_0);
        println!("Flag 1: {}", flag_1);
        println!("Flag 2: {}", flag_2);
        println!("Flag 3: {}", flag_3);
        println!("Flag 4: {}", flag_4);
        println!("Flag 5: {}", flag_5);
        println!("Flag 6: {}", flag_6);
        println!("Flag 7: {}", flag_7);
    }*/

    // current_index += 1;

    // Assemble return packet

    // Convert received bytes to binary representation and print
    // let mut binary_repr: String = String::new();
    // for byte in &buffer[..bytes_read] {
    //     binary_repr.push_str(&format!("{:08b} ", byte));
    // }

    // println!("{}", binary_repr);

    let connack_byte: [u8; 4] = [32, 2, 0, 0];

    // Return newly assembled return packet
    return Ok(connack_byte);
}
