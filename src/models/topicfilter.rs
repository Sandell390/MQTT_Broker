use std::hash::{ Hash, Hasher };

#[derive(Debug)]
pub struct Topicfilter{
    pub topic_name: String,
    pub qos: u8,
}

impl Eq for Topicfilter {}

impl PartialEq for Topicfilter {
    fn eq(&self, other: &Self) -> bool {
        self.topic_name == other.topic_name
    }
}

impl Hash for Topicfilter{
    fn hash<H: Hasher>(&self, state: &mut H){
        self.topic_name.hash(state);
    }
}