use crate::models::{ topic::Topic, client::Client };
use crate::common_fn;
use rand::Rng;

pub fn publish(
    topics: &mut Vec<Topic>,
    clients: &mut Vec<Client>,
    topic_name: &str,
    topic_message: &str,
    dup: &bool,
    qos: &u8,
    retain: &bool
) {
    for topic in topics.iter() {
        if topic_name == topic.topic_name {
            for client in clients.iter() {
                if
                    let Some(_client_index) = topic.client_ids
                        .iter()
                        .position(|c: &(String, u8)| &c.0 == &client.id)
                {
                    let mut packet: Vec<u8> = Vec::new();

                    let mut first_byte: u8 = 0b0011_0000;

                    if *dup {
                        first_byte |= 1 << 3;
                    }

                    if *qos == 2 {
                        first_byte |= 1 << 2;
                    }

                    if *qos == 1 {
                        first_byte |= 1 << 1;
                    }

                    if *retain {
                        first_byte |= 1 << 0;
                    }

                    let mut topic_name_bytes = common_fn::msb_lsb_creater
                        ::create_packet(topic_name)
                        .unwrap();

                    let packet_id: u16 = rand::thread_rng().gen_range(1..=u16::MAX);

                    let mut topic_message_bytes = topic_message.as_bytes().to_vec();

                    packet.push(first_byte);

                    // Sets the remaining length later
                    packet.push(0);
                    packet.append(&mut topic_name_bytes);

                    if qos == &1 || qos == &2{
                        packet.append(
                            u16::try_from(packet_id).unwrap().to_be_bytes().to_vec().as_mut()
                        );
    
                    }

                    packet.append(&mut topic_message_bytes);

                
                    packet[1] = u8
                                     ::try_from(packet.len() - 2)
                                   .unwrap();

                    println!("Publish: {:?}", packet);

                    let _ = client.tx.send(packet);
                };
            }
        }
    }
}

// for client in clients.iter() {
//     // Converts the topic message to bytes
//     let topic_message_bytes: Vec<u8> = topic_message.as_bytes().to_vec();

//     // Converts the topic name to bytes
//     let topic_name_bytes: Vec<u8> = topic_name.as_bytes().to_vec();

//     // Checks if there are any clients that subscribs on the given topic name
//     if true {
//         // Packet to send the client
//         let mut packet: Vec<u8> = Vec::new();

//         // Packet Type Publish: 3 + Publish flags (Needs to be set!) TODO: get publish flags
//         packet.push(48);

//         // Remaning packet lenght
//         packet.push(
//             u8
//                 ::try_from(2 + topic_name_bytes.len() + 2 + topic_message_bytes.len() + 2)
//                 .unwrap()
//         );

//         // LSB and MSB for topic name
//         packet.append(
//             u16::try_from(topic_name_bytes.len()).unwrap().to_be_bytes().to_vec().as_mut()
//         );

//         // Puts all topic name bytes in the packet
//         for byte in topic_name_bytes {
//             packet.push(byte);
//         }

//         // Packet ID
//         packet.append(u16::try_from(10).unwrap().to_be_bytes().to_vec().as_mut());

//         // LSB and MSB for topic message
//         packet.append(
//             u16::try_from(topic_message_bytes.len()).unwrap().to_be_bytes().to_vec().as_mut()
//         );

//         // Puts all topic message bytes in the packet
//         for byte in topic_message_bytes {
//             packet.push(byte);
//         }

//         // Sends the packet to the client
//         println!("Publish packet: {:?}", packet);
//         let _ = client.tx.send(packet);
//     }
// }
// }
