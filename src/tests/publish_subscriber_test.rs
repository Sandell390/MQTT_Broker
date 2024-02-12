#[cfg(test)]
mod tests {

    use std::net::SocketAddr;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::{Arc, Mutex};

    use std::thread::sleep;
    use std::time::Duration;
    use crate::common_fn;
    use crate::control_packet::publish::publish_to_client;
    use crate::models::client::Client;
    use crate::models::flags::ConnectFlags;
    use crate::models::publish_queue_item::PublishItemState;
    use crate::models::topic::Topic;

    #[test]
    fn test_publish_to_client() {
        let connect_flags = ConnectFlags {
            username_flag: true,
            password_flag: true,
            will_retain_flag: false,
            will_qos_flag: 1,
            will_flag: true,
            clean_session_flag: true,
        };

        let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        // Create a mock client with a receiver so we can check the messages sent to it
        let (tx, rx): (
            Sender<Result<Vec<u8>, String>>,
            Receiver<Result<Vec<u8>, String>>,
        ) = channel();
        let client = Client::new(
            "client_id".to_string(),
            "will_topic".to_string(),
            "will_message".to_string(),
            60,
            "username".to_string(),
            "password".to_string(),
            socket_addr,
            tx,
            connect_flags,
        );

        // Create a mock publish queue
        let publish_queue = Arc::new(Mutex::new(Vec::new()));

        // Create a new Topic
        let topic = Topic {
            topic_name: "test".to_string(),
            retained_msg: (String::new(), 0),
            client_ids: Vec::new(),
        };

        // Call the function with a test message and QoS level 0
        publish_to_client(
            &client,
            publish_queue.clone(),
            &topic,
            "test",
            &0,
            &false,
        );

        // Check that the publish queue is still empty (since QoS is 0)
        assert!(publish_queue.lock().unwrap().is_empty());

        // Check that the client received the correct packet
        // This will depend on how your publish_to_client function constructs the packet
        // Here's an example where we just check that the client received some packet
        let received_packet = rx.try_recv();
        assert!(received_packet.is_ok());

        // If your publish_to_client function constructs the packet in a specific way,
        // you can check the contents of the received packet here. For example:
        let expected_packet = vec![
            0b00110000, // Publish packet, QoS level 1, no retain
            0b00001010, // Remaining length (10 bytes)
            0b00000000, // Topic name MSB (0 bytes)
            0b00000100, // Topic name LSB (4 bytes)
            b't', b'e', b's', b't',       // Topic name ("test")
            b't', b'e', b's', b't', // Payload ("test")
        ];
        assert_eq!(received_packet.unwrap(), Ok(expected_packet));
    }

