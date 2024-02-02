use std::{ io::{ Read, Write }, net::{ SocketAddr, TcpListener, TcpStream }, thread, u8 };
use local_ip_address::local_ip;

mod control_packet;
mod common_fn;

fn handle_connection(mut stream: TcpStream) {
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
                    return;
                }

                // Convert first 4 bits to decimal value
                let packet_type: u8 = common_fn::bit_operations::split_byte(&buffer[0], 4).expect("")[0];

                // Match for incoming packets
                match packet_type {
                    1 => {
                        // Connect
                        if !has_first_packet_arrived {
                            match control_packet::connect::validate(buffer, bytes_read) {
                                Ok(return_packet) => {
                                    let _ = stream.write(&return_packet);
                                    let _ = stream.flush();

                                    if return_packet != [32, 2, 0, 0] {
                                        println!("Connack not accepted {:?}", return_packet);
                                        let _ = stream.shutdown(std::net::Shutdown::Both);
                                    }
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
                        } else {
                            // Disconnect
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            return;
                        }
                        return;
                    }
                    _ => {
                        // Disconnect
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        return;
                    }
                }

                // CONNECT

                // SUBCRIBE

                // PUBLISH

                // PING

                // Disconnect
            }
            Err(err) => {
                // Print error if reading from the client fails
                println!("Error reading from client: {:?}", err);
                return;
            }
        }

        has_first_packet_arrived = true;
    }
}

fn main() {
    // Fetch current ip
    let my_local_ip = local_ip().unwrap();

    // Create a TCP listener bound to port 1883 (the default MQTT port)
    let listener = TcpListener::bind(SocketAddr::new(my_local_ip, 1883)).expect(
        "Failed to bind to port 1883"
    );

    // Print a message indicating that the MQTT broker is listening
    println!("MQTT broker listening on {}:1883...", my_local_ip);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(err) => {
                // Print error if accepting a client connection fails
                println!("Error accepting client connection: {:?}", err);
            }
        }
    }
}
