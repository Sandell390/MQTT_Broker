// Function to decode the Remaining Length field from a packet of bytes
pub fn decode_remaining_length(mut bytes: &[u8]) -> Result<usize, &'static str> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;

    bytes = &bytes[1..]; // Skip first Byte, as this is the control packet / packet type

    loop {
        // Check if there are no more bytes in the packet
        if bytes.is_empty() {
            return Err("Unexpected end of packet");
        }

        let encoded_byte: u8 = bytes[0];

        bytes = &bytes[1..]; // Move to the next byte in the packet

        value += ((encoded_byte & 127) as u32) * multiplier;

        if multiplier > 128 * 128 * 128 {
            return Err("Malformed Remaining Length");
        }

        multiplier *= 128;

        if (encoded_byte & 128) == 0 {
            break; // Exit the loop when continuation bit is not set
        }
    }

    Ok(value.try_into().unwrap())
}


// Function to split a byte with a defined split index and return both parts as u8
pub fn split_byte(byte: &u8, split_index: usize) -> Result<[u8; 2], &'static str>{

    if split_index > 7 {
        return Err("split_index is not allowed to more than 7");
    }

    // Convert byte to a 8-bit string
    let bits_string: String = format!("{:08b}", byte);

    let mut first_half: u8 = 0;
    let mut second_half: u8 = 0;

    // Split the bit_string in half & 
    let string_split: (&str, &str) = bits_string.split_at(split_index);

    //convert the first bits to decimal value
    match u8
        ::from_str_radix(string_split.0, 2){
            Ok(value) => {
                first_half = value;
            }
            Err(_) => { return Err("Could not parse the first half byte to u8")}
        };
    
    //convert the last bits to decimal value
    match u8
        ::from_str_radix(string_split.1, 2){
            Ok(value) => {
                second_half = value;
            }
            Err(_) => {return Err("Could not parse the second half byte to u8")}
        };



    Ok([first_half,second_half])
}
