use std::{ net::SocketAddr, sync::mpsc::Sender };

use crate::{ common_fn, models::{ client::Client, flags::ConnectFlags } };

pub struct Response {
    pub return_packet: [u8; 4],
    pub keep_alive: u64,
}

pub fn handle(
    buffer: [u8; 8192],
    packet_length: usize,
    socket_addr: SocketAddr,
    clients: &mut Vec<Client>,
    tx: Sender<Vec<u8>>
) -> Result<Response, &'static str> {
    println!("MQTT Connection is being validated");

    // Validate packet
    let mut remaining_length: usize = 0;
    let mut connect_return_code: u8 = 0; // Used for assembling the connack packet

    match common_fn::bit_operations::decode_remaining_length(&buffer) {
        Ok(value) => {
            remaining_length = value;
        }
        Err(err) => println!("Error: {}", err),
    }

    let mut has_valid_protocol_length_and_name = true;
    let mut current_index: usize = packet_length - remaining_length;

    // Control protocol length & name
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
        Ok(response) => {
            // Match conditions
            if response.0 != 4 && response.1 != "MQTT" {
                has_valid_protocol_length_and_name = false;
            }

            current_index = response.2;
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    if !has_valid_protocol_length_and_name {
        return Err("Invalid protocol name");
    }

    // Control protocol level must be 4 (3.1.1)
    if (buffer[current_index] as u8) != 4 {
        connect_return_code = 1;
    }

    current_index += 1; // Move the pointer 1 byte to the right

    // Control the connect flags
    let byte: u8 = buffer[current_index];

    // Extracting individual flags
    let flag_0: bool = (byte & 0b00000001) != 0; // Reserved, must be 0
    let flag_1: bool = (byte & 0b00000010) != 0; // Clean Session
    let flag_2: bool = (byte & 0b00000100) != 0; // Will Flag
    let mut flag_3: bool = (byte & 0b00001000) != 0; // QoS 1 (Note: Both can not be true)
    let mut flag_4: bool = (byte & 0b00010000) != 0; // QoS 2 (Note: Both can not be true)
    let mut flag_5: bool = (byte & 0b00100000) != 0; // Will Retain
    let flag_6: bool = (byte & 0b01000000) != 0; // Password Flag
    let flag_7: bool = (byte & 0b10000000) != 0; // Username Flag

    let mut qos_level: u8 = 0;

    // Reserved flag
    if flag_0 {
        return Err("Reserved flag is not 0");
    }

    // Will flag
    if !flag_2 {
        flag_3 = false;
        flag_4 = false;
        flag_5 = false;
        // Disconnect if will message or topic is pressent
    }

    // QoS 1 flag
    if flag_3 {
        qos_level = 1;
    }

    // QoS 2 flag
    if flag_4 {
        qos_level = 2;
    }

    if flag_3 && flag_4 {
        return Err("Wrong QoS level specified");
    }

    let connect_flags: ConnectFlags = ConnectFlags::new(
        flag_1,
        flag_2,
        qos_level,
        flag_5,
        flag_6,
        flag_7
    );

    current_index += 1; // Move the pointer 1 byte to the right

    let mut keep_alive: u64 = 0;

    // Read the keep alive byte (MSB & LSB)
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, false) {
        Ok(response) => {
            keep_alive = response.0 as u64;

            current_index = response.2;
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    // Field order from here MUST be:
    // Client Identifier -> Will Topic -> Will Message -> User Name -> Password

    let mut client_id: String = String::new();
    let mut will_topic: String = String::new();
    let mut will_message: String = String::new();

    // Read the Client Identifier (MSB & LSB)
    match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
        Ok(response) => {
            client_id = response.1;

            current_index = response.2;
        }
        Err(err) => {
            println!("{}", err);
        }
    }

    // If will flag is true
    if connect_flags.will_flag {
        // Read the Will Topic (MSB & LSB)
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                will_topic = response.1;

                current_index = response.2;
            }
            Err(err) => {
                println!("{}", err);
            }
        }

        // Read the Will Message (MSB & LSB)
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                will_message = response.1;

                current_index = response.2;
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    let mut username: String = String::new();
    let mut password: String = String::new();

    // If username flag is true
    if connect_flags.username_flag {
        // Read the Username (MSB & LSB)
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                username = response.1;

                current_index = response.2;
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    // If password flag is true
    if connect_flags.password_flag {
        // Read the Username (MSB & LSB)
        match common_fn::msb_lsb_reader::get_values(&buffer, current_index, true) {
            Ok(response) => {
                password = response.1;

                current_index = response.2;
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    let client: Client = Client::new(
        client_id,
        will_topic,
        will_message,
        keep_alive,
        username,
        password,
        socket_addr,
        tx.clone(),
        connect_flags
    );

    // Set to 1.5 times the specified amount, AFTER a new Client is created.
    keep_alive = (keep_alive * 3) / 2;

    if packet_length != current_index {
        return Err("Invalid packet");
    }

    // Assemble return packet
    let mut session_present_byte: u8 = 0;

    if let Some(existing_client) = clients.iter_mut().find(|c: &&mut Client| c.id == client.id) {
        if existing_client.is_connected {
            // Reject the connection
            connect_return_code = 2;
        } else {
            // Update the existing client to be connected
            existing_client.keep_alive = client.keep_alive;
            existing_client.username = client.username;
            existing_client.password = client.password;
            existing_client.connect_flags = client.connect_flags;

            if existing_client.connect_flags.clean_session_flag {
                existing_client.will_topic = client.will_topic;
                existing_client.will_message = client.will_message;
                existing_client.subscriptions = client.subscriptions;
                // Store QoS messages, not yet completed
            } else {
                if connect_return_code == 0 {
                    session_present_byte = 1;
                }
            }

            existing_client.socket_addr = socket_addr;
            existing_client.is_connected = true;
        }
    } else {
        // Add the new client to the list
        clients.push(client);
    }

    let connack_packet: [u8; 4] = [32, 2, session_present_byte, connect_return_code];

    if connack_packet != [32, 2, 0, 0] && connack_packet != [32, 2, 1, 0] {
        println!("Connack not accepted {:?}", connack_packet);

        return Err("");
    }

    // Return newly assembled return packet
    return Ok(Response { return_packet: connack_packet, keep_alive });
}
