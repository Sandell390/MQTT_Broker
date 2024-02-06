use std::sync::mpsc::{ channel, Receiver, Sender };
use std::thread;
use std::io::{ Read, Write };
use std::net::{ SocketAddr, TcpListener, TcpStream };
use std::sync::{ Arc, Mutex, MutexGuard };
use std::time::Duration;
use local_ip_address::local_ip;

use crate::models::client::Client;
use crate::models::topic::Topic;

mod control_packet;
mod common_fn;
mod models;

/// Entry point of the MQTT broker application.
///
/// # Description
///
/// This function serves as the entry point of the MQTT broker application.
/// It initializes the TCP listener bound to port 1883, creates mutex-protected vectors for topics and clients,
/// and continuously listens for incoming client connections.
///
/// For each incoming connection, it spawns a new thread
/// to handle the client connection using the `handle_connection` function.
fn main() {
    // Fetch current ip
    let my_local_ip: std::net::IpAddr = local_ip().unwrap();

    // Create a TCP listener bound to port 1883 (the default MQTT port)
    let listener: TcpListener = TcpListener::bind(SocketAddr::new(my_local_ip, 1883)).expect(
        "Failed to bind to port 1883"
    );

    // Print a message indicating that the MQTT broker is listening
    println!("MQTT broker listening on {}:1883...", my_local_ip);

    // Create a mutex-protected topics Vector
    let topics: Arc<Mutex<Vec<Topic>>> = Arc::new(Mutex::new(Vec::new()));

    // Create a mutex-protected clients vector
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    // For each incoming connection -> Spawn a new thread to handle the client's connection, using the handle_connection function
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone clients for each thread
                let clients_clone: Arc<Mutex<Vec<Client>>> = Arc::clone(&clients);
                let topics_clone: Arc<Mutex<Vec<Topic>>> = Arc::clone(&topics);

                // Spawn a new thread to handle the client connection
                thread::spawn(move || {
                    handle_connection(stream, clients_clone, topics_clone); // Handle the client connection
                });
            }
            Err(err) => {
                // Print error if accepting a client connection fails
                println!("Error accepting client connection: {:?}", err);
            }
        }
    }
}

