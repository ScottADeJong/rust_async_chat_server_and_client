use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use tokio::net::TcpStream;

pub struct User {
    pub socket: Option<TcpStream>,
    pub nickname: Option<String>,
    pub address: String,
    pub is_active: bool
}
impl User {
    pub fn from(tcp_stream: TcpStream, address: Option<String>) -> Self {
        let address = match address {
            Some(address) => address,
            None => tcp_stream.local_addr().unwrap().to_string()
        };

        Self {
            socket: Some(tcp_stream),
            nickname: None,
            address,
            is_active: true
        }
    }

    pub async fn get_display_name(&self) -> String {
        match &self.nickname {
            Some(nick_name) => nick_name.clone(),
            None => self.address.clone()
        }
    }

    pub fn disconnect(&mut self) {
        self.is_active = false;
    }

    pub async fn set_nickname(&mut self, new_name: Option<String>) {
        match new_name {
            Some(string) => self.nickname = Some(string),
            None => self.nickname = None
        }
    }
}

struct MessageDTO {
    author: User,
    message_content: String,
    timestamp: DateTime<Utc>
}