use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents a user in a networked system, containing information related to their connection,
/// identifier, and activity status.
///
/// # Fields
/// - `socket`:
///   An optional `TcpStream` that represents the user's network connection.
///   If `None`, the user is not currently connected.
/// - `nickname`:
///   A `Mutex`-protected optional `String` that contains the nickname/identifier of the user.
///   The mutex allows safe concurrent access and modification of this field across threads.
/// - `address`:
///   A `String` representing the user's IP address or hostname. This value is immutable after
///   being set, ensuring consistency of the address for the user instance.
/// - `is_active`:
///   A `Mutex`-protected `bool` indicating whether the user is currently active.
///   This field can be safely updated from multiple threads and is used to track
///   whether the user is still participating in the system.
pub struct User {
    pub socket: Option<TcpStream>,
    pub is_active: Mutex<bool>,
    pub member: Arc<Member>,
}

impl User {
    /// Constructs a new instance of the struct using a provided `TcpStream`
    /// and an optional address.
    ///
    /// # Arguments
    /// * `tcp_stream` - A `TcpStream` that represents the underlying TCP connection.
    /// * `address` - An optional `String` containing the address associated with the connection.
    ///   If `None`, the address is determined from the local address of the provided `TcpStream`.
    ///
    /// # Returns
    /// A new instance of the struct with the following initialized fields:
    /// * `socket` - Set as `Some(tcp_stream)`, holding the associated TCP connection.
    /// * `nickname` - A `Mutex`-wrapped `Option` initialized to `None`, representing the optional user nickname.
    /// * `address` - The provided address if available, or the local address from the `TcpStream` converted to a string.
    /// * `is_active` - A `Mutex`-locked boolean value initialized to `true`, indicating that the connection is active.
    ///
    /// # Panics
    /// This function will panic if the call to `tcp_stream.local_addr()` fails, as `.unwrap()` is used
    /// to handle the result. Ensure that the `TcpStream` is correctly initialized and valid before calling this method.
    ///
    /// # Example
    /// ```rust
    /// use std::net::TcpStream;
    /// use your_module::YourStruct; // Replace with your actual module and structure name.
    ///
    /// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    /// let instance = YourStruct::from(stream, None);
    ///
    /// // Use the created instance...
    /// ```
    pub fn from(tcp_stream: TcpStream, address: Option<String>) -> Self {
        let address = match address {
            Some(address) => address,
            None => tcp_stream.local_addr().unwrap().to_string(),
        };

        Self {
            socket: Some(tcp_stream),
            is_active: Mutex::new(true),
            member: Arc::new(Member::new(address)),
        }
    }

    /// Retrieves the display name of the user.
    ///
    /// This asynchronous function checks if the user has a nickname assigned.
    /// - If a nickname is set, it returns the nickname.
    /// - If no nickname is available, it falls back to the user's address.
    ///
    /// # Returns
    /// A `String` representing the user's display name:
    /// - The nickname if it's set.
    /// - Otherwise, the address.
    ///
    /// # Concurrency
    /// This function uses an asynchronous lock to safely access the user's nickname
    /// in a potentially concurrent context.
    ///
    /// # Example
    /// ```rust
    /// let display_name = user.get_display_name().await;
    /// println!("User's display name: {}", display_name);
    /// ```
    ///
    /// # Dependencies
    /// - `self.nickname` must be a type that supports asynchronous locking.
    /// - `self.address` must be a `String`.
    ///
    /// # Panics
    /// This function does not explicitly handle panic scenarios.
    /// Ensure that `self.nickname` and `self.address` are properly initialized.
    ///
    /// # Errors
    /// This function does not return errors as it defaults to `self.address`
    /// if the nickname is absent.
    pub async fn get_display_name(&self) -> String {
        let nickname = self.member.nick_name.lock().await;

        match &*nickname {
            Some(nick_name) => nick_name.clone(),
            None => self.member.address.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MemberDataTransferObject {
    pub id: Vec<u8>,
    pub address: Vec<u8>,
    pub nick_name: Option<Vec<u8>>,
}

impl MemberDataTransferObject {
    pub async fn from(member: &Member) -> Self {
        let nick_name_guard = member.nick_name.lock().await;
        let nick_name_clone = nick_name_guard.as_deref();
        let nick_name_clone = match nick_name_clone {
            Some(nn) => Some(nn.to_string().into_bytes()),
            None => None,
        };
        Self {
            id: member.id.into_bytes().to_vec(),
            address: member.address.clone().into_bytes(),
            nick_name: nick_name_clone,
        }
    }
}

pub struct Member {
    pub id: String,
    pub address: String,
    pub nick_name: Mutex<Option<String>>,
}

impl Member {
    pub fn new(address: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            address,
            nick_name: Mutex::new(None),
        }
    }
}
