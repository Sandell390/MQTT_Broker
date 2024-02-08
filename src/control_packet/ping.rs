use crate::common_fn;

use crate::models::text_formatter:: { Color, Style, Reset};

/// Validates and handles the MQTT packet header by checking reserved bits and remaining length.
///
/// # Arguments
///
/// * `buffer` - The buffer containing the incoming packet data.
/// * `packet_length` - The length of the packet in the buffer.
///
/// # Returns
///
/// * `Ok([208, 0])` if the reserved bits are not set, and no payload is expected.
/// * `Err("Reserved bits MUST not be set")` if the reserved bits in the buffer are set.
/// * `Err("No payload expected here")` if the remaining length is not zero or the packet length is greater than 2.
///
/// # Description
///
/// This function validates and handles the MQTT packet header by checking the reserved bits
/// and remaining length. It first extracts the reserved bits from the buffer and checks if
/// they are set. If the reserved bits are set, it returns an error indicating that the reserved
/// bits must not be set. Otherwise, it decodes the remaining length of the packet and verifies
/// that no payload is expected by ensuring that the remaining length is zero and the packet
/// length is not greater than 2. If both conditions are met, it returns success with the
/// appropriate response; otherwise, it returns an error.
///
/// # Examples
///
/// ```
/// let buffer = [0xe0, 0x00];
/// let packet_length = 2;
///
/// match handle(buffer, packet_length) {
///     Ok(response) => println!("{:?}", response), // Output: [208, 0]
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
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
        Err(err) => println!("{1}Error! -> {2}{3}{0}{4}",
                        err,
                        Color::BrightRed,
                        Reset::All,
                        Style::Italic,
                        Reset::All
                    ),
    }

    if remaining_length != 0 || packet_length > 2 {
        return Err("No payload expected here");
    }

    return Ok([208, 0]);
}
