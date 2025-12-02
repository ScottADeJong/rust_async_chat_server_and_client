use crate::Member;
use std::sync::Arc;
use uuid::Uuid;

// TODO: implement serde here
pub struct Message {
    pub content: Vec<u8>,
    pub author: Arc<Member>,
    pub channel: Destination,
    pub kind: MessageKind,
}

pub enum Destination {
    Global,
    Channel(Channel),
    Direct(Member),
}

pub enum MessageKind {
    Message,
    Command,
    ServerBroadcast,
}

#[allow(dead_code)]
pub struct Channel {
    id: Uuid,
    display_name: String,
}

impl Message {
    pub fn new(author: Arc<Member>) -> Self {
        Self {
            author,
            content: Vec::new(),
            channel: Destination::Global,
            kind: MessageKind::Message,
        }
    }
    pub fn from_string(author: Arc<Member>, message: String) -> Self {
        Self {
            author,
            content: message.into_bytes(),
            channel: Destination::Global,
            kind: MessageKind::Message,
        }
    }

    pub fn as_string(&self) -> String {
        // FIX ME: This can consume, remove clone later
        String::from_utf8(self.content.clone()).unwrap_or_else(|_| String::new())
    }
}
