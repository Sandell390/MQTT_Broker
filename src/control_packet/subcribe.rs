use crate::{ common_fn, models::sub_info::SubInfo };
use crate::models::text_formatter:: { Color, Style, Reset };

/// Handles the Subscribe packet received from the client.
///
/// # Arguments
///
/// * `buffer` - The buffer containing the packet data.
/// * `packet_length` - The length of the packet.
///
/// # Returns
///
/// * `Result<SubInfo, &'static str>` - A result indicating success with subscription information
///   or an error message if the packet is invalid.
///
/// # Description
///
/// This function handles the Subscribe packet received from the client. It decodes the packet,
/// validates its structure, retrieves the packet ID, extracts the topic filters and associated
/// quality of service (QoS) levels, and constructs a SubInfo struct containing the subscription
/// information. If the packet is invalid, an error message is returned.
///
/// # Examples
///
/// ```
/// let buffer: [u8; 8192] = [0; 8192];
/// let packet_length = 10;
///
/// match handle(buffer, packet_length) {
///     Ok(sub_info) => {
///         println!("Packet ID: {}", sub_info.packet_id);
///         println!("Subscription Information: {:?}", sub_info.topic_qos_pair);
///         println!("Suback Packet: {:?}", sub_info.return_packet);
///     }
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
pub fn handle(buffer: &[u8], packet_length: usize) -> Result<SubInfo, &'static str> {
    let mut remaining_length: usize = 0;

    // Gets the remaining from the packet
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

    // Holds the toptic filters and the associated QoS
    let mut topic_qos_pair: Vec<(String, u8)> = Vec::new();

    // Only to hold the qos so it can be used to suback packet
    let mut qos_vec: Vec<u8> = Vec::new();

    // Get all topic filters
    while current_index <= remaining_length {
        // Find topic filter
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                // Puts the current index to after read string
                current_index = response.2;

                // Gets the QoS to the topic filter
                match common_fn::bit_operations::split_byte(&buffer[current_index], 6) {
                    Ok(splited_byte) => {

                        if splited_byte[1] >= 3 {
                            topic_qos_pair.push((response.1, 0x80));
                            qos_vec.push(0x80);
                        }else{
                        // Inserts both topic filter and QoS into the Vector
                        topic_qos_pair.push((response.1, splited_byte[1]));
                        qos_vec.push(splited_byte[1]);
                        }

                        current_index += 1;
                    }
                    Err(err) => println!("{1}Error! -> {2}{3}{0}{4}",
                                    err,
                                    Color::BrightRed,
                                    Reset::All,
                                    Style::Italic,
                                    Reset::All
                                ),
                }
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
    }

    // Makes the suback_packet
    let suback_packet: Vec<u8> = assemble_suback_packet(
        qos_vec.as_slice(),
        u16::to_be_bytes(packet_id)
    )?;

    return Ok(SubInfo {
        packet_id,
        topic_qos_pair,
        return_packet: suback_packet,
    });
}

/// Assembles the SUBACK packet.
///
/// # Arguments
///
/// * `qos_arr` - An array slice containing the quality of service (QoS) levels.
/// * `packet_id` - The packet ID as a 2-byte array.
///
/// # Returns
///
/// * `Result<Vec<u8>, &'static str>` - A result containing the assembled SUBACK packet if successful,
///   or an error message if the assembly fails.
///
/// # Description
///
/// This function assembles the SUBACK packet used in MQTT communication. It takes an array slice of QoS levels
/// and the packet ID, and constructs the SUBACK packet according to the MQTT protocol. The assembled packet
/// is returned as a vector of bytes. If there is an error during assembly, an error message is returned.
///
/// # Examples
///
/// ```
/// let qos_arr = &[0, 1, 2];
/// let packet_id: [u8; 2] = [0x12, 0x34];
///
/// match assemble_suback_packet(qos_arr, packet_id) {
///     Ok(suback_packet) => {
///         println!("Assembled SUBACK packet: {:?}", suback_packet);
///     }
///     Err(err) => println!("Error: {}", err),
/// }
/// ```
pub fn assemble_suback_packet(qos_arr: &[u8], packet_id: [u8; 2]) -> Result<Vec<u8>, &'static str> {
    // The length of the SUBACK Packet
    let packet_lenght: u8 = 2 + u8::try_from(qos_arr.len()).ok().unwrap();

    // Return packet
    let mut packet: Vec<u8> = Vec::new();

    // SUBACK control Type: 9
    packet.push(144);

    // Remaining length
    packet.push(packet_lenght);

    // Packet ID
    packet.push(packet_id[0]);
    packet.push(packet_id[1]);

    // Puts all the QoS levels in the packet
    for i in qos_arr.iter() {
        packet.push(*i);
    }

    Ok(packet)
}
