use std::thread;
use std::io::{ Read, Write };
use std::net::{ SocketAddr, TcpListener, TcpStream };
use std::sync::{ Arc, Mutex, MutexGuard };
use local_ip_address::local_ip;

use crate::models::client::Client;

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
                            match
                                control_packet::connect::validate(buffer, bytes_read, socket_addr)
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
                                            existing_client.is_connected = true;
                                            existing_client.socket_addr = socket_addr;
                                        }
                                    } else {
                                        // Add the new client to the list
                                        clients.push(new_client);
                                    }

                                    // Continue with handling the connection
                                    // Send response to the client
                                    let _ = stream.write(&response.return_packet);
                                    let _ = stream.flush();

                                    // Print information for each client
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
                            // Access the clients vector within the mutex
                            let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            if
                                let Some(index) = clients
                                    .iter()
                                    .position(|c: &Client| c.socket_addr == socket_addr)
                            {
                                // Extract the client from the list
                                let mut client: Client = clients.remove(index);

                                // Call handle_disconnect on the client
                                client.handle_disconnect();

                                // Re-add the updated client to the list
                                clients.push(client);

                                println!("Client found and updated.");
                            } else {
                                println!("Client not found.");
                            }

                            // Print information for each client
                            for client in clients.iter() {
                                println!("Client: {:?}", client);
                            }

                            // match
                            //     control_packet::disconnect::validate(
                            //         buffer,
                            //         bytes_read,
                            //         socket_addr
                            //     )
                            // {
                            //     Ok(Response) => {
                            //         // Code here
                            //     }
                            //     Err(err) => {
                            //         println!("{}", err);
                            //     }
                            // }
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

    // TO-DO: Instead of removing the client, update the is_connected flag to false, that way we still retain state
    remove_client(&mut clients, &socket_addr);

    println!("Client disconnected: {}", socket_addr);
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn remove_client(clients: &mut MutexGuard<Vec<Client>>, socket_addr: &SocketAddr) {
    if
        let Some(index) = clients
            .iter()
            .position(|client: &Client| client.socket_addr == *socket_addr)
    {
        clients.remove(index);
    }
}
