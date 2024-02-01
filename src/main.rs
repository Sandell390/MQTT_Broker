use std::{ io::{Read, Write}, net::{TcpListener, TcpStream}, ptr::null, thread, u8};

fn handle_connection(mut stream: TcpStream) {


    loop {
    // Loop:

    // Match packet type
        let mut buff = [0; 1024];
        match stream.read(&mut buff){
            Ok(_) => { println!("Read first byte! ")},
            Err(_) => { println!("Could not read first byte! "); continue;}
        };

        let first_byte = buff[0];

        if first_byte == 0{
            println!("Client disconnected unexpected");
            return;
        }

        println!("byte: {:?}", format!("{:08b}", first_byte));

        let bits_string: String = format!("{:08b}", first_byte);

        let packet_type =  u8::from_str_radix(bits_string.split_at(4).0, 2).expect("Could not parse first string byte to u8"); 

        println!("{packet_type}");

        
        match packet_type {
            1 => {
                // CONNECT
                println!("MQTT Connect");

                let connackByte: [u8; 4] = [32, 2, 0, 0];

                stream.write(&connackByte);
                stream.flush();
            },
            2 => {
                // CONNACK
            },
            3 => {
                // PUBLISH
            },
            4 => {
                // PUBACK
            },
            5 => {
                // PUBREC
            },
            6 => {
                // PUBREL
            },
            7 => {
                // PUBCOMP
            },
            8 => {
                // SUBSCRIBE
            },
            9 => {
                // SUBACK
            },
            10 => {
                // UNSUBSCRIBE
            },
            11 => {
                // UNSUBACK
            },
            12 => {
                // PINGREQ
            },
            13 => {
                // PINGRESP
            },
            14 => {
                // FUCK OFF! (DISCONNECT)
            },

            _ => {}


        }
        
    // CONNECT

    // SUBCRIBE
    
    // PUBLISH

    // PING

    // DISCONNECT
    }

    

}


fn main() {
    let listener = TcpListener::bind("192.168.50.23:1883").unwrap();

    
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            println!("New client connected");

            handle_connection(stream);
        });
    }
    
}
