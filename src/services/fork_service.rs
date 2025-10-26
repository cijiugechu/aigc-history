use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::DbError;
use crate::domain::{ContentType, Conversation, Message, MetadataContent};
use crate::repositories::{BranchRepository, LineageRepository};

pub struct ForkService {
    lineage_repo: LineageRepository,
    branch_repo: BranchRepository,
    app_config: AppConfig,
}

impl ForkService {
    pub fn new(
        lineage_repo: LineageRepository,
        branch_repo: BranchRepository,
        app_config: AppConfig,
    ) -> Self {
        Self {
            lineage_repo,
            branch_repo,
            app_config,
        }
    }

    /// Fork an entire conversation to a new conversation
    pub async fn fork_conversation(
        &self,
        source_conversation_id: Uuid,
        title: String,
        created_by: String,
    ) -> Result<Conversation, DbError> {
        // Get all messages from source conversation
        let source_messages = self
            .lineage_repo
            .get_all_messages(source_conversation_id)
            .await?;

        // Create new conversation with fork metadata
        let new_conversation_id = Uuid::new_v4();
        let new_root_id = Uuid::new_v4();

        let root_message = Message {
            conversation_id: new_conversation_id,
            message_id: new_root_id,
            parent_message_id: None,
            role: crate::domain::MessageRole::Root,
            content: ContentType::Metadata(MetadataContent {
                title: title.clone(),
                description: Some(format!(
                    "Forked from conversation {}",
                    source_conversation_id
                )),
                is_public: false,
                fork_from_conversation_id: Some(source_conversation_id),
                fork_from_message_id: None,
            }),
            content_metadata: std::collections::HashMap::new(),
            lineage: vec![new_root_id],
            created_at: chrono::Utc::now(),
            created_by: created_by.clone(),
        };

        // Copy all non-root messages with new conversation_id
        let mut forked_messages = vec![root_message.clone()];

        for msg in source_messages.iter() {
            if !msg.is_root() {
                let mut forked_msg = msg.clone();
                forked_msg.conversation_id = new_conversation_id;
                forked_messages.push(forked_msg);
            }
        }

        // Batch insert all messages
        self.batch_insert_with_limit(&forked_messages).await?;

        Ok(Conversation {
            conversation_id: new_conversation_id,
            root_message,
        })
    }

    /// Fork a specific branch to a new conversation
    pub async fn fork_branch(
        &self,
        source_conversation_id: Uuid,
        source_branch_id: Uuid,
        title: String,
        created_by: String,
    ) -> Result<Conversation, DbError> {
        // Get the branch
        let branch = self
            .branch_repo
            .get_branch(source_conversation_id, source_branch_id)
            .await?;

        // Get the leaf message to get its lineage
        let leaf_message = self
            .lineage_repo
            .get_message(source_conversation_id, branch.leaf_message_id)
            .await?;

        // Get all messages in the lineage
        let source_messages = self
            .lineage_repo
            .get_messages_by_ids(source_conversation_id, &leaf_message.lineage)
            .await?;

        // Fork from the specific branch point
        self.fork_messages(source_conversation_id, &source_messages, title, created_by)
            .await
    }

    /// Fork from a specific message (copies lineage up to that point)
    pub async fn fork_from_message(
        &self,
        source_conversation_id: Uuid,
        source_message_id: Uuid,
        title: String,
        created_by: String,
    ) -> Result<Conversation, DbError> {
        // Get the message to get its lineage
        let message = self
            .lineage_repo
            .get_message(source_conversation_id, source_message_id)
            .await?;

        // Get all messages in the lineage
        let source_messages = self
            .lineage_repo
            .get_messages_by_ids(source_conversation_id, &message.lineage)
            .await?;

        // Fork from this point
        self.fork_messages(source_conversation_id, &source_messages, title, created_by)
            .await
    }

    /// Helper function to fork a list of messages
    async fn fork_messages(
        &self,
        source_conversation_id: Uuid,
        source_messages: &[Message],
        title: String,
        created_by: String,
    ) -> Result<Conversation, DbError> {
        // Create new conversation with fork metadata
        let new_conversation_id = Uuid::new_v4();
        let new_root_id = Uuid::new_v4();

        let root_message = Message {
            conversation_id: new_conversation_id,
            message_id: new_root_id,
            parent_message_id: None,
            role: crate::domain::MessageRole::Root,
            content: ContentType::Metadata(MetadataContent {
                title: title.clone(),
                description: Some(format!(
                    "Forked from conversation {}",
                    source_conversation_id
                )),
                is_public: false,
                fork_from_conversation_id: Some(source_conversation_id),
                fork_from_message_id: source_messages.last().map(|m| m.message_id),
            }),
            content_metadata: std::collections::HashMap::new(),
            lineage: vec![new_root_id],
            created_at: chrono::Utc::now(),
            created_by: created_by.clone(),
        };

        // Copy messages with new conversation_id
        let mut forked_messages = vec![root_message.clone()];

        for msg in source_messages.iter() {
            if !msg.is_root() {
                let mut forked_msg = msg.clone();
                forked_msg.conversation_id = new_conversation_id;
                forked_messages.push(forked_msg);
            }
        }

        // Batch insert all messages
        self.batch_insert_with_limit(&forked_messages).await?;

        Ok(Conversation {
            conversation_id: new_conversation_id,
            root_message,
        })
    }

    /// Helper to batch insert with size limits
    async fn batch_insert_with_limit(&self, messages: &[Message]) -> Result<(), DbError> {
        let batch_size = self.app_config.max_batch_size;

        for chunk in messages.chunks(batch_size) {
            self.lineage_repo.batch_insert_messages(chunk).await?;
        }

        Ok(())
    }
}
