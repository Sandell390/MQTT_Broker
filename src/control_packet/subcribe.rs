use crate::common_fn;
use std::{collections::HashMap, convert::TryFrom};

pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> Result<([u8; 2], HashMap<String, u8>), &'static str>{


    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }


    if remaining_length < 6{
        return Err("Subscribe Packet does not have the required lenght");
    }

    let mut current_index: usize = bytes_read - remaining_length;


    // Test if the first byte have bit 1 is on
    match common_fn::bit_operations::split_byte(&buffer[0], 4) {
        Ok(value) => {

            if value[1] != 2{
                return Err("The first byte have bit 1 is on");
            }
        }
        Err(err) => println!("Error: {}",err ),
    }

    let mut packet_id: [u8; 2] = [0,0];

    // Get Packet ID
    packet_id[0] = buffer[current_index];
    current_index += 1;
    packet_id[1] = buffer[current_index];
    current_index += 1;

    /* 
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, false) {
        Ok(response) => {
            // Match conditions
            packet_id = u16::try_from(response.0).ok().unwrap();

            current_index = response.2;

            println!("Packet ID: {}", packet_id);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
    */
    // Holds the toptic filters and the associated QoS 
    let mut topic_filters: HashMap<String, u8>= HashMap::new();

    // Get all topic filters
    while current_index <= remaining_length {

        // Find topic filter
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
               
                println!("Topic: {}", response.1);

                // Puts the current index to after read string
                current_index = response.2;

                // Gets the QoS to the topic filter
                match common_fn::bit_operations::split_byte(&buffer[current_index], 6) {
                    Ok(value) => {
            
                        // Inserts both topic filter and QoS into the hashmap
                        topic_filters.insert(response.1, value[0]);

                        current_index += 1;
                    }
                    Err(err) => println!("Error: {}",err ),
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }


    return Ok((packet_id,topic_filters));
}

// Assembles the SUBACK packet
pub fn assemble_suback_packet(qos_arr: &[u8], packet_id: &[u8]) -> Result<Vec<u8>,  &'static str> {

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
    for i in qos_arr.iter(){
        packet.push(*i);
    }

    Ok(packet)
}