    #[test]
fn test_publish_to_client_qos_1() {
    let connect_flags = ConnectFlags {
        username_flag: true,
        password_flag: true,
        will_retain_flag: false,
        will_qos_flag: 1,
        will_flag: true,
        clean_session_flag: true,
    };

    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let (tx, rx): (
        Sender<Result<Vec<u8>, String>>,
        Receiver<Result<Vec<u8>, String>>,
    ) = channel();
    let client = Client::new(
        "client_id".to_string(),
        "will_topic".to_string(),
        "will_message".to_string(),
        60,
        "username".to_string(),
        "password".to_string(),
        socket_addr,
        tx,
        connect_flags,
    );

    let publish_queue = Arc::new(Mutex::new(Vec::new()));

    let topic = Topic {
        topic_name: "test".to_string(),
        retained_msg: (String::new(), 0),
        client_ids: Vec::new(),
    };

    publish_to_client(
        &client,
        publish_queue.clone(),
        &topic,
        "test",
        &1,
        &false,
    );


    sleep(Duration::from_millis(100));
    // Check that the publish queue is not empty (since QoS is 1)
    assert!(!publish_queue.lock().unwrap().is_empty());

    let received_packet = rx.try_recv();
    assert!(received_packet.is_ok());

    let packet_id = common_fn::msb_lsb_creater::split_into_msb_lsb(publish_queue.lock().unwrap()[0].packet_id);

    let mut expected_packet = vec![
        0b00110010, // Publish packet, QoS level 1, no retain
        0b00001100, // Remaining length (12 bytes)
        0b00000000, // Topic name MSB (0 bytes)
        0b00000100, // Topic name LSB (4 bytes)
        b't', b'e', b's', b't', // Topic name ("test")
        0b00000000, // Packet identifier MSB (0 bytes)
        0b00000001, // Packet identifier LSB (1 byte)
        b't', b'e', b's', b't', // Payload ("test")
    ];

    expected_packet[8] = packet_id[0];
    expected_packet[9] = packet_id[1];


    assert_eq!(received_packet.unwrap(), Ok(expected_packet));

    _ = publish_queue.lock().unwrap()[0].tx.send(PublishItemState::PubackRecieved);


    sleep(Duration::from_millis(1000));
    // Check that the publish queue is empty after "broker" has recieved Puback
    assert!(publish_queue.lock().unwrap().is_empty());

}

#[test]
fn test_publish_to_client_qos_2() {
    let connect_flags = ConnectFlags {
        username_flag: true,
        password_flag: true,
        will_retain_flag: false,
        will_qos_flag: 2,
        will_flag: true,
        clean_session_flag: true,
    };

    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let (tx, rx): (
        Sender<Result<Vec<u8>, String>>,
        Receiver<Result<Vec<u8>, String>>,
    ) = channel();
    let client = Client::new(
        "client_id".to_string(),
        "will_topic".to_string(),
        "will_message".to_string(),
        60,
        "username".to_string(),
        "password".to_string(),
        socket_addr,
        tx,
        connect_flags,
    );

    let publish_queue = Arc::new(Mutex::new(Vec::new()));

    let topic = Topic {
        topic_name: "test".to_string(),
        retained_msg: (String::new(), 0),
        client_ids: Vec::new(),
    };

    publish_to_client(
        &client,
        publish_queue.clone(),
        &topic,
        "test",
        &2,
        &false,
    );

    sleep(Duration::from_millis(100));
    // Check that the publish queue is not empty (since QoS is 2)
    assert!(!publish_queue.lock().unwrap().is_empty());

    let received_packet = rx.try_recv();
    assert!(received_packet.is_ok());

    let packet_id = common_fn::msb_lsb_creater::split_into_msb_lsb(publish_queue.lock().unwrap()[0].packet_id);

    let mut expected_packet = vec![
        0b00110100, // Publish packet, QoS level 2, no retain
        0b00001100, // Remaining length (12 bytes)
        0b00000000, // Topic name MSB (0 bytes)
        0b00000100, // Topic name LSB (4 bytes)
        b't', b'e', b's', b't', // Topic name ("test")
        0b00000000, // Packet identifier MSB (0 bytes)
        0b00000001, // Packet identifier LSB (1 byte)
        b't', b'e', b's', b't', // Payload ("test")
    ];

    expected_packet[8] = packet_id[0];
    expected_packet[9] = packet_id[1];

    assert_eq!(received_packet.unwrap(), Ok(expected_packet));

    _ = publish_queue.lock().unwrap()[0].tx.send(PublishItemState::PubrecRecieved);

    sleep(Duration::from_millis(1200));
    // Check that the publish queue is not empty after "broker" has received Pubrec
    assert_eq!(publish_queue.lock().unwrap()[0].state, PublishItemState::PubrecRecieved);

    _ = publish_queue.lock().unwrap()[0].tx.send(PublishItemState::PubcompRecieved);

    sleep(Duration::from_millis(1000));
    // Check that the publish queue is empty after "broker" has received Pubcomp
    assert!(publish_queue.lock().unwrap().is_empty());
}
}
