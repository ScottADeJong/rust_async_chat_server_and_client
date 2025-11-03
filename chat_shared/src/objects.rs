use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::net::TcpStream;

pub struct User {
    pub socket: Option<Arc<&'static TcpStream>>,
    pub nickname: Option<String>,
    pub address: String
}
impl User {
    pub fn new_from_server(tcp_stream: Arc<&'static TcpStream>, address: String) -> Self {
        Self {
            socket: Some(tcp_stream),
            nickname: None,
            address
        }
    }

    pub fn new_from_client(address: String) -> Self {
        Self {
            socket: None,
            nickname: None,
            address
        }
    }
}

struct MessageDTO {
    author: User,
    message_content: String,
    timestamp: DateTime<Utc>
}