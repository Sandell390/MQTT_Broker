use crate::models::client::Client;

pub struct Response {
    pub return_packet: [u8; 4],
    pub client: Client,
}

// pub fn validate(buffer: [u8; 8192], bytes_read: usize) -> Result<Response, &'static str> {
//     // Code goes here
// }
