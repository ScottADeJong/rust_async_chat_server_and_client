use crate::{Member, MemberDataTransferObject};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct MessageDataTransferObject {
    pub content: Vec<u8>,
    pub author: MemberDataTransferObject,
    pub channel: Destination,
    pub kind: MessageKind,
}

impl MessageDataTransferObject {
    pub async fn from(message: Message) -> Self {
        Self {
            content: message.content,
            author: MemberDataTransferObject::from(message.author.as_ref()).await,
            channel: message.channel,
            kind: message.kind,
        }
    }
}

pub struct Message {
    pub content: Vec<u8>,
    pub author: Arc<Member>,
    pub channel: Destination,
    pub kind: MessageKind,
}

#[derive(Serialize, Deserialize)]
pub enum Destination {
    Global,
    Channel(Channel),
    Direct(Member),
}

#[derive(Serialize, Deserialize)]
pub enum MessageKind {
    Message,
    Command,
    ServerBroadcast,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
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
