use crate::{ common_fn, models::sub_info::SubInfo };

pub fn validate(buffer: [u8; 8192], packet_length: usize) -> Result<SubInfo, &'static str> {
    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    if remaining_length < 6 {
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
        Err(err) => println!("Error: {}", err),
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
            println!("{}", err);
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
