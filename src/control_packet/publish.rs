use crate::models::{client::Client, topicfilter::Topicfilter};

pub fn publish(clients: &mut Vec<Client>, topic_name: &str, topic_message: &str){

    for client in clients.iter(){

        // Converts the topic message to bytes
        let topic_message_bytes: Vec<u8> = topic_message.as_bytes().to_vec();
        
        // Converts the topic name to bytes
        let topic_name_bytes: Vec<u8> = topic_name.as_bytes().to_vec();

        // Checks if there are any clients that subscribs on the given topic name
        if client.subscriptions.contains(&Topicfilter{topic_name: topic_name.to_string(), qos: 0}){

            // Packet to send the client
            let mut packet: Vec<u8> = Vec::new();

            // Packet Type Publish: 3 + Publish flags (Needs to be set!) TODO: get publish flags
            packet.push(48);


            // Remaning packet lenght
            packet.push(u8::try_from(2 + topic_name_bytes.len() + topic_message_bytes.len() + 2).unwrap());

            // LSB and MSB for topic name
            packet.append(u16::try_from(topic_name_bytes.len()).unwrap().to_be_bytes().to_vec().as_mut());

            // Puts all topic name bytes in the packet
            for byte in topic_name_bytes{
                packet.push(byte);
            }

            // LSB and MSB for topic message
            packet.append(u16::try_from(10).unwrap().to_be_bytes().to_vec().as_mut());

            // Puts all topic message bytes in the packet
            for byte in topic_message_bytes{
                packet.push(byte);
            }

            // Sends the packet to the client
            let _ = client.tx.send(packet);
        }
    }

}