use local_ip_address::local_ip;
use std::io::{ Read, Write };
use std::net::{ SocketAddr, TcpListener, TcpStream };
use std::sync::mpsc::{ channel, Receiver, Sender };
use std::sync::{ Arc, Mutex, MutexGuard };
use std::thread;
use std::time::{ Duration, Instant };

use crate::models::client::Client;
use crate::models::publish_queue_item::{ PublishItemDirection, PublishItemState, PublishQueueItem };
use crate::models::topic::Topic;
use crate::models::text_formatter::{ Color, Reset, Style };

mod common_fn;
mod control_packet;
mod models;
mod tests;

/// Entry point of the MQTT broker application.
///
/// # Description
///
/// This function serves as the entry point of the MQTT broker application.
/// It initializes the TCP listener bound to port 1883, creates mutex-protected vectors for topics and clients,
/// and continuously listens for incoming client connections.
///
/// For each incoming connection, it spawns a new thread
/// to handle the client connection using the `handle_connection` function.
///
/// # Features to consider, i another afsnit of the mqtt kalender
/// - Verbose debugging. Would be helpful, to sometimes be able to see what happens?
/// - A logger, to go back and review errors.
/// - Better utilisation of PublishQueueItem and it's states
fn main() {
    // Fetch current ip
    let my_local_ip: std::net::IpAddr = local_ip().unwrap();

    // Create a TCP listener bound to port 1883 (the default MQTT port)
    let listener: TcpListener = TcpListener::bind(SocketAddr::new(my_local_ip, 1883)).expect(
        "Failed to bind to port 1883"
    );

    // Print a message indicating that the MQTT broker is listening
    println!(
        "{1}Success! -> {2}{3}MQTT broker listening on {0}:1883{2}",
        my_local_ip,
        Color::LimeGreen,
        Reset::All,
        Style::Italic
    );

    // Create a mutex-protected topics Vector
    let topics: Arc<Mutex<Vec<Topic>>> = Arc::new(Mutex::new(Vec::new()));

    // Create a mutex-protected clients vector
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    // Create a mutex-protected publish_queue
    let publish_queue: Arc<Mutex<Vec<PublishQueueItem>>> = Arc::new(Mutex::new(Vec::new()));

    // For each incoming connection -> Spawn a new thread to handle the client's connection, using the handle_connection function
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone Lists for each thread
                let clients_clone: Arc<Mutex<Vec<Client>>> = Arc::clone(&clients);
                let topics_clone: Arc<Mutex<Vec<Topic>>> = Arc::clone(&topics);
                let publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>> = Arc::clone(
                    &publish_queue
                );

                // Spawn a new thread to handle the client connection
                thread::spawn(move || {
                    // Handle the client connection
                    handle_connection(stream, clients_clone, topics_clone, publish_queue_clone);
                });
            }
            Err(err) => {
                // Print error if accepting a client connection fails
                println!(
                    "{1}Error! -> {2}{3}Could not accept client connection: {0:?}{4}",
                    err,
                    Color::BrightRed,
                    Reset::All,
                    Style::Italic,
                    Reset::All
                );
            }
        }
    }

    print!("{}", Reset::Default);
}

