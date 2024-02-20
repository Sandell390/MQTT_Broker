use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use crate::common_fn;
use crate::models::publish_queue_item::{ PublishItemDirection, PublishItemState };
use crate::models::{ client::Client, publish_queue_item::PublishQueueItem, topic::Topic };
use crate::models::text_formatter:: { Color, Style, Reset };
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

/// Handles publish packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns
///
/// A Result containing a [`Response`] struct, or an error message.
///
/// # Errors
///
/// Returns an error if the publish packet is malformed in any way, or doesn't conform to the MQTT specification.
pub fn handle_publish(buffer: [u8; 8192], packet_length: usize) -> Result<Response, &'static str> {
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

    // Get the remaining length from the publish packet
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

    // Throws an error if the packet length is lower than remaining length or equel to
    if packet_length <= remaining_length {
        return Err("Packet length is lower than remaining length");
    }

    // Sets the current index in the buffer
    let mut current_index: usize = packet_length - remaining_length;

    // Gets the topic name
    let mut topic_name: String = String::new();
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
        Ok(response) => {
            // Get the topic name
            topic_name = response.1;

            // Update current index
            current_index = response.2;
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

    let mut packet_id: usize = 0;

    // If the QoS Level is not 0 then there is a packet id in the packet 
    if qos_level != 0 {
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, false) {
            Ok(response) => {
                // Get the packet identifier
                packet_id = response.0;

                // Update current index
                current_index = response.2;
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

    // Gets the payload of the publish packet
    let mut payload_message: String = String::new();
    while current_index < packet_length {
        payload_message.push(buffer[current_index].clone() as u8 as char);
        current_index += 1;
    }

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

/// Handles puback packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if the packet is malformed in any way, or doesn't conform to the MQTT specification.
pub fn handle_puback(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {
    validate_qos_packet(buffer, packet_length)
}

/// Handles pubrec packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if the packet is malformed in any way, or doesn't conform to the MQTT specification.
pub fn handle_pubrec(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {
    validate_qos_packet(buffer, packet_length)
}

/// Handles pubrel packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if the packet is malformed in any way, or doesn't conform to the MQTT specification.
pub fn handle_pubrel(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {
    validate_qos_packet(buffer, packet_length)
}

/// Handles pubcomp packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if the packet is malformed in any way, or doesn't conform to the MQTT specification.
pub fn handle_pubcomp(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {
    validate_qos_packet(buffer, packet_length)
}

/// Validates QoS packets, according to the MQTT protocol.
///
/// # Arguments
///
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if the packet is malformed in any way, or doesn't conform to the MQTT specification.
fn validate_qos_packet(buffer: [u8; 8192], packet_length: usize) -> Result<usize, &'static str> {
    if packet_length != 4 {
        return Err("Invalid packet length");
    }

    if buffer[1] != 2 {
        return Err("Invalid remaining length");
    }

    let packet_id: usize = common_fn::msb_lsb_reader::get_values(&buffer, 2, false)?.0;

    Ok(packet_id)
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
    _dup: &bool,
    _qos: &u8,
    retain: &bool,
) {
    // Loops through the topic list and if the topic name matches
    // Then loops all the clients to get the client id
    // And if that matches with the client id in the topic 
    for topic in topics.iter() {
        if topic_name == topic.topic_name {
            for client in clients.iter() {
                if let Some(client_index) = topic.client_ids.iter().position(|c: &(String, u8)| &c.0 == &client.id)
                {
                    publish_to_client(client, Arc::clone(&publish_queue), topic, topic_message, &topic.client_ids[client_index].1, retain);
                };
            }
        }
    }
}

/// Publish a payload to a client.
/// 
/// # Arguments
///
/// * `client` - A reference to the [`Client`] struct.
/// * `publish_queue` - A clone of the publish queue
/// * `buffer` - A reference to the buffer, passed from the tcpStream Read function.
/// * `packet_length` - The length of the buffer, to consider part of the packet.
///
/// # Returns the packet identifier as a usize
///
/// A Result containing a , or an error message.
///
/// # Errors
///
/// Returns an error if reciever fails.
pub fn publish_to_client(client: &Client, publish_queue: Arc<Mutex<Vec<PublishQueueItem>>>, topic: &Topic, topic_message: &str, qos: &u8,retain: &bool) {
    
    // Publish packet
    let mut packet: Vec<u8> = Vec::new();

    // Sets the first half of the first byte (control type) to publish
    let mut first_byte: u8 = 0b0011_0000;

    if *qos == 2 {
        first_byte |= 1 << 2;
    } else if *qos == 1 {
        first_byte |= 1 << 1;
    }

    // Sets the retain flag, according to the passed parameter [`retain`]
    if *retain {
        first_byte |= 1 << 0;
    }

    // Gets the topic name bytes 
    let mut topic_name_bytes = common_fn::msb_lsb_creater::create_packet(&topic.topic_name).unwrap();

    // Generates a random packet id
    let packet_id: usize = rand::thread_rng().gen_range(1..=65535);

    // Gets the topic message bytes, from the passed parameter
    let mut topic_message_bytes = topic_message.as_bytes().to_vec();

    // Puts the first in the packet
    packet.push(first_byte);

    // Sets the remaining length later
    packet.push(0);

    // Append all the topic name bytes to the packet
    packet.append(&mut topic_name_bytes);

    // If the client have subscribed with QoS 1 or QoS 2 then the publish packet needs to have a packet id
    if *qos == 1 || *qos == 2 {
        packet.append(
            common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id)
                .to_vec()
                .as_mut(),
        );
    }

    // Append the topic_message_bytes to the publish packet
    packet.append(&mut topic_message_bytes);

    // Sets the remaining length minus the fixed header
    packet[1] = u8::try_from(packet.len() - 2).unwrap();

    // Send publish packet to the client
    let _ = client.tx.send(Ok(packet.clone()));

    // If the client have subscribe with QoS 1, then make the QoS 1 flow
    if *qos == 1 {
        
        // Create a new channel, for later passing the tx back to another thread (handle_connection), for inter-thread communication.
        let (tx, rx): (Sender<PublishItemState>, Receiver<PublishItemState>) = channel();

        // Clones the publish queue list
        let publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>> = Arc::clone(&publish_queue);

        // Clone the client
        let client_clone: Client = client.clone();

        // QoS 1 Session thread
        thread::spawn(move || {
            {
                // Unlock the publish_queue, or wait until it is available
                let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> = publish_queue_clone.lock().unwrap();

                // Adds a new publish queue item to the publish queue 
                publish_queue.push(PublishQueueItem {
                    tx,
                    packet_id,
                    timestamp_sent: Instant::now(),
                    publish_packet: packet.clone(),
                    state: PublishItemState::AwaitingPuback,
                    qos_level: 1,
                    flow_direction: PublishItemDirection::ToSubscriber,
                });
            }
            // Wait for 2220 miliseconds, calculated by the max size of a payload (256 mb)
            // Downloaded with a 10Mbps internet connection (205 seconds), and then a little 
            // extra time to handle the packet and get the reply.
            for _i in 0..2220 {
                match rx.try_recv() {
                    Ok(state) => {
                        if state == PublishItemState::PubackRecieved {
                            // Unlock the publish_queue, or wait until it is available
                            let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> = publish_queue_clone.lock().unwrap();

                            // Attempt to find the publish item in the queue, to remove it.
                            if let Some(index) = publish_queue.iter().position(
                                |t: &PublishQueueItem| t.packet_id == packet_id,
                            ) {
                                publish_queue.remove(index);
                            }

                            return;
                        }
                    }
                    Err(_) => {}
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Send the packet to the client
            let _ = client_clone.tx.send(Ok(packet.clone()));

            // Unlock the publish_queue, or wait until it is available
            let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> = publish_queue_clone.lock().unwrap();

            // Attempt to find the publish item in the queue, to remove it.
            if let Some(index) = publish_queue
                .iter()
                .position(|t: &PublishQueueItem| t.packet_id == packet_id)
            {
                publish_queue.remove(index);
            }
        });
    } else if *qos == 2 {
        // Create a new channel, for later passing the tx back to another thread (handle_connection), for inter-thread communication.
        let (tx, rx): (Sender<PublishItemState>, Receiver<PublishItemState>) = channel();

        // Clones the publish queue list
        let publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>> = Arc::clone(&publish_queue);

        // Clone the client
        let client_clone: Client = client.clone();

        // QoS 2 Session thread
        thread::spawn(move || {

            // Adds a new publish queue item to the publish queue 
            {
                let mut publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> = publish_queue_clone.lock().unwrap();

                // Adds a new publish queue item to the publish queue 
                publish_queue.push(PublishQueueItem {
                    tx,
                    packet_id,
                    timestamp_sent: Instant::now(),
                    publish_packet: packet.clone(),
                    state: PublishItemState::AwaitingPubrec,
                    qos_level: 2,
                    flow_direction: PublishItemDirection::ToSubscriber,
                });
            }

            let mut has_received_pubrec: bool = false;
            // Wait for 2220 miliseconds, calculated by the max size of a payload (256 mb)
            // Downloaded with a 10Mbps internet connection (205 seconds), and then a little 
            // extra time to handle the packet and get the reply.
            'pubrec: while !has_received_pubrec {
                for _i in 0..2220 {
                    match rx.try_recv() {
                        Ok(state) => {
                            if state == PublishItemState::PubrecRecieved {

                                let mut publish_queue: MutexGuard<
                                    '_,
                                    Vec<PublishQueueItem>,
                                > = publish_queue_clone.lock().unwrap();
                                if let Some(index) = publish_queue.iter().position(
                                    |t: &PublishQueueItem| t.packet_id == packet_id,
                                ) {
                                    publish_queue[index].state = state;
                                }

                                has_received_pubrec = true;
                                break 'pubrec;
                            }
                        }
                        Err(err) => {
                            println!("{1}Subscriber QoS 2 (Pubrec) | {5} Error! -> {2}{3}{0}{4}",
                                err,
                                Color::BrightRed,
                                Reset::All,
                                Style::Italic,
                                Reset::All,
                                client_clone.id
                            );
                        }
                    }

                    thread::sleep(Duration::from_millis(100));
                }

                // Set dup flag on packet
                println!("{:b}", &packet[0]);
                packet[0] |= 1 << 3;
                _ = client_clone.tx.send(Ok(packet.clone()))
            }

            // The control packet type and remaining length of the packet
            let mut pubrel_packet: Vec<u8> = vec![96, 2];
            
            // Appends the packet id to the PUBREL packet
            pubrel_packet.append(common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut(),);

            // Sends pubrel to the client
            _ = client_clone.tx.send(Ok(pubrel_packet));

            let mut has_received_pubcomp: bool = false;
            // Waits for the client to send a pubcomp
            'pubcomp: while !has_received_pubcomp {
                for _i in 0..2220 {
                    match rx.try_recv() {
                        Ok(state) => {
                            if state == PublishItemState::PubcompRecieved {
                                let mut publish_queue: MutexGuard<
                                    '_,
                                    Vec<PublishQueueItem>,
                                > = publish_queue_clone.lock().unwrap();
                                if let Some(index) = publish_queue.iter().position(
                                    |t: &PublishQueueItem| t.packet_id == packet_id,
                                ) {
                                    publish_queue.remove(index);
                                }
                                has_received_pubcomp = true;
                                break 'pubcomp;
                            }
                        }
                        Err(_) => {}
                    }

                    thread::sleep(Duration::from_millis(100));
                }
                
                // Set the control packet type and remaining length of the packet
                let mut pubrel_packet: Vec<u8> = vec![96, 2];
                
                // Appends the packet id to the PUBREL packet
                pubrel_packet.append(common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut(),);
                
                // Send the PUBREL packet to the client
                _ = client_clone.tx.send(Ok(pubrel_packet));
            }

        });
    }
}
