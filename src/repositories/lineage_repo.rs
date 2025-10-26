use scylla::IntoTypedRows;
use scylla::query::Query;
use uuid::Uuid;

use crate::db::{DbClient, DbError, MessageRow};
use crate::domain::Message;

#[derive(Clone)]
pub struct LineageRepository {
    client: DbClient,
}

impl LineageRepository {
    pub fn new(client: DbClient) -> Self {
        Self { client }
    }

    /// Insert a new message into the conversation lineage
    pub async fn insert_message(&self, message: &Message) -> Result<(), DbError> {
        let row = MessageRow::from_message(message).map_err(DbError::SerializationError)?;

        let query = Query::new(crate::db::queries::INSERT_MESSAGE);

        self.client
            .session()
            .query(
                query,
                (
                    row.conversation_id,
                    row.message_id,
                    row.parent_message_id,
                    row.role,
                    row.content_type,
                    row.content_data,
                    row.content_metadata,
                    row.lineage,
                    row.created_at,
                    row.created_by,
                ),
            )
            .await?;

        Ok(())
    }

    /// Get a specific message by conversation_id and message_id
    pub async fn get_message(
        &self,
        conversation_id: Uuid,
        message_id: Uuid,
    ) -> Result<Message, DbError> {
        let query = Query::new(crate::db::queries::SELECT_MESSAGE);

        let result = self
            .client
            .session()
            .query(query, (conversation_id, message_id))
            .await?;

        let row = result
            .rows
            .ok_or(DbError::NotFound)?
            .into_typed::<MessageRow>()
            .next()
            .ok_or(DbError::NotFound)?
            .map_err(|e| DbError::InvalidData(format!("Failed to parse message row: {}", e)))?;

        row.to_message().map_err(DbError::InvalidData)
    }

    /// Get all child messages of a given message (branches from this point)
    pub async fn get_children(
        &self,
        conversation_id: Uuid,
        parent_message_id: Uuid,
    ) -> Result<Vec<Message>, DbError> {
        let query = Query::new(crate::db::queries::SELECT_MESSAGE_CHILDREN);

        let result = self
            .client
            .session()
            .query(query, (conversation_id, parent_message_id))
            .await?;

        let rows = result.rows.unwrap_or_default();
        let mut messages = Vec::new();

        for row in rows.into_typed::<MessageRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            let message = row.to_message().map_err(DbError::InvalidData)?;
            messages.push(message);
        }

        Ok(messages)
    }

    /// Get multiple messages by their IDs (useful for fetching a lineage path)
    pub async fn get_messages_by_ids(
        &self,
        conversation_id: Uuid,
        message_ids: &[Uuid],
    ) -> Result<Vec<Message>, DbError> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = Query::new(crate::db::queries::SELECT_MESSAGES_BY_IDS);

        let result = self
            .client
            .session()
            .query(query, (conversation_id, message_ids))
            .await?;

        let rows = result.rows.unwrap_or_default();
        let mut messages = Vec::new();

        for row in rows.into_typed::<MessageRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            let message = row.to_message().map_err(DbError::InvalidData)?;
            messages.push(message);
        }

        // Sort by lineage depth to maintain order
        messages.sort_by_key(|m| m.lineage.len());

        Ok(messages)
    }

    /// Get all messages in a conversation (entire tree)
    pub async fn get_all_messages(&self, conversation_id: Uuid) -> Result<Vec<Message>, DbError> {
        let query = Query::new(crate::db::queries::SELECT_ALL_MESSAGES);

        let result = self
            .client
            .session()
            .query(query, (conversation_id,))
            .await?;

        let rows = result.rows.unwrap_or_default();
        let mut messages = Vec::new();

        for row in rows.into_typed::<MessageRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            let message = row.to_message().map_err(DbError::InvalidData)?;
            messages.push(message);
        }

        Ok(messages)
    }

    /// Delete an entire conversation (all messages)
    pub async fn delete_conversation(&self, conversation_id: Uuid) -> Result<(), DbError> {
        let query = Query::new(crate::db::queries::DELETE_CONVERSATION);

        self.client
            .session()
            .query(query, (conversation_id,))
            .await?;

        Ok(())
    }

    /// Batch insert multiple messages (useful for forking)
    pub async fn batch_insert_messages(&self, messages: &[Message]) -> Result<(), DbError> {
        use scylla::batch::Batch;
        use scylla::batch::BatchType;

        let mut batch = Batch::new(BatchType::Unlogged);
        let query_str = crate::db::queries::INSERT_MESSAGE;

        let mut values_list = Vec::new();

        for message in messages {
            let row = MessageRow::from_message(message).map_err(DbError::SerializationError)?;

            batch.append_statement(query_str);
            values_list.push((
                row.conversation_id,
                row.message_id,
                row.parent_message_id,
                row.role,
                row.content_type,
                row.content_data,
                row.content_metadata,
                row.lineage,
                row.created_at,
                row.created_by,
            ));
        }

        self.client.session().batch(&batch, values_list).await?;

        Ok(())
    }
}
