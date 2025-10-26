use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::content::{ContentMetadata, ContentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub role: MessageRole,
    pub content: ContentType,
    pub content_metadata: ContentMetadata,
    pub lineage: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    Root,
    Human,
    Assistant,
    System,
    Tool,
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        match self {
            MessageRole::Root => "root",
            MessageRole::Human => "human",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
            MessageRole::Tool => "tool",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "root" => Some(MessageRole::Root),
            "human" => Some(MessageRole::Human),
            "assistant" => Some(MessageRole::Assistant),
            "system" => Some(MessageRole::System),
            "tool" => Some(MessageRole::Tool),
            _ => None,
        }
    }
}

impl Message {
    pub fn new_root(
        conversation_id: Uuid,
        message_id: Uuid,
        title: String,
        created_by: String,
    ) -> Self {
        Message {
            conversation_id,
            message_id,
            parent_message_id: None,
            role: MessageRole::Root,
            content: ContentType::Metadata(super::content::MetadataContent {
                title,
                description: None,
                is_public: false,
                fork_from_conversation_id: None,
                fork_from_message_id: None,
            }),
            content_metadata: ContentMetadata::new(),
            lineage: vec![message_id],
            created_at: Utc::now(),
            created_by,
        }
    }

    pub fn is_root(&self) -> bool {
        self.role == MessageRole::Root
    }

    pub fn depth(&self) -> usize {
        self.lineage.len()
    }
}
