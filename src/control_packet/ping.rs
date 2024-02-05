use crate::common_fn;

pub fn handle(buffer: [u8; 8192], packet_length: usize) -> Result<[u8; 2], &'static str> {
    // Convert 4 last bits to decimal value
    let reserved_bits: u8 = common_fn::bit_operations::split_byte(&buffer[0], 4).expect("")[1];
    let mut remaining_length: usize = 0;

    if reserved_bits != 0 {
        return Err("Reserved bits MUST not be set");
    }

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    if remaining_length != 0 {
        return Err("No payload expected here");
    }

    return Ok([13, 0]);
}
