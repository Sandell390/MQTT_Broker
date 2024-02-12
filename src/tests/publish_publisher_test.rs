#[cfg(test)]
mod tests {
    use crate::control_packet::publish::Response;
    use crate::models::client::Client;
    use crate::models::flags::ConnectFlags;
    use crate::models::publish_queue_item::PublishItemState;
    use crate::models::topic::Topic;
    use crate::{handle_qos_1_session, handle_qos_2_session};
    use std::net::SocketAddr;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_handle_qos_1_session() {
        let clients = Arc::new(Mutex::new(Vec::new()));

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
            tx.clone(),
            connect_flags,
        );

        clients.lock().unwrap().push(client);
        let topics = Arc::new(Mutex::new(vec![
            Topic {
                topic_name: "topic1".to_string(),
                retained_msg: (String::new(), 0),
                client_ids: vec![("client1".to_string(), 0)],
            },
            // Add more topics if needed...
        ]));
        let publish_queue = Arc::new(Mutex::new(Vec::new()));
        let response = Response {
            dup_flag: false,
            qos_level: 1,
            retain_flag: false,
            packet_id: 10,
            topic_name: "test".to_string(),
            payload_message: "test".to_string(),
            // Fill in the fields of the Response struct
            // ...
        };

        handle_qos_1_session(
            tx.clone(),
            response,
            clients.clone(),
            topics.clone(),
            publish_queue.clone(),
        );

        sleep(Duration::from_millis(100));
        // Check that the publish queue is not empty (since QoS is 2)
        let received_packet = rx.try_recv().expect("Failed to receive packet");
        assert!(received_packet.is_ok());

        let packet_puback = received_packet.unwrap();
        assert_eq!(
            packet_puback[0], 64,
            "First byte of a Puback packet should be 64"
        );
        assert_eq!(
            packet_puback[1], 2,
            "Second byte of a Puback packet should be 2"
        );
    }

    #[test]
    fn test_handle_qos_2_session() {
        let response = Response {
            dup_flag: false,
            qos_level: 2,
            retain_flag: false,
            packet_id: 10,
            topic_name: "test".to_string(),
            payload_message: "test".to_string(),
            // Fill in the fields of the Response struct
            // ...
        };
        let clients = Arc::new(Mutex::new(Vec::new()));
        let topics = Arc::new(Mutex::new(Vec::new()));
        let publish_queue = Arc::new(Mutex::new(Vec::new()));

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
            tx.clone(),
            connect_flags,
        );

        clients.lock().unwrap().push(client);

        handle_qos_2_session(tx.clone(), response, clients, topics, publish_queue.clone());

        sleep(Duration::from_millis(100));
        // Check that the publish queue is not empty (since QoS is 2)
        assert!(!publish_queue.lock().unwrap().is_empty());

        let received_packet = rx.try_recv();
        assert!(received_packet.is_ok());

        let expected_packet = vec![80, 2, 0, 10];

        assert_eq!(received_packet.unwrap(), Ok(expected_packet));
        assert_eq!(
            publish_queue.lock().unwrap()[0].state,
            PublishItemState::AwaitingPubrel
        );

        _ = publish_queue.lock().unwrap()[0]
            .tx
            .send(PublishItemState::PubrelRecieved);

        sleep(Duration::from_millis(1200));
        // Check that the publish queue is not empty after "broker" has received Pubrec

        // Check that the publish queue is empty after "broker" has received Pubcomp
        assert!(publish_queue.lock().unwrap().is_empty());
    }
}
