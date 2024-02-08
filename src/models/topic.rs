use std::hash::{ Hash, Hasher };
#[derive(Debug, Clone)]
pub struct Topic {
    pub topic_name: String,
    pub retained_msg: (String, u8),
    pub client_ids: Vec<(String, u8)>,
}

impl Topic {
    pub fn new(topic_name: String) -> Topic {
        Topic {
            topic_name,
            retained_msg: (String::new(), 0),
            client_ids: Vec::new(),
        }
    }
}

impl Eq for Topic {}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.topic_name == other.topic_name
    }
}

impl Hash for Topic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.topic_name.hash(state);
    }
}
