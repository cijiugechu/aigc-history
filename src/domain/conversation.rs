use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub conversation_id: Uuid,
    pub root_message: Message,
}

impl Conversation {
    pub fn new(title: String, created_by: String) -> Self {
        let conversation_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();

        Conversation {
            conversation_id,
            root_message: Message::new_root(conversation_id, root_message_id, title, created_by),
        }
    }

    pub fn title(&self) -> Option<String> {
        match &self.root_message.content {
            super::content::ContentType::Metadata(m) => Some(m.title.clone()),
            _ => None,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.root_message.created_at
    }

    pub fn created_by(&self) -> &str {
        &self.root_message.created_by
    }
}
