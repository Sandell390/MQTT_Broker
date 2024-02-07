use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use crate::common_fn;
use crate::models::publish_queue_item::{self, PublishItemDirection, PublishItemState};
use crate::models::{client::Client, publish_queue_item::PublishQueueItem, topic::Topic};
use rand::Rng;

#[derive(Clone)]
pub struct Response {
    pub dup_flag: bool,
    pub qos_level: u8,
    pub retain_flag: bool,
    pub packet_id: usize,
    pub topic_name: String,
    pub payload_message: String,
}

pub fn handle_publish(buffer: [u8; 8192], packet_length: usize) -> Result<Response, &'static str> {
    println!("Handling publish packet from client");

    // Check if each bit is set
    let flag_3: bool = (&buffer[0] & (1 << 3)) != 0; // DUP Flag
    let flag_2: bool = (&buffer[0] & (1 << 2)) != 0; // QoS 2 Flag
    let flag_1: bool = (&buffer[0] & (1 << 1)) != 0; // QoS 1 Flag
    let flag_0: bool = (&buffer[0] & (1 << 0)) != 0; // Retain Flag

    let mut qos_level: u8 = 0;

    // QoS 1 flag
    if flag_1 {
        qos_level = 1;
    }

    // QoS 2 flag
    if flag_2 {
        qos_level = 2;
    }

    if flag_1 && flag_2 {
        return Err("Wrong QoS level specified");
    }

    let mut remaining_length: usize = 0;

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    let mut current_index: usize = packet_length - remaining_length;
    let mut topic_name: String = String::new();
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
        Ok(response) => {
            // Get the topic name
            topic_name = response.1;

            // Update current index
            current_index = response.2;
            print!("Topic_name: {topic_name} |");
            println!("Current index: {current_index}");
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    let mut packet_id: usize = 0;

    if qos_level != 0 {
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, false) {
            Ok(response) => {
                // Get the packet identifier
                packet_id = response.0;
    
                // Update current index
                current_index = response.2;
    
                print!("Packet_id: {packet_id} |");
                println!("Current index: {current_index}");
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }


    let mut payload_message: String = String::new();

    while current_index < packet_length {
        payload_message.push(buffer[current_index].clone() as u8 as char);
        current_index += 1;

        
    }

    
    print!("payload_message: {payload_message} |");
    println!("Current index: {current_index}");

    // Assemble return struct
    let response: Response = Response {
        dup_flag: flag_3,
        qos_level,
        retain_flag: flag_0,
        packet_id,
        topic_name,
        payload_message,
    };

    return Ok(response);
}

pub fn handle_puback(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {

    if packet_length != 4{
        return Err("Invalid packet length");
    }

    if buffer[1] != 2{
        return Err("Invalid remaining length");
    }

    let packet_id: usize = common_fn::msb_lsb_reader::get_values(&buffer, 2, false)?.0;

    Ok(packet_id)
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
    publish_queue: Arc<Mutex<Vec<PublishQueueItem>>>,
    topic_name: &str,
    topic_message: &str,
    dup: &bool,
    qos: &u8,
    retain: &bool,
) {
    for topic in topics.iter() {
        if topic_name == topic.topic_name {
            for client in clients.iter() {
                if let Some(_client_index) = topic
                    .client_ids
                    .iter()
                    .position(|c: &(String, u8)| &c.0 == &client.id)
                {

                    println!("hej");

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

                    let mut topic_name_bytes =
                        common_fn::msb_lsb_creater::create_packet(topic_name).unwrap();

                    let packet_id: usize = rand::thread_rng().gen_range(1..=65535);

                    let mut topic_message_bytes = topic_message.as_bytes().to_vec();

                    packet.push(first_byte);

                    // Sets the remaining length later
                    packet.push(0);

                    packet.append(&mut topic_name_bytes);

                    if qos == &1 || qos == &2 {
                        packet.append(
                            common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id)
                                .to_vec()
                                .as_mut(),
                        );
                    }

                    packet.append(&mut topic_message_bytes);

                    
                    
                    packet[1] = u8::try_from(packet.len() - 2).unwrap();

                    println!("Publish: {:?}", packet);

                    let _ = client.tx.send(Ok(packet.clone()));

                    if *qos == 1 {
                        let (tx, rx): (Sender<PublishItemState>, Receiver<PublishItemState>) =
                            channel();

                        let publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>> =
                            Arc::clone(&publish_queue);

                        thread::spawn(move || {

                            {
                                let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> =
                                publish_queue_clone.lock().unwrap();
    
                                publish_queue.push(PublishQueueItem {
                                    tx,
                                    packet_id,
                                    timestamp_sent: Instant::now(),
                                    publish_packet: packet,
                                    state: PublishItemState::AwaitingPuback,
                                    qos_level: 1,
                                    flow_direction: PublishItemDirection::ToSubscriber,
                                });
                            }
                            

                            for i in 0..222 {
                                match rx.try_recv() {
                                    Ok(state) => {
                                        if state == PublishItemState::PubackRecieved {
                                            let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> =
                                publish_queue_clone.lock().unwrap();
                                            if let Some(index) = publish_queue.iter().position(
                                                |t: &PublishQueueItem| t.packet_id == packet_id,
                                            ) {
                                                publish_queue.remove(index);
                                            }

                                            println!("QoS 1 Complete!");
                                            break;
                                        }
                                    }
                                    Err(_) => {}
                                }

                                thread::sleep(Duration::from_secs(1));
                            }

                            let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> =
                                publish_queue_clone.lock().unwrap();
                            if let Some(index) = publish_queue
                                .iter()
                                .position(|t: &PublishQueueItem| t.packet_id == packet_id)
                            {
                                publish_queue.remove(index);
                            }
                        });
                    }
                };
            }
        }
    }
}
