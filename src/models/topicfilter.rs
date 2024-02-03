use std::hash::{ Hash, Hasher };

#[derive(Debug)]
pub struct Topfilter{
    pub topic_name: String,
    pub qos: u8,
}

impl Eq for Topfilter {}

impl PartialEq for Topfilter {
    fn eq(&self, other: &Self) -> bool {
        self.topic_name == other.topic_name
    }
}

impl Hash for Topfilter{
    fn hash<H: Hasher>(&self, state: &mut H){
        self.topic_name.hash(state);
    }
}