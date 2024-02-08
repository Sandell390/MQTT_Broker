use std::hash::{ Hash, Hasher };

#[derive(Debug, Clone)]
pub struct ConnectFlags {
    pub clean_session_flag: bool,
    pub will_flag: bool,
    pub will_qos_flag: u8,
    pub will_retain_flag: bool,
    pub password_flag: bool,
    pub username_flag: bool,
}

impl Eq for ConnectFlags {}

impl PartialEq for ConnectFlags {
    fn eq(&self, other: &Self) -> bool {
        self.clean_session_flag == other.clean_session_flag &&
            self.will_flag == other.will_flag &&
            self.will_qos_flag == other.will_qos_flag &&
            self.will_retain_flag == other.will_retain_flag &&
            self.password_flag == other.password_flag &&
            self.username_flag == other.username_flag
    }
}

impl Hash for ConnectFlags {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.clean_session_flag.hash(state);
        self.will_flag.hash(state);
        self.will_qos_flag.hash(state);
        self.will_retain_flag.hash(state);
        self.password_flag.hash(state);
        self.username_flag.hash(state);
    }
}

impl ConnectFlags {
    // Constructor for creating a new ConnectFlags instance
    pub fn new(
        clean_session_flag: bool,
        will_flag: bool,
        will_qos_flag: u8,
        will_retain_flag: bool,
        password_flag: bool,
        username_flag: bool
    ) -> ConnectFlags {
        ConnectFlags {
            clean_session_flag,
            will_flag,
            will_qos_flag,
            will_retain_flag,
            password_flag,
            username_flag,
        }
    }
}
