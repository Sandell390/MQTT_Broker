use std::collections::HashSet;
use std::hash::{ Hash, Hasher };
use std::net::SocketAddr;

use super::flags::ConnectFlags;

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub will_topic: String,
    pub will_message: String,
    pub is_connected: bool,
    pub subscriptions: HashSet<String>,
    pub keep_alive: usize,
    pub username: String,
    pub password: String,
    pub socket_addr: SocketAddr,
    pub flags: ConnectFlags,
}

// Implement Eq, PartialEq, and Hash for the Client struct
impl Eq for Client {}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        // Implement PartialEq based on field comparisons
        self.id == other.id &&
            self.will_topic == other.will_topic &&
            self.will_message == other.will_message &&
            self.is_connected == other.is_connected &&
            self.subscriptions == other.subscriptions &&
            self.keep_alive == other.keep_alive &&
            self.username == other.username &&
            self.password == other.password &&
            self.socket_addr == other.socket_addr &&
            self.flags == other.flags
    }
}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Combine hashes of all fields
        self.id.hash(state);
        self.will_topic.hash(state);
        self.will_message.hash(state);
        self.is_connected.hash(state);

        // Use hash_combine for sets (HashSet)
        for subscription in &self.subscriptions {
            subscription.hash(state);
        }

        self.keep_alive.hash(state);
        self.username.hash(state);
        self.password.hash(state);
        self.socket_addr.hash(state);
        self.flags.hash(state);
    }
}

impl Client {
    // Constructor for creating a new client session
    pub fn new(
        client_id: String,
        will_topic: String,
        will_message: String,
        keep_alive: usize,
        username: String,
        password: String,
        socket_addr: SocketAddr,
        flags: ConnectFlags
    ) -> Client {
        Client {
            id: client_id,
            will_topic,
            will_message,
            is_connected: true,
            subscriptions: HashSet::new(),
            keep_alive,
            username,
            password,
            socket_addr,
            flags,
        }
    }

    // // Method for adding a subscription
    // pub fn add_subscription(&mut self, topic_filter: &str) {
    //     // Implement code for handling a new subscription, and putting it into the client's subscription list
    //     self.subscriptions.insert(topic_filter.to_string());
    // }

    // // Method for removing a subscription
    // pub fn remove_subscription(&mut self, topic_filter: &str) {
    //     // Implement code for removing a subscription from the client's subscription list
    //     self.subscriptions.remove(topic_filter);
    // }

    // // Method for handling will topic to publish on when the client disconnects
    // pub fn handle_will_topic(&self, topic: &str, payload: &[u8]) {
    //     // Implement will topic handling here
    //     println!("Received message on topic '{}': {:?}", topic, payload);
    // }

    // // Method for handling a will message to be published when the client disconnects
    // pub fn handle_will_message(&self, message: &str, payload: &[u8]) {
    //     // Implement will message handling here
    //     println!("Received message on topic '{}': {:?}", message, payload);
    // }

    // // Method for handling client connection
    // pub fn handle_connect(mut self) -> Self {
    //     println!("Client '{}' connected", self.id);

    //     // Update is_connected, to reflect connection state
    //     self.is_connected = true;

    //     return self;
    // }

    // Method for handling client disconnection
    pub fn handle_disconnect(&mut self) {
        println!("Client '{}' disconnected", self.id);

        // Update is_connected, to reflect connection state
        self.is_connected = false;
    }
}
