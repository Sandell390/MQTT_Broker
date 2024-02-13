use crate::common_fn;

use crate::models::text_formatter::{ Color, Style, Reset };
/// Validates and handles MQTT disconnection by checking the reserved bits in the buffer.
///
/// # Arguments
///
/// * `buffer` - The buffer containing the incoming packet data.
/// * `packet_length` - The length of the packet in the buffer.
///
/// # Returns
///
/// * `Ok("Reserved bits not set")` if the reserved bits in the buffer are not set.
/// * `Err("Reserved bits are set")` if the reserved bits in the buffer are set.
///
/// # Description
///
/// This function validates and handles MQTT disconnection by checking the reserved bits
/// in the buffer. It first decodes the remaining length of the packet and calculates the
/// current index based on the packet length and remaining length. Then, it checks whether
/// the reserved bits in the buffer are set. If the reserved bits are set, it returns an
/// error indicating that the reserved bits are set; otherwise, it returns success.
///
/// # Examples
///
/// ```
/// let buffer: [u8; 8192]; // Read from a tcp stream
///
/// // Validate reserved bits are not set
/// match control_packet::disconnect::handle(buffer, packet_length) {
///     Ok(_response) => {
///         disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, true);
///     }
///     Err(_err) => {
///         disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, true);
///     }
/// }
/// ```
pub fn handle(buffer: &[u8], packet_length: usize) -> Result<&'static str, &'static str> {
    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) =>
            println!(
                "{1}Error! -> {2}{3}{0}{4}",
                err,
                Color::BrightRed,
                Reset::All,
                Style::Italic,
                Reset::All
            ),
    }

    let _current_index: usize = packet_length - remaining_length;

    // Reserved bits MUST be 0
    let mut is_reserved_bits_set: bool = false;

    if buffer[1] != 0 {
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