/// Handles the connection with a client, continuously reading data from the client
/// and processing incoming packets according to the MQTT protocol.
///
/// # Arguments
///
/// * `stream` - A mutable reference to a TCP stream representing the connection with the client.
/// * `clients` - An Arc-wrapped Mutex-protected vector of clients currently connected to the server.
/// * `topics` - An Arc-wrapped Mutex-protected vector of topics subscribed to by clients.
///
/// # Description
///
/// This function processes incoming packets from the client according to the MQTT protocol.
/// It continuously reads data from the client, interprets packet types, and handles them appropriately.
/// Depending on the packet type, it performs actions such as establishing connections,
/// publishing messages, subscribing to topics, unsubscribing, responding to PING requests, and disconnecting.
///
/// # Notes
///
/// This function spawns a separate thread to handle message transmission to the client
/// and ensures that each client's connection and disconnection are logged.
/// It also prints information about connected clients for debugging purposes.
/// Furthermore it creates a channel (Transmit and Recieve), to handle communication between threads.
fn handle_connection(
    mut stream: TcpStream,
    clients: Arc<Mutex<Vec<Client>>>,
    topics: Arc<Mutex<Vec<Topic>>>,
    publish_queue: Arc<Mutex<Vec<PublishQueueItem>>>
) {
    // Creates a new asynchronous channel, returning the sender/receiver halves.
    // All data sent on the Sender will become available on the Receiver, also across threads.
    let (tx, rx): (Sender<Result<Vec<u8>, String>>, Receiver<Result<Vec<u8>, String>>) = channel();

    // Copy the stream
    let mut stream_clone: TcpStream = stream.try_clone().unwrap();

    // Write thread
    thread::spawn(move || {
        for message in rx {
            match message {
                Ok(response) => {
                    // Sends the message to the client
                    _ = stream_clone.write(response.as_slice());
                    _ = stream_clone.flush();
                }
                Err(err) => {
                    println!(
                        "{1}Stream Error! -> {2}{3}{0}{4}",
                        err,
                        Color::BrightRed,
                        Reset::All,
                        Style::Italic,
                        Reset::All
                    );

                    _ = stream_clone.shutdown(std::net::Shutdown::Both);
                    break;
                }
            }
        }
    });

    let socket_addr: SocketAddr = stream.peer_addr().unwrap();
    _ = stream.set_read_timeout(Some(Duration::from_secs(0)));

    // Print client connection information
    println!(
        "{1}Success! -> {2}{3}Client connected: {0}{2}",
        socket_addr,
        Color::LimeGreen,
        Reset::All,
        Style::Italic
    );

    let mut has_first_packet_arrived: bool = false;
    let mut discard_will_msg: bool = false;

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

                println!("{:?}", &buffer[..packet_length]);
                println!("{:?}", packet_type);

                // Match for incoming packets
                match packet_type {
                    1 => {
                        // Connect
                        if !has_first_packet_arrived {
                            // Access the clients vector within the mutex
                            let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            match
                                control_packet::connect::handle(
                                    buffer,
                                    packet_length,
                                    socket_addr,
                                    &mut clients,
                                    tx.clone()
                                )
                            {
                                Ok(response) => {
                                    let keep_alive: u64 = response.keep_alive;
                                    // Continue with handling the connection
                                    // Send response to the client
                                    _ = tx.send(Ok(response.return_packet.to_vec()));

                                    // Set keep_alive
                                    _ = stream.set_read_timeout(
                                        Some(Duration::from_secs(keep_alive))
                                    );
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
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
                            match control_packet::publish::handle_publish(buffer, packet_length) {
                                Ok(response) => {
                                    // Clone clients for each thread
                                    let clients_clone: Arc<Mutex<Vec<Client>>> = Arc::clone(
                                        &clients
                                    );

                                    // Clone topic for each thread
                                    let topics_clone: Arc<Mutex<Vec<Topic>>> = Arc::clone(&topics);

                                    // Clone publish_queue
                                    let publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>> =
                                        Arc::clone(&publish_queue);

                                    // Access the clients vector within the mutex
                                    let mut clients: MutexGuard<'_, Vec<Client>> = clients
                                        .lock()
                                        .unwrap();

                                    // Access the topics vector within the mutex
                                    let mut topics: MutexGuard<'_, Vec<Topic>> = topics
                                        .lock()
                                        .unwrap();

                                    // Check QoS
                                    match response.qos_level {
                                        0 => {
                                            if response.dup_flag {
                                                break;
                                            }

                                            // Publish to subscribers
                                            control_packet::publish::publish(
                                                &mut topics,
                                                &mut clients,
                                                publish_queue_clone,
                                                &response.topic_name,
                                                &response.payload_message,
                                                &false,
                                                &response.qos_level,
                                                &false
                                            );
                                        }
                                        1 => {
                                            handle_qos_1_session(
                                                tx.clone(),
                                                response.clone(),
                                                clients_clone,
                                                topics_clone,
                                                publish_queue_clone
                                            );
                                        }
                                        2 => {
                                            handle_qos_2_session(
                                                tx.clone(),
                                                response.clone(),
                                                clients_clone,
                                                topics_clone,
                                                publish_queue_clone
                                            );
                                        }
                                        _ => {
                                            break;
                                        }
                                    }

                                    // If response.retain_flag is set
                                    if response.retain_flag {
                                        // Check if topic already exists, else push the new topic with retain message
                                        if
                                            let Some(index) = topics
                                                .iter()
                                                .position(|t: &Topic| {
                                                    t.topic_name == response.topic_name
                                                })
                                        {
                                            topics[index].retained_msg = (
                                                response.payload_message,
                                                response.qos_level,
                                            );
                                        } else {
                                            let mut new_topic: Topic = Topic::new(
                                                response.topic_name
                                            );
                                            new_topic.retained_msg = (
                                                response.payload_message,
                                                response.qos_level,
                                            );
                                            topics.push(new_topic);
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    4 => {
                        // PUBACK
                        if has_first_packet_arrived {
                            match control_packet::publish::handle_puback(buffer, packet_length) {
                                Ok(response) => {
                                    // Access the publish queue within mutex
                                    let publish_queue: MutexGuard<
                                        '_,
                                        Vec<PublishQueueItem>
                                    > = publish_queue.lock().unwrap();

                                    // Finds the index of the publish queue item that matches the incoming packet id
                                    if
                                        let Some(index) = publish_queue
                                            .iter()
                                            .position(|item: &PublishQueueItem| {
                                                item.packet_id == response
                                            })
                                    {
                                        // Sends Puback state to the publish queue item receiver
                                        _ = publish_queue[index].tx.send(
                                            PublishItemState::PubackRecieved
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    5 => {
                        // PUBREC
                        if has_first_packet_arrived {
                            match control_packet::publish::handle_pubrec(buffer, packet_length) {
                                Ok(response) => {
                                    // Access the publish queue within mutex
                                    let publish_queue: MutexGuard<
                                        '_,
                                        Vec<PublishQueueItem>
                                    > = publish_queue.lock().unwrap();

                                    // Finds the index of the publish queue item that matches the incoming packet id
                                    if
                                        let Some(index) = publish_queue
                                            .iter()
                                            .position(|item: &PublishQueueItem| {
                                                item.packet_id == response
                                            })
                                    {
                                        // Sends Puback state to the publish queue item receiver
                                        _ = publish_queue[index].tx.send(
                                            PublishItemState::PubrecRecieved
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    6 => {
                        // PUBREL
                        if has_first_packet_arrived {
                            match control_packet::publish::handle_pubrel(buffer, packet_length) {
                                Ok(response) => {
                                    // Access the publish queue within mutex
                                    let publish_queue: MutexGuard<
                                        '_,
                                        Vec<PublishQueueItem>
                                    > = publish_queue.lock().unwrap();

                                    // Finds the index of the publish queue item that matches the incoming packet id
                                    if
                                        let Some(index) = publish_queue
                                            .iter()
                                            .position(|item: &PublishQueueItem| {
                                                item.packet_id == response
                                            })
                                    {
                                        // Sends Puback state to the publish queue item receiver
                                        _ = publish_queue[index].tx.send(
                                            PublishItemState::PubrelRecieved
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    7 => {
                        // PUBCOMP
                        if has_first_packet_arrived {
                            match control_packet::publish::handle_pubcomp(buffer, packet_length) {
                                Ok(response) => {
                                    // Access the publish queue within mutex
                                    let publish_queue: MutexGuard<
                                        '_,
                                        Vec<PublishQueueItem>
                                    > = publish_queue.lock().unwrap();

                                    // Finds the index of the publish queue item that matches the incoming packet id
                                    if
                                        let Some(index) = publish_queue
                                            .iter()
                                            .position(|item: &PublishQueueItem| {
                                                item.packet_id == response
                                            })
                                    {
                                        // Sends Puback state to the publish queue item receiver
                                        _ = publish_queue[index].tx.send(
                                            PublishItemState::PubcompRecieved
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    8 => {
                        // SUBSCRIBE
                        if has_first_packet_arrived {
                            // Access the topic Vector
                            match control_packet::subcribe::handle(&buffer, packet_length) {
                                Ok(sub_packet) => {
                                    // Sends suback to the client
                                    _ = tx.send(Ok(sub_packet.return_packet));

                                    {
                                        // Access the clients vector within the mutex
                                        let clients: MutexGuard<'_, Vec<Client>> = clients
                                            .lock()
                                            .unwrap();

                                        // Finds the client that matches the socket_addr so we can add the client to the topic list
                                        if
                                            let Some(index) = clients
                                                .iter()
                                                .position(|c: &Client| c.socket_addr == socket_addr)
                                        {
                                            // Adding topic filters to the client
                                            for topicfilter in sub_packet.topic_qos_pair {
                                                // Access the topics list within mutex
                                                let mut topics: MutexGuard<'_, Vec<Topic>> = topics
                                                    .lock()
                                                    .unwrap();

                                                // Adds the client to the topic list
                                                add_client_to_topic_list(
                                                    &mut topics,
                                                    clients[index].id.clone(),
                                                    topicfilter.clone()
                                                );
                                                let client_clone: Client = clients[index].clone();

                                                // Finds the index of the topic that the client wants to subscribe on
                                                // And send a publish message if the topic have a retained message
                                                if
                                                    let Some(index) = topics
                                                        .iter()
                                                        .position(
                                                            |t: &Topic|
                                                                t.topic_name == topicfilter.0
                                                        )
                                                {
                                                    if
                                                        topics[index].retained_msg.0 !=
                                                        "".to_string()
                                                    {
                                                        let message: &str =
                                                            &topics[index].retained_msg.0.clone();
                                                        control_packet::publish::publish_to_client(
                                                            &client_clone,
                                                            Arc::clone(&publish_queue),
                                                            &topics[index],
                                                            message,
                                                            &topicfilter.1,
                                                            &true
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
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
                            // Access the clients vector within the mutex
                            let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

                            match control_packet::unsubcribe::handle(&buffer, packet_length) {
                                Ok(unsub_packet) => {
                                    // Finds the client that matches the socket_addr so we can remove the client from the topic list
                                    if
                                        let Some(index) = clients
                                            .iter()
                                            .position(|c: &Client| c.socket_addr == socket_addr)
                                    {
                                        // Access the topic Vector
                                        let mut topics: MutexGuard<'_, Vec<Topic>> = topics
                                            .lock()
                                            .unwrap();

                                        // Removing the client from the topic list
                                        for topic_name in unsub_packet.topic_qos_pair {
                                            remove_client_from_topic_list(
                                                &mut topics,
                                                clients[index].id.clone(),
                                                topic_name
                                            );
                                        }
                                    }

                                    // Sends an unsuback
                                    _ = tx.send(Ok(unsub_packet.return_packet));
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                    break;
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    12 => {
                        // PINGREQ
                        if has_first_packet_arrived {
                            match control_packet::ping::handle(buffer, packet_length) {
                                Ok(return_packet) => {
                                    // Send response to the client
                                    _ = tx.send(Ok(return_packet.to_vec()));
                                }
                                Err(err) => {
                                    println!(
                                        "{1}Error! -> {2}{3}{0}{4}",
                                        err,
                                        Color::BrightRed,
                                        Reset::All,
                                        Style::Italic,
                                        Reset::All
                                    );
                                }
                            }
                        } else {
                            // Disconnect
                            break;
                        }
                    }
                    14 => {
                        // Disconnect
                        if has_first_packet_arrived {
                            // Validate reserved bits are not set
                            match control_packet::disconnect::handle(&buffer, packet_length) {
                                Ok(_response) => {
                                    discard_will_msg = true;
                                }
                                Err(_err) => {
                                    discard_will_msg = true;
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
                println!(
                    "{1}Error! -> {2}{3}{0}\nClosing the Stream{4}",
                    err,
                    Color::BrightRed,
                    Reset::All,
                    Style::Italic,
                    Reset::All
                );
                break;
            }
        }

        has_first_packet_arrived = true;
    }

    // Access the clients vector within the mutex
    let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();

    // Access the topics vector within the mutex
    let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();

    disconnect_client_by_socket_addr(
        &mut topics,
        &mut clients,
        publish_queue,
        socket_addr,
        discard_will_msg
    );

    println!(
        "{1}Success! -> {2}{3}Client disconnected: {0}{2}",
        socket_addr,
        Color::LimeGreen,
        Reset::All,
        Style::Italic
    );

    // Sends an error to the Write thread so it can stop the thread and closes the connection
    _ = tx.send(Err("Close Stream".to_string()));

    _ = stream.shutdown(std::net::Shutdown::Both);
}

/// Disconnects a client based on its socket address and performs cleanup tasks.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `clients` - A mutable reference to a vector of clients.
/// * `socket_addr` - The socket address of the client to be disconnected.
/// * `discard_will_msg` - A boolean indicating whether to discard the client's will message.
///
/// # Description
///
/// This function disconnects a client based on its socket address and performs the following tasks:
/// - Publishes the will message to clients that have subscribed to the will topic.
/// - Calls the `handle_disconnect` method on the client.
/// - Optionally discards the client's will message if specified.
/// - Re-adds the updated client to the list of clients.
///
/// # Examples
/// ```
/// // Access the clients vector within the mutex
/// let mut clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
///
/// // Access the topics vector within the mutex
/// let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
///
/// // Obtain the socket address
/// let socket_addr: SocketAddr = stream.peer_addr().unwrap();
///
/// disconnect_client_by_socket_addr(&mut topics, &mut clients, socket_addr, false);
/// ```
fn disconnect_client_by_socket_addr(
    topics: &mut Vec<Topic>,
    clients: &mut Vec<Client>,
    publish_queue: Arc<Mutex<Vec<PublishQueueItem>>>,
    socket_addr: SocketAddr,
    discard_will_msg: bool
) {
    if let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr) {
        // Extract the client from the list
        let mut client: Client = clients.remove(index);

        // Publish the will message to clients that have subscribed on the will topic
        control_packet::publish::publish(
            topics,
            clients,
            publish_queue,
            &client.will_topic,
            &client.will_message,
            &false,
            &client.connect_flags.will_qos_flag,
            &false
        );

        // Call handle_disconnect on the client
        client.handle_disconnect();

        if discard_will_msg {
            client.will_message = String::new();
        }

        // Re-add the updated client to the list
        clients.push(client);
    }
}

/// Adds a client and its associated topic filter to the list of topics.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `client_id` - The ID of the client to be added.
/// * `topic_filter` - A tuple containing the topic name and quality of service (QoS) level.
///
/// # Description
///
/// This function adds a client and its associated topic filter to the list of topics.
/// It searches for the specified topic within the topics vector and either adds the client
/// to the existing topic or creates a new topic if the specified topic does not exist.
///
/// # Examples
///
/// ```
/// // Access the clients vector within the mutex
/// let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
/// // Access the topic Vector
/// let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
///
/// match control_packet::subcribe::handle(buffer, packet_length) {
///     Ok(sub_packet) => {
///         if
///             let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr)
///         {
///             // Adding topic filters to the client
///             for topicfilter in sub_packet.topic_qos_pair {
///                 //clients[index].add_subscription(topicfilter);
///                 add_client_to_topic_list(&mut topics, clients[index].id.clone(), topicfilter);
///             }
///         }
///         _ = tx.send(sub_packet.return_packet);
///     }
/// ```
fn add_client_to_topic_list(topics: &mut Vec<Topic>, client_id: String, topic: (String, u8)) {
    // If the topic exist then we add the client to the topic list
    // If not, then creates a new topic and puts the client in
    if let Some(index) = topics.iter().position(|t: &Topic| t.topic_name == topic.0) {
        // If the topic exists, see if we have a client subscribed on that topic
        // And update (remove and then add) the client
        if
            let Some(index_client_id) = topics[index].client_ids
                .iter()
                .position(|t: &(String, u8)| t.0 == client_id)
        {
            // Remove the client
            topics[index].client_ids.remove(index_client_id);
        }

        // Add the client
        topics[index].client_ids.push((client_id, topic.1));
    } else {
        // Create a new topic
        let mut new_topic: Topic = Topic::new(topic.0);
        new_topic.client_ids.push((client_id, topic.1));
        topics.push(new_topic);
    }
}

/// Removes a client from being subcribed to a specific topic, from the list of topics.
///
/// # Arguments
///
/// * `topics` - A mutable reference to a vector of topics.
/// * `client_id` - The ID of the client whose topic filter is to be removed.
/// * `topic_filter` - A tuple containing the topic name and quality of service (QoS) level.
///
/// # Description
///
/// This function removes a specific topic filter associated with a client from the list of topics.
/// It searches for the specified topic filter within the topics vector and removes it
/// from the client IDs associated with that topic, if found.
///
/// # Examples
///
/// ```
/// // Access the clients vector within the mutex
/// let clients: MutexGuard<'_, Vec<Client>> = clients.lock().unwrap();
/// match control_packet::unsubcribe::handle(buffer, packet_length) {
///     Ok(unsub_packet) => {
///         if
///             let Some(index) = clients.iter().position(|c: &Client| c.socket_addr == socket_addr)
///         {
///             // Access the topic Vector
///             let mut topics: MutexGuard<'_, Vec<Topic>> = topics.lock().unwrap();
///             // Removing topic filters to the client
///             for topicfilter in unsub_packet.topic_qos_pair {
///                 remove_client_topic_list(&mut topics, clients[index].id.clone(), topicfilter);
///             }
///         }
///
///         _ = tx.send(unsub_packet.return_packet);
///     }
/// ```
fn remove_client_from_topic_list(topics: &mut Vec<Topic>, client_id: String, topic: (String, u8)) {
    // Removes the client from the topic
    if let Some(index) = topics.iter().position(|t: &Topic| t.topic_name == topic.0) {
        if
            let Some(client_index) = topics[index].client_ids
                .iter()
                .position(|c: &(String, u8)| &c.0 == &client_id)
        {
            topics[index].client_ids.remove(client_index);
        }
    }
}

fn handle_qos_2_session(
    tx: Sender<Result<Vec<u8>, String>>,
    response: control_packet::publish::Response,
    clients_clone: Arc<Mutex<Vec<Client>>>,
    topics_clone: Arc<Mutex<Vec<Topic>>>,
    publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>>
) {
    // Clone the client sender to be used in the publish thread
    let publish_tx_clone: Sender<Result<Vec<u8>, String>> = tx.clone();

    // Clone the respone object from handle_publish
    let response_clone: control_packet::publish::Response = response.clone();

    // Creates the channels so the "main client" thread can send the QoS packets to the publisher QoS Session
    let (tx_qos, rx_qos): (Sender<PublishItemState>, Receiver<PublishItemState>) = channel();

    // QoS Session thread
    thread::spawn(move || {
        // Access the clients vector within the mutex
        let mut clients: MutexGuard<'_, Vec<Client>> = clients_clone.lock().unwrap();

        // Access the topics vector within the mutex
        let mut topics: MutexGuard<'_, Vec<Topic>> = topics_clone.lock().unwrap();

        // Save packet_id
        let packet_id: usize = response_clone.packet_id;

        // Creates another clone of publish so it can be used more times
        let publish_queue_clone_clone: Arc<Mutex<Vec<PublishQueueItem>>> = Arc::clone(
            &publish_queue_clone
        );
        // Checks if the packet id is already used with a Publish Item
        {
            // Access the publish queue within the mutex
            let publish_queue: MutexGuard<'_, Vec<PublishQueueItem>> = publish_queue_clone
                .lock()
                .unwrap();

            // If the packet id already is in the publish queue then sends another pubrec
            if
                let Some(_index) = publish_queue
                    .iter()
                    .position(|queue_item: &PublishQueueItem| { queue_item.packet_id == packet_id })
            {
                // Send pubrec to client (publisher)
                let mut pubrec_packet: Vec<u8> = vec![80, 2];

                pubrec_packet.append(
                    common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut()
                );

                _ = publish_tx_clone.send(Ok(pubrec_packet));
            } else {
                // If the packet id is not used then we can send a publish to the subscribers
                drop(publish_queue);

                // Publish to subscribers with dup 0
                control_packet::publish::publish(
                    &mut topics,
                    &mut clients,
                    publish_queue_clone,
                    &response_clone.topic_name,
                    &response_clone.payload_message,
                    &false,
                    &response_clone.qos_level,
                    &false
                );

                // Send pubrec to client (publisher)
                let mut pubrec_packet: Vec<u8> = vec![80, 2];

                pubrec_packet.append(
                    common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut()
                );
                _ = publish_tx_clone.send(Ok(pubrec_packet));

                // Push new publish queue item to the list
                {
                    let mut publish_queue: MutexGuard<
                        '_,
                        Vec<PublishQueueItem>
                    > = publish_queue_clone_clone.lock().unwrap();

                    publish_queue.push(PublishQueueItem {
                        tx: tx_qos,
                        packet_id,
                        timestamp_sent: Instant::now(),
                        publish_packet: vec![],
                        state: PublishItemState::AwaitingPubrel,
                        qos_level: 2,
                        flow_direction: PublishItemDirection::ToSubscriber,
                    });
                }
            }
        }

        // Loops until we have received Pubrel packet
        let has_recieved_pubrel: bool = false;
        'pubrel: while !has_recieved_pubrel {
            for _i in 0..2220 {
                // If there is something in the QoS receiver then we can contiune with the flow
                match rx_qos.try_recv() {
                    Ok(state) => {
                        if state == PublishItemState::PubrelRecieved {
                            // Access the publish queue
                            let mut publish_queue: MutexGuard<
                                '_,
                                Vec<PublishQueueItem>
                            > = publish_queue_clone_clone.lock().unwrap();

                            // Finds the index of publish queue item that match with packet id
                            if
                                let Some(index) = publish_queue
                                    .iter()
                                    .position(|t: &PublishQueueItem| { t.packet_id == packet_id })
                            {
                                // Send Pubcomp
                                let mut pubcomp_packet: Vec<u8> = vec![112, 2];
                                pubcomp_packet.append(
                                    common_fn::msb_lsb_creater
                                        ::split_into_msb_lsb(packet_id)
                                        .to_vec()
                                        .as_mut()
                                );
                                _ = publish_tx_clone.send(Ok(pubcomp_packet));

                                // Removes the publish queue item from the queue
                                publish_queue.remove(index);
                            }

                            break 'pubrel;
                        }
                    }
                    Err(_) => {}
                }

                thread::sleep(Duration::from_millis(100));
            }

            // Sends pubrec again if we have not received pubrel from the client
            let mut pubrec_packet: Vec<u8> = vec![80, 2];

            pubrec_packet.append(
                common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut()
            );
            _ = publish_tx_clone.send(Ok(pubrec_packet));
        }
    });
}

fn handle_qos_1_session(
    tx: Sender<Result<Vec<u8>, String>>,
    response: control_packet::publish::Response,
    clients_clone: Arc<Mutex<Vec<Client>>>,
    topics_clone: Arc<Mutex<Vec<Topic>>>,
    publish_queue_clone: Arc<Mutex<Vec<PublishQueueItem>>>
) {
    // Clone the client sender to be used in the publish thread
    let publish_tx_clone: Sender<Result<Vec<u8>, String>> = tx.clone();

    // Clone the response object from handle_publish
    let response_clone: control_packet::publish::Response = response.clone();

    thread::spawn(move || {
        // Access the clients vector within the mutex
        let mut clients: MutexGuard<'_, Vec<Client>> = clients_clone.lock().unwrap();

        // Access the topics vector within the mutex
        let mut topics: MutexGuard<'_, Vec<Topic>> = topics_clone.lock().unwrap();

        // Store the packet id
        let packet_id: usize = response_clone.packet_id;

        // Publish to subscribers with dup 0
        control_packet::publish::publish(
            &mut topics,
            &mut clients,
            publish_queue_clone,
            &response_clone.topic_name,
            &response_clone.payload_message,
            &false,
            &response_clone.qos_level,
            &false
        );

        // Send Puback packet
        let mut puback_packet: Vec<u8> = vec![64, 2];
        puback_packet.append(
            common_fn::msb_lsb_creater::split_into_msb_lsb(packet_id).to_vec().as_mut()
        );

        _ = publish_tx_clone.send(Ok(puback_packet));
    });
}
