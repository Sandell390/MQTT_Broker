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
                        // Inserts both topic filter and QoS into the Vector
                        topic_qos_pair.push((response.1, splited_byte[1]));
                        qos_vec.push(splited_byte[1]);
                        current_index += 1;
                    }
                    Err(err) => println!("Error: {}", err),
                }
            }
            Err(err) => {
                println!("{}", err);
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

// Assembles the SUBACK packet
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
