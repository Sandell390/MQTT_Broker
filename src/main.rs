use std::thread;
use std::io::{ Read, Write };
use std::net::{ SocketAddr, TcpListener, TcpStream };
use std::sync::{ Arc, Mutex, MutexGuard };
use std::time::{ Duration, Instant };
use local_ip_address::local_ip;

use crate::models::client::Client;
use crate::models::{sub_info, topicfilter};

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

    // Create a mutex-protected clients vector
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    // Clone clients for use in the thread
    let clients_clone: Arc<Mutex<Vec<Client>>> = Arc::clone(&clients);

    // Start a background thread for monitoring keep alive
    thread::spawn(move || {
        monitor_keep_alive(clients_clone);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone clients for each thread
                let clients_clone: Arc<Mutex<Vec<Client>>> = Arc::clone(&clients);

                // Spawn a new thread to handle the client connection
                thread::spawn(move || {
                    handle_connection(stream, clients_clone); // Handle the client connection
                });
            }
            Err(err) => {
                // Print error if accepting a client connection fails
                println!("Error accepting client connection: {:?}", err);
            }
        }
    }
}

fn monitor_keep_alive(clients: Arc<Mutex<Vec<Client>>>) {
    loop {
        // Sleep for a short duration
        thread::sleep(Duration::from_secs(1));

        // Access the clients vector within the mutex
        let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

        // Iterate over the indices of the clients vector
        let now: Instant = Instant::now();
        let mut i: usize = 0;
        while i < clients.len() {
            if clients[i].keep_alive > 0 {
                if let Some(last_packet_received) = clients[i].last_packet_received {
                    let elapsed = now - last_packet_received;

                    if elapsed > Duration::from_secs(((clients[i].keep_alive as u64) * 3) / 2) {
                        // Disconnect the client
                        println!(
                            "Exceeded keep alive timeout, disconnecting client: {:?}",
                            clients[i]
                        );

                        clients[i].handle_disconnect();
                        clients.remove(i);

                        continue; // Skip incrementing i since we removed an element
                    }
                }
            }

            i += 1;
        }
    }
}

fn handle_connection(mut stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>) {
    let socket_addr: SocketAddr = stream.peer_addr().unwrap();
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

                // Match for incoming packets
                match packet_type {
                    1 => {
                        // Connect
                        if !has_first_packet_arrived {
                            match
                                control_packet::connect::validate(
                                    buffer,
                                    packet_length,
                                    socket_addr
                                )
                            {
                                Ok(response) => {
                                    if
                                        response.return_packet != [32, 2, 0, 0] &&
                                        response.return_packet != [32, 2, 1, 0]
                                    {
                                        println!(
                                            "Connack not accepted {:?}",
                                            response.return_packet
                                        );

                                        let _ = stream.shutdown(std::net::Shutdown::Both);
                                        break;
                                    }

                                    // Create a new client from the response of connect::validate()
                                    let new_client: Client = response.client;

                                    // Access the clients vector within the mutex
                                    let mut clients: MutexGuard<'_, Vec<Client>> = clients
                                        .lock()
                                        .unwrap();

                                    if
                                        let Some(existing_client) = clients
                                            .iter_mut()
                                            .find(|c: &&mut Client| c.id == new_client.id)
                                    {
                                        if existing_client.is_connected {
                                            // Reject the connection
                                            let _ = stream.write(&[32, 2, 0, 2]); // Reject Identifier
                                            let _ = stream.flush();

                                            break; // Exit the loop without adding the client to the list
                                        } else {
                                            // Update the existing client to be connected
                                            existing_client.will_topic = new_client.will_topic;
                                            existing_client.will_message = new_client.will_message;
                                            existing_client.is_connected = true;
                                            existing_client.subscriptions =
                                                new_client.subscriptions;
                                            existing_client.keep_alive = new_client.keep_alive;
                                            existing_client.username = new_client.username;
                                            existing_client.password = new_client.password;
                                            existing_client.socket_addr = socket_addr;
                                            existing_client.connect_flags =
                                                new_client.connect_flags;

                                            // Update last packet time for the client
                                            update_client_last_packet_time(
                                                &mut clients,
                                                &new_client.id
                                            );
                                        }
                                    } else {
                                        // Update last packet time for the client
                                        update_client_last_packet_time(
                                            &mut clients,
                                            &new_client.id
                                        );

                                        // Add the new client to the list
                                        clients.push(new_client);
                                    }

                                    // Continue with handling the connection
                                    // Send response to the client
                                    let _ = stream.write(&response.return_packet);
                                    let _ = stream.flush();

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
                            let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            // Validation Logic Goes here, I think...
                            match control_packet::subcribe::validate(buffer, packet_length) {
                                Ok(sub_packet) => {
                                
                                    if let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr){

                                        // Adding topic filters to the client
                                        for topicfilter in sub_packet.topic_qos_pair {
                                            clients[index].add_subscription(topicfilter);
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
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    12 => {
                        // PINGREQ
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
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

                            // Validate reserved bits are not set
                            match control_packet::disconnect::validate(buffer, packet_length) {
                                Ok(_response) => {
                                    disconnect_client_by_socket_addr(
                                        &mut clients,
                                        socket_addr,
                                        true
                                    );
                                }
                                Err(_err) => {
                                    disconnect_client_by_socket_addr(
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

    disconnect_client_by_socket_addr(&mut clients, socket_addr, false);

    println!("Client disconnected: {}", socket_addr);

    // Print information for each client
    println!("Client List:");
    for client in clients.iter() {
        println!("Client: {:?}", client);
    }

    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn update_client_last_packet_time(clients: &mut Vec<Client>, client_id: &str) {
    if let Some(index) = clients.iter().position(|c: &Client| c.id == client_id) {
        // Update the client's last_packet_received
        clients[index].last_packet_received = Some(Instant::now());

        println!("Client found and updated keep alive.");
    } else {
        println!("Client not found");
    }
}

fn disconnect_client_by_socket_addr(
    clients: &mut Vec<Client>,
    socket_addr: SocketAddr,
    discard_will_msg: bool
) {
    if let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr) {
        // Extract the client from the list
        let mut client: Client = clients.remove(index);

        // Call handle_disconnect on the client
        client.handle_disconnect();

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
