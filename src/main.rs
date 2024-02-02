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
    // Print client connection information
    println!("Client connected: {:?}", stream.peer_addr().unwrap());

    let mut has_first_packet_arrived: bool = false;

    // Infinite loop to continuously read data from the client
    loop {
        // Buffer to store received data from the client
        let mut buffer: [u8; 8192] = [0; 8192];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                // Check if the client has suddenly disconnected
                if bytes_read == 0 {
                    println!("Client disconnected: {:?}", stream.peer_addr().unwrap());
                    break;
                }

                // Extract first byte, for surgery
                let first_byte: u8 = buffer[0];

                // Convert byte to a 8-bit string
                let bits_string: String = format!("{:08b}", first_byte);

                // Split the bit_string in half & convert first 4 bits to decimal value
                let packet_type: u8 = u8
                    ::from_str_radix(bits_string.split_at(4).0, 2)
                    .expect("Could not parse first string byte to u8");

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
                                                    internal_client.client_id == client.client_id
                                            )
                                    {
                                        // Client with the same client_id already exists
                                        println!(
                                            "Client with ID '{}' already exists.",
                                            client.client_id
                                        );

                                        let _ = stream.write(&[32, 2, 0, 2]); // Reject Identifier
                                        let _ = stream.flush();

                                        break; // Exit the function without adding the client
                                    }

                                    // Insert the new client into the clients HashSet
                                    clients_guard.insert(client);

                                    // Create a Vec to temporarily store clients
                                    let clients_vec: Vec<&Client> = clients_guard.iter().collect();

                                    // Iterate over all clients in the clients HashSet
                                    for client in clients_vec {
                                        // Access client properties or perform operations
                                        println!("Client ID: {}", client.client_id);
                                    }

                                    let _ = stream.write(&response.return_packet);
                                    let _ = stream.flush();
                                }
                                Err(err) => {
                                    println!("An error has occured: {}", err);
                                    let _ = stream.shutdown(std::net::Shutdown::Both);
                                }
                            }
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    3 => {
                        // PUBLISH
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    4 => {
                        // PUBACK
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    5 => {
                        // PUBREC
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    6 => {
                        // PUBREL
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    7 => {
                        // PUBCOMP
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    8 => {
                        // SUBSCRIBE
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    10 => {
                        // UNSUBSCRIBE
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    12 => {
                        // PINGREQ
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                    }
                    14 => {
                        // FUCK OFF! (Disconnect)
                        if has_first_packet_arrived {
                            // Validation Logic Goes here, I think...
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                        }

                        println!("Client disconnected: {:?}", stream.peer_addr().unwrap());

                        break;
                    }
                    _ => {
                        // Disconnect
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        break;
                    }
                }
            }
            Err(err) => {
                // Print error if reading from the client fails
                println!("Error reading from client: {:?}", err);
                break;
            }
        }

        has_first_packet_arrived = true;
    }
}
