use uuid::Uuid;

use crate::db::DbError;
use crate::domain::{Branch, Message};
use crate::repositories::{BranchRepository, LineageRepository};

pub struct BranchService {
    branch_repo: BranchRepository,
    lineage_repo: LineageRepository,
}

impl BranchService {
    pub fn new(branch_repo: BranchRepository, lineage_repo: LineageRepository) -> Self {
        Self {
            branch_repo,
            lineage_repo,
        }
    }

    /// Create a new branch
    pub async fn create_branch(
        &self,
        conversation_id: Uuid,
        branch_name: String,
        leaf_message_id: Uuid,
        created_by: String,
    ) -> Result<Branch, DbError> {
        // Validate that the leaf message exists
        self.lineage_repo
            .get_message(conversation_id, leaf_message_id)
            .await?;

        let branch = Branch::new(conversation_id, branch_name, leaf_message_id, created_by);

        self.branch_repo.insert_branch(&branch).await?;

        Ok(branch)
    }

    /// Get a specific branch
    pub async fn get_branch(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
    ) -> Result<Branch, DbError> {
        self.branch_repo
            .get_branch(conversation_id, branch_id)
            .await
    }

    /// Get all branches in a conversation
    pub async fn get_branches(&self, conversation_id: Uuid) -> Result<Vec<Branch>, DbError> {
        self.branch_repo
            .get_branches_by_conversation(conversation_id)
            .await
    }

    /// Get all messages in a branch (from root to leaf)
    pub async fn get_branch_messages(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
    ) -> Result<Vec<Message>, DbError> {
        let branch = self
            .branch_repo
            .get_branch(conversation_id, branch_id)
            .await?;

        let leaf_message = self
            .lineage_repo
            .get_message(conversation_id, branch.leaf_message_id)
            .await?;

        self.lineage_repo
            .get_messages_by_ids(conversation_id, &leaf_message.lineage)
            .await
    }

    /// Update branch leaf (move branch pointer to a new message)
    pub async fn update_branch_leaf(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        new_leaf_id: Uuid,
    ) -> Result<(), DbError> {
        // Validate that the new leaf message exists
        self.lineage_repo
            .get_message(conversation_id, new_leaf_id)
            .await?;

        let branch = self
            .branch_repo
            .get_branch(conversation_id, branch_id)
            .await?;

        self.branch_repo
            .update_branch_leaf(
                conversation_id,
                branch_id,
                branch.leaf_message_id,
                new_leaf_id,
            )
            .await
    }

    /// Update branch name
    pub async fn update_branch_name(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        new_name: String,
    ) -> Result<(), DbError> {
        self.branch_repo
            .update_branch_name(conversation_id, branch_id, new_name)
            .await
    }

    /// Delete a branch (messages remain in the conversation)
    pub async fn delete_branch(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
    ) -> Result<(), DbError> {
        let branch = self
            .branch_repo
            .get_branch(conversation_id, branch_id)
            .await?;

        self.branch_repo
            .delete_branch(conversation_id, branch_id, branch.leaf_message_id)
            .await
    }

    /// Automatically extend branch when a new message is appended
    pub async fn extend_branch_with_message(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        new_message_id: Uuid,
    ) -> Result<(), DbError> {
        let branch = self
            .branch_repo
            .get_branch(conversation_id, branch_id)
            .await?;

        self.branch_repo
            .update_branch_leaf(
                conversation_id,
                branch_id,
                branch.leaf_message_id,
                new_message_id,
            )
            .await
    }
}
