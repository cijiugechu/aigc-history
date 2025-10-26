use chrono::Utc;
use uuid::Uuid;

use crate::db::DbError;
use crate::domain::{Permission, Share};
use crate::repositories::ShareRepository;

pub struct ShareService {
    share_repo: ShareRepository,
}

impl ShareService {
    pub fn new(share_repo: ShareRepository) -> Self {
        Self { share_repo }
    }

    /// Share a conversation with a user
    pub async fn share_conversation(
        &self,
        conversation_id: Uuid,
        shared_with: String,
        permission: Permission,
        shared_by: String,
    ) -> Result<Share, DbError> {
        let share = Share {
            conversation_id,
            shared_with,
            permission,
            shared_at: Utc::now(),
            shared_by,
        };

        self.share_repo.insert_share(&share).await?;

        Ok(share)
    }

    /// Get a specific share
    pub async fn get_share(
        &self,
        conversation_id: Uuid,
        shared_with: &str,
    ) -> Result<Share, DbError> {
        self.share_repo
            .get_share(conversation_id, shared_with)
            .await
    }

    /// Get all shares for a conversation
    pub async fn get_conversation_shares(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Share>, DbError> {
        self.share_repo
            .get_shares_by_conversation(conversation_id)
            .await
    }

    /// Revoke a share
    pub async fn revoke_share(
        &self,
        conversation_id: Uuid,
        shared_with: &str,
    ) -> Result<(), DbError> {
        self.share_repo
            .delete_share(conversation_id, shared_with)
            .await
    }

    /// Check if a user has permission to access a conversation
    pub async fn check_permission(
        &self,
        conversation_id: Uuid,
        user_id: &str,
        required_permission: Permission,
    ) -> Result<bool, DbError> {
        // Try to get the share
        match self.share_repo.get_share(conversation_id, user_id).await {
            Ok(share) => {
                // Check if the permission is sufficient
                let has_permission = match required_permission {
                    Permission::Read => share.permission.can_read(),
                    Permission::Branch => share.permission.can_branch(),
                    Permission::Fork => share.permission.can_fork(),
                };
                Ok(has_permission)
            }
            Err(DbError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Update user's conversation activity
    pub async fn update_user_activity(
        &self,
        user_id: &str,
        conversation_id: Uuid,
        active_branch_id: Option<Uuid>,
    ) -> Result<(), DbError> {
        self.share_repo
            .upsert_user_conversation(user_id, conversation_id, active_branch_id)
            .await
    }

    /// Get user's conversations
    pub async fn get_user_conversations(
        &self,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<Uuid>, DbError> {
        let rows = self
            .share_repo
            .get_user_conversations(user_id, limit)
            .await?;
        Ok(rows.into_iter().map(|r| r.conversation_id).collect())
    }
}
