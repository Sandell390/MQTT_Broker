use crate::{ common_fn, models::sub_info::SubInfo };
use crate::models::text_formatter:: { Color, Style, Reset };

/// Handles the unsubscribe packet.
///
/// # Arguments
///
/// * `buffer` - A fixed-size buffer containing the unsubscribe packet data.
/// * `packet_length` - The length of the packet in bytes.
///
/// # Returns
///
/// * `Result<SubInfo, &'static str>` - A result containing subscription information if successful,
///   or an error message if the handling fails.
///
/// # Description
///
/// This function handles the unsubscribe packet in an MQTT communication. It extracts information from
/// the unsubscribe packet, including the packet ID and the list of topics to unsubscribe from. It then
/// assembles the corresponding unsuback packet. The extracted subscription information is returned along
/// with the assembled unsuback packet. If there is an error during handling, an error message is returned.
///
/// # Examples
///
/// ```
/// let buffer: [u8; 8192] = [0x82, 0x09, 0x00, 0x42, 0x01, 0x61, 0x01, 0x62, 0x01, 0x63];
/// let packet_length: usize = 9;
///
/// match handle_unsubscribe(buffer, packet_length) {
///     Ok(sub_info) => {
///         println!("Unsubscribe Packet ID: {}", sub_info.packet_id);
///         println!("Unsubscribed Topics: {:?}", sub_info.topic_qos_pair);
///         println!("Unsuback Packet: {:?}", sub_info.return_packet);
///     }
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
pub fn handle(buffer: &[u8], packet_length: usize) -> Result<SubInfo, &'static str> {
    let mut remaining_length: usize = 0;

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

    if packet_length < 6 {
        return Err("Subscribe Packet does not have the required lenght");
    }

    if packet_length < remaining_length {
        return Err("Packet lenght is lower than remaining lenght");

    }

    let mut current_index: usize = packet_length - remaining_length;

    // Test if the first byte have bit 1 is on
    match common_fn::bit_operations::split_byte(&buffer[0], 4) {
        Ok(value) => {
            if value[1] != 2 {
                return Err("The first byte have bit 1 is on");
            }
        }
        Err(err) => println!("{1}Error! -> {2}{3}{0}{4}",
                        err,
                        Color::BrightRed,
                        Reset::All,
                        Style::Italic,
                        Reset::All
                    ),
    }

    let mut packet_id: u16 = 0;

    // Get Packet ID
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, false) {
        Ok(response) => {
            // Match conditions
            packet_id = u16::try_from(response.0).ok().unwrap();

            current_index = response.2;

            //println!("Packet ID: {}", packet_id);
        }
        Err(err) => {
            println!("{1}Error! -> {2}{3}{0}{4}",
                err,
                Color::BrightRed,
                Reset::All,
                Style::Italic,
                Reset::All
            );
        }
    }

    // Holds topics
    let mut topics: Vec<(String, u8)> = Vec::new();

    // Get all topic filters
    while current_index <= remaining_length {
        // Find topic filter
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                // Puts the current index to after read string
                current_index = response.2;

                topics.push((response.1, 0));
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    // Assembles the unsuback packet
    let mut unsuback_packet: Vec<u8> = vec![176, 2];
    unsuback_packet.push(u16::to_be_bytes(packet_id)[0]);
    unsuback_packet.push(u16::to_be_bytes(packet_id)[1]);

    Ok(SubInfo { packet_id, topic_qos_pair: topics, return_packet: unsuback_packet })
}
