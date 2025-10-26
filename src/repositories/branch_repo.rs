use chrono::Utc;
use scylla::IntoTypedRows;
use scylla::query::Query;
use uuid::Uuid;

use crate::db::{BranchByLeafRow, BranchRow, DbClient, DbError};
use crate::domain::Branch;

#[derive(Clone)]
pub struct BranchRepository {
    client: DbClient,
}

impl BranchRepository {
    pub fn new(client: DbClient) -> Self {
        Self { client }
    }

    /// Insert a new branch
    pub async fn insert_branch(&self, branch: &Branch) -> Result<(), DbError> {
        let row = BranchRow::from_branch(branch);
        let query = Query::new(crate::db::queries::INSERT_BRANCH);

        self.client
            .session()
            .query(
                query,
                (
                    row.conversation_id,
                    row.branch_id,
                    row.branch_name,
                    row.leaf_message_id,
                    row.created_at,
                    row.last_updated,
                    row.created_by,
                    row.is_active,
                ),
            )
            .await?;

        // Also insert into branch_by_leaf index
        self.insert_branch_by_leaf(
            branch.leaf_message_id,
            branch.conversation_id,
            branch.branch_id,
        )
        .await?;

        Ok(())
    }

    /// Get a specific branch
    pub async fn get_branch(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
    ) -> Result<Branch, DbError> {
        let query = Query::new(crate::db::queries::SELECT_BRANCH);

        let result = self
            .client
            .session()
            .query(query, (conversation_id, branch_id))
            .await?;

        let row = result
            .rows
            .ok_or(DbError::NotFound)?
            .into_typed::<BranchRow>()
            .next()
            .ok_or(DbError::NotFound)?
            .map_err(|e| DbError::InvalidData(format!("Failed to parse branch row: {}", e)))?;

        Ok(row.to_branch())
    }

    /// Get all branches for a conversation
    pub async fn get_branches_by_conversation(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Branch>, DbError> {
        let query = Query::new(crate::db::queries::SELECT_BRANCHES_BY_CONVERSATION);

        let result = self
            .client
            .session()
            .query(query, (conversation_id,))
            .await?;

        let rows = result.rows.unwrap_or_default();
        let mut branches = Vec::new();

        for row in rows.into_typed::<BranchRow>() {
            let row =
                row.map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;
            branches.push(row.to_branch());
        }

        Ok(branches)
    }

    /// Update branch leaf message
    pub async fn update_branch_leaf(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        old_leaf_id: Uuid,
        new_leaf_id: Uuid,
    ) -> Result<(), DbError> {
        let now = Utc::now();
        let query = Query::new(crate::db::queries::UPDATE_BRANCH_LEAF);

        self.client
            .session()
            .query(query, (new_leaf_id, now, conversation_id, branch_id))
            .await?;

        // Update branch_by_leaf index
        self.delete_branch_by_leaf(old_leaf_id).await?;
        self.insert_branch_by_leaf(new_leaf_id, conversation_id, branch_id)
            .await?;

        Ok(())
    }

    /// Update branch name
    pub async fn update_branch_name(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        new_name: String,
    ) -> Result<(), DbError> {
        let now = Utc::now();
        let query = Query::new(crate::db::queries::UPDATE_BRANCH_NAME);

        self.client
            .session()
            .query(query, (new_name, now, conversation_id, branch_id))
            .await?;

        Ok(())
    }

    /// Delete a branch
    pub async fn delete_branch(
        &self,
        conversation_id: Uuid,
        branch_id: Uuid,
        leaf_message_id: Uuid,
    ) -> Result<(), DbError> {
        let query = Query::new(crate::db::queries::DELETE_BRANCH);

        self.client
            .session()
            .query(query, (conversation_id, branch_id))
            .await?;

        // Also delete from branch_by_leaf index
        self.delete_branch_by_leaf(leaf_message_id).await?;

        Ok(())
    }

    /// Get branch by leaf message ID
    pub async fn get_branch_by_leaf(&self, leaf_message_id: Uuid) -> Result<(Uuid, Uuid), DbError> {
        let query = Query::new(crate::db::queries::SELECT_BRANCH_BY_LEAF);

        let result = self
            .client
            .session()
            .query(query, (leaf_message_id,))
            .await?;

        let row = result
            .rows
            .ok_or(DbError::NotFound)?
            .into_typed::<BranchByLeafRow>()
            .next()
            .ok_or(DbError::NotFound)?
            .map_err(|e| DbError::InvalidData(format!("Failed to parse row: {}", e)))?;

        Ok((row.conversation_id, row.branch_id))
    }

    // Helper methods for branch_by_leaf index
    async fn insert_branch_by_leaf(
        &self,
        leaf_message_id: Uuid,
        conversation_id: Uuid,
        branch_id: Uuid,
    ) -> Result<(), DbError> {
        let query = Query::new(crate::db::queries::INSERT_BRANCH_BY_LEAF);

        self.client
            .session()
            .query(query, (leaf_message_id, conversation_id, branch_id))
            .await?;

        Ok(())
    }

    async fn delete_branch_by_leaf(&self, leaf_message_id: Uuid) -> Result<(), DbError> {
        let query = Query::new(crate::db::queries::DELETE_BRANCH_BY_LEAF);

        self.client
            .session()
            .query(query, (leaf_message_id,))
            .await?;

        Ok(())
    }
}