/// Handles the connection with a client, continuously reading data from the client
/// and processing incoming packets according to the MQTT protocol.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TCP stream representing the connection with the client.
/// * `clients` - An Arc-wrapped Mutex-protected vector of clients currently connected to the server.
/// * `topics` - An Arc-wrapped Mutex-protected vector of topics subscribed to by clients.
///
/// # Description
///
/// This function processes incoming packets from the client according to the MQTT protocol.
/// It continuously reads data from the client, interprets packet types, and handles them appropriately.
/// Depending on the packet type, it performs actions such as establishing connections,
/// publishing messages, subscribing to topics, unsubscribing, responding to PING requests, and disconnecting.
///
/// # Notes
///
/// This function spawns a separate thread to handle message transmission to the client
/// and ensures that each client's connection and disconnection are logged.
/// It also prints information about connected clients for debugging purposes.
/// Furthermore it creates a channel (Transmit and Recieve), to handle communication between threads.
fn handle_connection(
    mut stream: TcpStream,
    clients: Arc<Mutex<Vec<Client>>>,
    topics: Arc<Mutex<Vec<Topic>>>
) {
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

    let mut stream_clone: TcpStream = stream.try_clone().unwrap();
    thread::spawn(move || {
        for message in rx {
            // Sends the message to the client
            println!("Sending Publish pakcet");
            let _ = stream_clone.write(message.as_slice());
            let _ = stream_clone.flush();
        }
    });

    let socket_addr: SocketAddr = stream.peer_addr().unwrap();
    let _ = stream.set_read_timeout(Some(Duration::from_secs(0)));

    // Print client connection information
    println!("Client connected: {}", socket_addr);

    let mut has_first_packet_arrived: bool = false;
    let mut discard_will_msg: bool = false;

    // Infinite loop to continuously read data from the client
    loop {
        // Buffer to store received data from the client
        let mut buffer: [u8; 8192] = [0; 8192];

        match stream.read(&mut buffer) {
            Ok(packet_length) => {
                // Check if the client has suddenly disconnected
                if packet_length == 0 {
                    break;
                }

                // Convert first 4 bits to decimal value
                let packet_type: u8 = common_fn::bit_operations
                    ::split_byte(&buffer[0], 4)
                    .expect("")[0];

                println!("Recieved packet: {}", packet_type);
                println!("{:?}", &buffer[..packet_length]);
                // Match for incoming packets
                match packet_type {
                    1 => {
                        // Connect
                        if !has_first_packet_arrived {
                            // Access the clients vector within the mutex
                            let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            match
                                control_packet::connect::handle(
                                    buffer,
                                    packet_length,
                                    socket_addr,
                                    &mut clients,
                                    tx.clone()
                                )
                            {
                                Ok(response) => {
                                    let keep_alive: u64 = response.keep_alive;

                                    // Continue with handling the connection
                                    // Send response to the client
                                    _ = tx.send(response.return_packet.to_vec());

                                    // Set keep_alive
                                    let _ = stream.set_read_timeout(
                                        Some(Duration::from_secs(keep_alive))
                                    );

                                    // DEBUG
                                    // Print information for each client
                                    println!("Client List:");
                                    for client in clients.iter() {
                                        println!("Client: {:?}", client);
                                    }
                                }
                                Err(err) => {
                                    println!("An error has occured: {}", err);
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    3 => {
                        // PUBLISH
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    4 => {
                        // PUBACK
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    5 => {
                        // PUBREC
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    6 => {
                        // PUBREL
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    7 => {
                        // PUBCOMP
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    8 => {
                        // SUBSCRIBE
                        if has_first_packet_arrived {
                            // Access the clients vector within the mutex
                            let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            // Access the topic Vector
                            let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();

                            // Validation Logic Goes here, I think...
                            match control_packet::subcribe::handle(buffer, packet_length) {
                                Ok(sub_packet) => {
                                    if
                                        let Some(index) = clients
                                            .iter()
                                            .position(|c: &Client| c.socket_addr == socket_addr)
                                    {
                                        // Adding topic filters to the client
                                        for topicfilter in sub_packet.topic_qos_pair {
                                            //clients[index].add_subscription(topicfilter);
                                            add_client_to_topic_list(
                                                &mut topics,
                                                clients[index].id.clone(),
                                                topicfilter
                                            );
                                        }
                                    }

                                    _ = tx.send(sub_packet.return_packet);
                                }
                                Err(err) => {
                                    println!("An error has occured: {}", err);
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    10 => {
                        // UNSUBSCRIBE
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                            // Access the clients vector within the mutex
                            let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            // Validation Logic Goes here, I think...
                            match control_packet::unsubcribe::handle(buffer, packet_length) {
                                Ok(unsub_packet) => {
                                    if
                                        let Some(index) = clients
                                            .iter()
                                            .position(|c: &Client| c.socket_addr == socket_addr)
                                    {
                                        // Access the topic Vector
                                        let mut topics: MutexGuard<'_, Vec<Topic>> = topics
                                            .lock()
                                            .unwrap();
                                        // Removing topic filters to the client
                                        for topicfilter in unsub_packet.topic_qos_pair {
                                            remove_client_from_topic_list(
                                                &mut topics,
                                                clients[index].id.clone(),
                                                topicfilter
                                            );
                                        }
                                    }

                                    _ = tx.send(unsub_packet.return_packet);
                                }
                                Err(err) => {
                                    println!("An error has occured: {}", err);
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    12 => {
                        // PINGREQ
                        if has_first_packet_arrived {
                            match control_packet::ping::handle(buffer, packet_length) {
                                Ok(return_packet) => {
                                    // Send response to the client

                                    _ = tx.send(return_packet.to_vec());
                                }
                                Err(err) => {
                                    println!("{err}");
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    14 => {
                        // Disconnect
                        if has_first_packet_arrived {
                            // Validate reserved bits are not set
                            match control_packet::disconnect::handle(buffer, packet_length) {
                                Ok(_response) => {
                                    discard_will_msg = true;
                                }
                                Err(_err) => {
                                    discard_will_msg = true;
                                }
                            }

                            break;
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    _ => {
                        // Disconnect
                        break;
                    }
                }
            }
            Err(err) => {
                // Print error if reading from the client fails
                println!("Error reading from client: {:?}\nClosing the Stream", err);
                break;
            }
        }

        has_first_packet_arrived = true;
    }

    // Access the clients vector within the mutex
    let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

    // Access the topics vector within the mutex
    let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();

    disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, discard_will_msg);

    println!("Client disconnected: {}", socket_addr);

    // Print information for each client
    println!("Client List:");
    for client in clients.iter() {
        println!("Client: {:?}", client);
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

/// Disconnects a client based on its socket address and performs cleanup tasks.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `clients` - A mutable reference to a vector of clients.
/// * `socket_addr` - The socket address of the client to be disconnected.
/// * `discard_will_msg` - A boolean indicating whether to discard the client's will message.
///
/// # Description
///
/// This function disconnects a client based on its socket address and performs the following tasks:
/// - Publishes the will message to clients that have subscribed to the will topic.
/// - Calls the `handle_disconnect` method on the client.
/// - Optionally discards the client's will message if specified.
/// - Re-adds the updated client to the list of clients.
///
/// # Examples
/// ```
/// // Access the clients vector within the mutex
/// let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
///
/// // Access the topics vector within the mutex
/// let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
///
/// // Obtain the socket address
/// let socket_addr: SocketAddr = stream.peer_addr().unwrap();
///
/// disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, false);
/// ```
fn disconnect_client_by_socket_addr(
    topics: &mut Vec<Topic>,
    clients: &mut Vec<Client>,
    socket_addr: SocketAddr,
    discard_will_msg: bool
) {
    if let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr) {
        // Extract the client from the list
        let mut client: Client = clients.remove(index);

        // Publish the will message to clients that have subscribed on the will topic
        control_packet::publish::publish(
            topics,
            clients,
            &client.will_topic,
            &client.will_message,
            &false,
            &client.connect_flags.will_qos_flag,
            &client.connect_flags.will_retain_flag
        );

        // Call handle_disconnect on the client
        client.handle_disconnect();

        if discard_will_msg {
            client.will_message = String::new();
        }

        // Re-add the updated client to the list
        clients.push(client);

        println!("Client found and disconnected.");
    } else {
        println!("Client could not be disconnected: Client not found.");
    }
}

/// Adds a client and its associated topic filter to the list of topics.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `client_id` - The ID of the client to be added.
/// * `topic_filter` - A tuple containing the topic name and quality of service (QoS) level.
///
/// # Description
///
/// This function adds a client and its associated topic filter to the list of topics.
/// It searches for the specified topic within the topics vector and either adds the client
/// to the existing topic or creates a new topic if the specified topic does not exist.
///
/// # Examples
///
/// ```
/// // Access the clients vector within the mutex
/// let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
/// // Access the topic Vector
/// let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
/// // Validation Logic Goes here, I think...
/// match control_packet::subcribe::validate(buffer, packet_length) {
///     Ok(sub_packet) => {
///         if
///             let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr)
///         {
///             // Adding topic filters to the client
///             for topicfilter in sub_packet.topic_qos_pair {
///                 //clients[index].add_subscription(topicfilter);
///                 add_client_to_topic_list(&mut topics, clients[index].id.clone(), topicfilter);
///             }
///         }
///         _ = tx.send(sub_packet.return_packet);
///     }
/// ```
fn add_client_to_topic_list(
    topics: &mut Vec<Topic>,
    client_id: String,
    topic_filter: (String, u8)
) {
    if let Some(index) = topics.iter().position(|t: &Topic| t.topic_name == topic_filter.0) {
        topics[index].client_ids.push((client_id, topic_filter.1));
    } else {
        let mut new_topic: Topic = Topic::new(topic_filter.0);

        new_topic.client_ids.push((client_id, topic_filter.1));
        topics.push(new_topic);
    }
}

/// Removes a client from being subcribed to a specific topic, from the list of topics.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `client_id` - The ID of the client whose topic filter is to be removed.
/// * `topic_filter` - A tuple containing the topic name and quality of service (QoS) level.
///
/// # Description
///
/// This function removes a specific topic filter associated with a client from the list of topics.
/// It searches for the specified topic filter within the topics vector and removes it
/// from the client IDs associated with that topic, if found.
///
/// # Examples
///
/// ```
/// // Access the clients vector within the mutex
/// let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
/// match control_packet::unsubcribe::validate(buffer, packet_length) {
///     Ok(unsub_packet) => {
///         if
///             let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr)
///         {
///             // Access the topic Vector
///             let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
///             // Removing topic filters to the client
///             for topicfilter in unsub_packet.topic_qos_pair {
///                 remove_client_topic_list(&mut topics, clients[index].id.clone(), topicfilter);
///             }
///         }
///
///         _ = tx.send(unsub_packet.return_packet);
///     }
/// ```
fn remove_client_from_topic_list(
    topics: &mut Vec<Topic>,
    client_id: String,
    topic_filter: (String, u8)
) {
    if let Some(index) = topics.iter().position(|t: &Topic| t.topic_name == topic_filter.0) {
        if
            let Some(client_index) = topics[index].client_ids
                .iter()
                .position(|c: &(String, u8)| &c.0 == &client_id)
        {
            topics[index].client_ids.remove(client_index);
        }
    }
}
