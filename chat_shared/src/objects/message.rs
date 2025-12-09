use crate::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub address: String,
    pub content: Vec<u8>,
    pub channel: Destination,
    pub kind: MessageKind,
}

#[derive(Serialize, Deserialize)]
pub enum Destination {
    Global,
    Channel(Channel),
    Direct(Client),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageKind {
    Message,
    Command,
    ServerBroadcast,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Channel {
    id: String,
    display_name: String,
}

impl Message {
    pub fn new(author: Arc<Client>) -> Self {
        Self {
            address: author.address.to_string(),
            content: Vec::new(),
            channel: Destination::Global,
            kind: MessageKind::Message,
        }
    }

    pub fn from_string(author: Arc<Client>, message: String, kind: MessageKind) -> Self {
        Self {
            address: author.address.to_string(),
            content: message.into_bytes(),
            channel: Destination::Global,
            kind,
        }
    }

    pub fn as_string(&self) -> String {
        // FIX ME: This can consume, remove clone later
        String::from_utf8(self.content.clone()).unwrap_or_else(|_| String::new())
    }
}
