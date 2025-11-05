use tokio::sync::Mutex;
use tokio::net::TcpStream;

pub struct User {
    pub socket: Option<TcpStream>,
    pub nickname: Mutex<Option<String>>,
    pub address: String,
    pub is_active: Mutex<bool>
}
impl User {
    pub fn from(tcp_stream: TcpStream, address: Option<String>) -> Self {
        let address = match address {
            Some(address) => address,
            None => tcp_stream.local_addr().unwrap().to_string()
        };

        Self {
            socket: Some(tcp_stream),
            nickname: Mutex::new(None),
            address,
            is_active: Mutex::new(true)
        }
    }

    pub async fn get_display_name(&self) -> String {
        let nickname = self.nickname.lock().await;

        match &*nickname {
            Some(nick_name) => nick_name.clone(),
            None => self.address.clone()
        }
    }

    pub async fn disconnect(&mut self) {
        let mut is_active = self.is_active.lock().await;

        *is_active = false;
    }
}