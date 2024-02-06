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

fn main() {
    // Fetch current ip
    let my_local_ip: std::net::IpAddr = local_ip().unwrap();

    // Create a TCP listener bound to port 1883 (the default MQTT port)
    let listener: TcpListener = TcpListener::bind(SocketAddr::new(my_local_ip, 1883)).expect(
        "Failed to bind to port 1883"
    );

    // Print a message indicating that the MQTT broker is listening
    println!("MQTT broker listening on {}:1883...", my_local_ip);

    // Create a mutex-protected topic Vector
    let topics: Arc<Mutex<Vec<Topic>>> = Arc::new(Mutex::new(Vec::new()));

    // Create a mutex-protected clients vector
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

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

fn handle_connection(
    mut stream: TcpStream,
    clients: Arc<Mutex<Vec<Client>>>,
    topics: Arc<Mutex<Vec<Topic>>>
) {
    let socket_addr: SocketAddr = stream.peer_addr().unwrap();
    let _ = stream.set_read_timeout(Some(Duration::from_secs(0)));

    // Print client connection information
    println!("Client connected: {}", socket_addr);

    let mut has_first_packet_arrived: bool = false;

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
                                    &mut clients
                                )
                            {
                                Ok(response) => {
                                    let keep_alive: u64 = response.keep_alive;
                                    // Copy the stream to the event thread
                                    let mut stream_clone = stream.try_clone().unwrap();
                                    thread::spawn(move || {
                                        for message in response.rx {
                                            // Sends the message to the client
                                            let _ = stream_clone.write(message.as_slice());
                                        }
                                    });

                                    // Continue with handling the connection
                                    // Send response to the client
                                    let _ = stream.write(&response.return_packet);
                                    let _ = stream.flush();

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
                            match control_packet::subcribe::validate(buffer, packet_length) {
                                Ok(sub_packet) => {
                                    if
                                        let Some(index) = clients
                                            .iter()
                                            .position(|c: &Client| c.socket_addr == socket_addr)
                                    {
                                        // Adding topic filters to the client
                                        for topicfilter in sub_packet.topic_qos_pair {
                                            //clients[index].add_subscription(topicfilter);
                                            add_client_topic_list(
                                                &mut topics,
                                                clients[index].id.clone(),
                                                topicfilter
                                            );
                                        }
                                    }

                                    let _ = stream.write(sub_packet.return_packet.as_slice());
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
                            match control_packet::unsubcribe::validate(buffer, packet_length) {
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
                                            remove_client_topic_list(
                                                &mut topics,
                                                clients[index].id.clone(),
                                                topicfilter
                                            );
                                        }
                                    }

                                    let _ = stream.write(unsub_packet.return_packet.as_slice());
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
                                    let _ = stream.write(&return_packet);
                                    let _ = stream.flush();
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
                        // FUCK OFF! (Disconnect)
                        if has_first_packet_arrived {
                            // Access the clients vector within the mutex
                            let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            // Access the topic Vector
                            let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();

                            // Validate reserved bits are not set
                            match control_packet::disconnect::handle(buffer, packet_length) {
                                Ok(_response) => {
                                    disconnect_client_by_socket_addr(
                                        &mut topics,
                                        &mut clients,
                                        socket_addr,
                                        true
                                    );
                                }
                                Err(_err) => {
                                    disconnect_client_by_socket_addr(
                                        &mut topics,
                                        &mut clients,
                                        socket_addr,
                                        true
                                    );
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

    disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, false);

    println!("Client disconnected: {}", socket_addr);

    // Print information for each client
    println!("Client List:");
    for client in clients.iter() {
        println!("Client: {:?}", client);
    }

    let _ = stream.shutdown(std::net::Shutdown::Both);
}

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
        // TO-DO: FIX
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
        println!("Disconnecting the client");

        // Update the client's socket_addr
        client.socket_addr = socket_addr;

        if discard_will_msg {
            client.will_message = String::new();
        }

        // Re-add the updated client to the list
        clients.push(client);

        println!("Client found and updated.");
    } else {
        println!("Client not found.");
    }
}

fn add_client_topic_list(topics: &mut Vec<Topic>, client_id: String, topic_filter: (String, u8)) {
    if let Some(index) = topics.iter().position(|t: &Topic| t.topic_name == topic_filter.0) {
        topics[index].client_ids.push((client_id, topic_filter.1));
    } else {
        let mut new_topic: Topic = Topic::new(topic_filter.0);

        new_topic.client_ids.push((client_id, topic_filter.1));
        topics.push(new_topic);
    }
}

fn remove_client_topic_list(
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
