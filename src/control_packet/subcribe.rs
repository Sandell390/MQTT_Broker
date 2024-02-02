use crate::common_fn;

pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> Result<(u8, Vec<String>), &'static str>{


    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    // Test if the first byte have second bit is on
    match common_fn::bit_operations::split_byte(&buffer[0], 4) {
        Ok(value) => {

            if value[1] != 2{
                return Err("The first byte have second bit is on");
            }
        }
        Err(err) => println!("Error: {}",err ),
    }

    // Get Packet ID
    







    // Get all topic filters


    return Ok((0,Vec::new()));
}