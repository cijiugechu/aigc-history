use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::DbError;
use crate::domain::{ContentType, Conversation, Message, MessageRole};
use crate::repositories::LineageRepository;
use crate::utils::{compute_lineage, validate_lineage_depth};

pub struct ConversationService {
    lineage_repo: LineageRepository,
    app_config: AppConfig,
}

impl ConversationService {
    pub fn new(lineage_repo: LineageRepository, app_config: AppConfig) -> Self {
        Self {
            lineage_repo,
            app_config,
        }
    }

    /// Create a new conversation with a root message
    pub async fn create_conversation(
        &self,
        title: String,
        created_by: String,
    ) -> Result<Conversation, DbError> {
        let conversation = Conversation::new(title, created_by);

        // Insert the root message
        self.lineage_repo
            .insert_message(&conversation.root_message)
            .await?;

        Ok(conversation)
    }

    /// Get conversation metadata (root message)
    pub async fn get_conversation(&self, conversation_id: Uuid) -> Result<Conversation, DbError> {
        // Find the root message (parent_message_id is NULL)
        let all_messages = self.lineage_repo.get_all_messages(conversation_id).await?;

        let root_message = all_messages
            .into_iter()
            .find(|m| m.is_root())
            .ok_or(DbError::NotFound)?;

        Ok(Conversation {
            conversation_id,
            root_message,
        })
    }

    /// Update conversation metadata (root message)
    pub async fn update_conversation(
        &self,
        conversation_id: Uuid,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<(), DbError> {
        let mut conversation = self.get_conversation(conversation_id).await?;

        // Update the root message content
        if let ContentType::Metadata(ref mut metadata) = conversation.root_message.content {
            if let Some(new_title) = title {
                metadata.title = new_title;
            }
            if let Some(new_desc) = description {
                metadata.description = Some(new_desc);
            }
        }

        // Re-insert the root message (upsert behavior)
        self.lineage_repo
            .insert_message(&conversation.root_message)
            .await?;

        Ok(())
    }

    /// Delete an entire conversation
    pub async fn delete_conversation(&self, conversation_id: Uuid) -> Result<(), DbError> {
        self.lineage_repo.delete_conversation(conversation_id).await
    }

    /// Append a new message to a conversation
    pub async fn append_message(
        &self,
        conversation_id: Uuid,
        parent_message_id: Uuid,
        role: MessageRole,
        content: ContentType,
        content_metadata: std::collections::HashMap<String, String>,
        created_by: String,
    ) -> Result<Message, DbError> {
        // Get parent message to compute lineage
        let parent = self
            .lineage_repo
            .get_message(conversation_id, parent_message_id)
            .await?;

        // Compute new lineage
        let message_id = Uuid::new_v4();
        let lineage = compute_lineage(&parent.lineage, message_id);

        // Validate lineage depth
        validate_lineage_depth(&lineage, self.app_config.max_lineage_depth)
            .map_err(DbError::InvalidData)?;

        // Create new message
        let message = Message {
            conversation_id,
            message_id,
            parent_message_id: Some(parent_message_id),
            role,
            content,
            content_metadata,
            lineage,
            created_at: Utc::now(),
            created_by,
        };

        // Insert message
        self.lineage_repo.insert_message(&message).await?;

        Ok(message)
    }

    /// Get a specific message
    pub async fn get_message(
        &self,
        conversation_id: Uuid,
        message_id: Uuid,
    ) -> Result<Message, DbError> {
        self.lineage_repo
            .get_message(conversation_id, message_id)
            .await
    }

    /// Get all child messages (branches from this point)
    pub async fn get_children(
        &self,
        conversation_id: Uuid,
        parent_message_id: Uuid,
    ) -> Result<Vec<Message>, DbError> {
        self.lineage_repo
            .get_children(conversation_id, parent_message_id)
            .await
    }

    /// Get the lineage path for a message (from root to this message)
    pub async fn get_lineage_path(
        &self,
        conversation_id: Uuid,
        message_id: Uuid,
    ) -> Result<Vec<Message>, DbError> {
        let message = self
            .lineage_repo
            .get_message(conversation_id, message_id)
            .await?;

        self.lineage_repo
            .get_messages_by_ids(conversation_id, &message.lineage)
            .await
    }

    /// Get entire conversation tree
    pub async fn get_conversation_tree(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Message>, DbError> {
        self.lineage_repo.get_all_messages(conversation_id).await
    }
}
