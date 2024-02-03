use std::thread;
use std::collections::HashSet;
use std::io::{ Read, Write };
use std::sync::{ Arc, Mutex };
use std::net::{ SocketAddr, TcpListener, TcpStream };
use local_ip_address::local_ip;

use crate::models::client::Client;

mod control_packet;
mod common_fn;
mod models;

fn main() {
    // Fetch current ip
    let my_local_ip = local_ip().unwrap();

    // Create an Arc<Mutex<HashSet<Client>>> to store clients
    let clients: Arc<Mutex<HashSet<Client>>> = Arc::new(Mutex::new(HashSet::new()));

    // Create a TCP listener bound to port 1883 (the default MQTT port)
    let listener = TcpListener::bind(SocketAddr::new(my_local_ip, 1883)).expect(
        "Failed to bind to port 1883"
    );

    // Print a message indicating that the MQTT broker is listening
    println!("MQTT broker listening on {}:1883...", my_local_ip);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_connection(stream, clients_clone);
                });
            }
            Err(err) => {
                // Print error if accepting a client connection fails
                println!("Error accepting client connection: {:?}", err);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, clients: Arc<Mutex<HashSet<Client>>>) {
    let client_addr: SocketAddr = stream.peer_addr().unwrap();

    // Print client connection information
    println!("Client connected: {}", client_addr);

    let mut has_first_packet_arrived: bool = false;

    // Infinite loop to continuously read data from the client
    loop {
        // Buffer to store received data from the client
        let mut buffer: [u8; 8192] = [0; 8192];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                // Check if the client has suddenly disconnected
                if bytes_read == 0 {
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
                            match control_packet::connect::validate(buffer, bytes_read) {
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

                                    // Acquire a lock on the mutex guarding the clients HashSet
                                    let mut clients_guard = clients.lock().unwrap();
                                    let client: Client = response.client;

                                    // Check if a client with the same client_id already exists
                                    if
                                        clients_guard
                                            .iter()
                                            .any(
                                                |internal_client|
                                                    internal_client.client_id == client.client_id &&
                                                    internal_client.is_connected ==
                                                        client.is_connected
                                            )
                                    {
                                        // Client with the same client_id already exists, and is already connected
                                        println!(
                                            "Client: {:?} already exists & is already connected.",
                                            client
                                        );

                                        let _ = stream.write(&[32, 2, 0, 2]); // Reject Identifier
                                        let _ = stream.flush();

                                        break; // Exit the function without adding the client
                                    }

                                    // Insert/Update the client within the clients HashSet
                                    // 'Adds a value to the set, replacing the existing value, if any, that is equal to the given one.'
                                    clients_guard.replace(client);

                                    // Create a Vec to temporarily store clients
                                    let clients_vec: Vec<&Client> = clients_guard.iter().collect();

                                    // Iterate over all clients in the clients HashSet
                                    for client in clients_vec {
                                        // Access client properties or perform operations
                                        println!("Client: {:?}", client);
                                    }

                                    let _ = stream.write(&response.return_packet);
                                    let _ = stream.flush();
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
                            // Validation Logic Goes here, I think...
                            match control_packet::subcribe::validate(buffer, bytes_read) {
                                Ok(sub_packet) => {
                                    /* 
                                    // Debug prints
                                    println!("PacketID: {:?}", sub_packet.0);
                                    for (topic_filter, qos) in &sub_packet.1 {
                                        println!("{topic_filter:?} {qos}");
                                    }
                                    */
                                    // Convert Hashmap vaules to Vec u8
                                    let values: Vec<u8> = sub_packet.1.values().cloned().collect();
                                    // Assembles the return and sends it
                                    match
                                        control_packet::subcribe::assemble_suback_packet(
                                            values.as_slice(),
                                            &sub_packet.0
                                        )
                                    {
                                        Ok(return_packet_vec) => {
                                            // Convert the byte vector into a byte slice
                                            let suback_buffer: &[u8] = return_packet_vec.as_slice();

                                            // Sends to the client
                                            let _ = stream.write(suback_buffer);
                                        }
                                        Err(_) => println!("Error"),
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
                            // Validation Logic Goes here, I think...
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

    println!("Client disconnected: {}", client_addr);
    let _ = stream.shutdown(std::net::Shutdown::Both);
}
