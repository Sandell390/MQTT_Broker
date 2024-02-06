use crate::models::{ topic::Topic, client::Client };
use crate::common_fn;
use rand::Rng;

//********************************************************,
//  TO-DO: Implement handle for incoming publish packets  |
//********************************************************´

pub fn handle_publish() {
    // Code goes here, I think?
}

pub fn handle_puback() {
    // Code goes here, I think?
}

pub fn handle_pubrec() {
    // Code goes here, I think?
}

pub fn handle_pubrel() {
    // Code goes here, I think?
}

pub fn handle_pubcomp() {
    // Code goes here, I think?
}

/// Publishes a message to clients subscribed to the specified topic.
///
/// # Arguments
///
/// * `topics` - A mutable reference to the vector of topics.
/// * `clients` - A mutable reference to the vector of clients.
/// * `topic_name` - The name of the topic to which the message is published.
/// * `topic_message` - The message to be published.
/// * `dup` - A boolean indicating if the message is a duplicate.
/// * `qos` - The quality of service level of the message.
/// * `retain` - A boolean indicating if the message should be retained by the broker.
///
/// # Description
///
/// This function publishes a message to clients subscribed to the specified topic. It iterates
/// over the list of topics to find the matching topic by name. Then, it iterates over the list
/// of clients to find clients subscribed to the topic. For each subscribed client, it creates
/// and sends a packet containing the message to be published. The packet is constructed based
/// on the specified quality of service level, whether the message is a duplicate, and if it
/// should be retained by the broker.
///
/// # Examples
///
/// ```
/// let mut topics = vec![Topic::new("topic1")];
/// let mut clients = vec![Client::new("client1", "", "", 0, "", "", SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0), tx.clone(), ConnectFlags::default())];
/// let topic_name = "topic1";
/// let topic_message = "message";
/// let dup = false;
/// let qos = 0;
/// let retain = false;
///
/// publish(&mut topics, &mut clients, topic_name, topic_message, &dup, &qos, &retain);
/// ```
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

                    if qos == &1 || qos == &2 {
                        packet.append(
                            u16::try_from(packet_id).unwrap().to_be_bytes().to_vec().as_mut()
                        );
                    }

                    packet.append(&mut topic_message_bytes);

                    packet[1] = u8::try_from(packet.len() - 2).unwrap();

                    println!("Publish: {:?}", packet);

                    let _ = client.tx.send(packet);
                };
            }
        }
    }
}
