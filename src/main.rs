use std::{ io::{ Read, Write }, net::{ SocketAddr, TcpListener, TcpStream }, thread, u8 };
use local_ip_address::local_ip;

mod control_packet;

fn handle_connection(mut stream: TcpStream) {
    // Print client connection information
    println!("Client connected: {:?}", stream.peer_addr().unwrap());

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
                        let return_packet: [u8; 4] = control_packet::connect::validate(
                            buffer,
                            bytes_read
                        );
                        let _ = stream.write(&return_packet);
                        let _ = stream.flush();
                    }
                    2 => {
                        // CONNACK
                    }
                    3 => {
                        // PUBLISH
                    }
                    4 => {
                        // PUBACK
                    }
                    5 => {
                        // PUBREC
                    }
                    6 => {
                        // PUBREL
                    }
                    7 => {
                        // PUBCOMP
                    }
                    8 => {
                        // SUBSCRIBE
                    }
                    9 => {
                        // SUBACK
                    }
                    10 => {
                        // UNSUBSCRIBE
                    }
                    11 => {
                        // UNSUBACK
                    }
                    12 => {
                        // PINGREQ
                    }
                    13 => {
                        // PINGRESP
                    }
                    14 => {
                        // FUCK OFF! (DISCONNECT)
                    }

                    _ => {}
                }

                // CONNECT

                // SUBCRIBE

                // PUBLISH

                // PING

                // DISCONNECT
            }
            Err(err) => {
                // Print error if reading from the client fails
                println!("Error reading from client: {:?}", err);
                return;
            }
        }
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
    println!("MQTT broker listening on port 1883...");

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
