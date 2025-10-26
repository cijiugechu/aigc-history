use chrono::Utc;
use scylla::IntoTypedRows;
use scylla::query::Query;
use uuid::Uuid;

use crate::db::{DbClient, DbError, ShareRow, UserConversationRow};
use crate::domain::Share;

#[derive(Clone)]
pub struct ShareRepository {
    client: DbClient,
}

impl ShareRepository {
    pub fn new(client: DbClient) -> Self {
        Self { client }
    }

    /// Insert a new share
    pub async fn insert_share(&self, share: &Share) -> Result<(), DbError> {
        let row = ShareRow::from_share(share);
        let query = Query::new(crate::db::queries::INSERT_SHARE);

        self.client
            .session()
            .query(
                query,
                (
                    row.conversation_id,
                    row.shared_with,
                    row.permission,
                    row.shared_at,
                    row.shared_by,
                ),
            )
            .await?;

        Ok(())
    }

    /// Get a specific share
    pub async fn get_share(
        &self,
        conversation_id: Uuid,
        shared_with: &str,
    ) -> Result<Share, DbError> {
        let query = Query::new(crate::db::queries::SELECT_SHARE);

        let result = self
            .client
            .session()
            .query(query, (conversation_id, shared_with))
            .await?;

        let row = result
            .rows
            .ok_or(DbError::NotFound)?
            .into_typed::<ShareRow>()
            .next()
            .ok_or(DbError::NotFound)?
            .map_err(|e| DbError::InvalidData(format!("Failed to parse share row: {}", e)))?;

        row.to_share().map_err(DbError::InvalidData)
    }

    /// Get all shares for a conversation
    pub async fn get_shares_by_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Share>, DbError> {
        let query = Query::new(crate::db::queries::SELECT_SHARES_BY_CONVERSATION);

        let result = self
            .client
            .session()
            .query(query, (conversation_id,))
            .await?;

        let rows = result.rows.unwrap_or_default();
        let mut shares = Vec::new();

        for row in rows.into_typed::<ShareRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            let share = row.to_share().map_err(DbError::InvalidData)?;
            shares.push(share);
        }

        Ok(shares)
    }

    /// Delete a share
    pub async fn delete_share(
        &self,
        conversation_id: Uuid,
        shared_with: &str,
    ) -> Result<(), DbError> {
        let query = Query::new(crate::db::queries::DELETE_SHARE);

        self.client
            .session()
            .query(query, (conversation_id, shared_with))
            .await?;

        Ok(())
    }

    /// Add or update user conversation activity
    pub async fn upsert_user_conversation(
        &self,
        user_id: &str,
        conversation_id: Uuid,
        active_branch_id: Option<Uuid>,
    ) -> Result<(), DbError> {
        let now = Utc::now();
        let query = Query::new(crate::db::queries::INSERT_USER_CONVERSATION);

        self.client
            .session()
            .query(query, (user_id, now, conversation_id, active_branch_id))
            .await?;

        Ok(())
    }

    /// Get user's conversations (most recent first)
    pub async fn get_user_conversations(
        &self,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<UserConversationRow>, DbError> {
        let query = Query::new(crate::db::queries::SELECT_USER_CONVERSATIONS);

        let result = self.client.session().query(query, (user_id, limit)).await?;

        let rows = result.rows.unwrap_or_default();
        let mut conversations = Vec::new();

        for row in rows.into_typed::<UserConversationRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            conversations.push(row);
        }

        Ok(conversations)
    }
}
