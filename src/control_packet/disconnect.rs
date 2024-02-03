use crate::common_fn;

pub fn validate(buffer: [u8; 8192], packet_length: usize) -> Result<&'static str, &'static str> {
    println!("MQTT Disconnection is being validated");

    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    let current_index: usize = packet_length - remaining_length;

    // Reserved bits MUST be 0
    let mut is_reserved_bits_set: bool = false;

    if buffer[current_index] != 0 {
        is_reserved_bits_set = true;
    }

    match is_reserved_bits_set {
        true => {
            return Err("Reserved bits are set");
        }
        false => {
            return Ok("Reserved bits not set");
        }
    }
}
