use chrono::{DateTime, Utc};
use scylla::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Branch, Message, MessageRole, Permission, Share};

// Database row model for conversation_lineage table
#[derive(Debug, Clone, FromRow)]
pub struct MessageRow {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub role: String,
    pub content_type: String,
    pub content_data: String,
    pub content_metadata: Option<HashMap<String, String>>,
    pub lineage: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl MessageRow {
    pub fn from_message(message: &Message) -> Result<Self, String> {
        let content_type = message.content.to_type_string().to_string();
        let content_data = message
            .content
            .to_json_string()
            .map_err(|e| format!("Failed to serialize content: {}", e))?;

        Ok(MessageRow {
            conversation_id: message.conversation_id,
            message_id: message.message_id,
            parent_message_id: message.parent_message_id,
            role: message.role.as_str().to_string(),
            content_type,
            content_data,
            content_metadata: Some(message.content_metadata.clone()),
            lineage: message.lineage.clone(),
            created_at: message.created_at,
            created_by: message.created_by.clone(),
        })
    }

    pub fn to_message(self) -> Result<Message, String> {
        let role =
            MessageRole::parse(&self.role).ok_or_else(|| format!("Invalid role: {}", self.role))?;

        let content =
            crate::domain::ContentType::from_parts(&self.content_type, &self.content_data)
                .map_err(|e| format!("Failed to deserialize content: {}", e))?;

        Ok(Message {
            conversation_id: self.conversation_id,
            message_id: self.message_id,
            parent_message_id: self.parent_message_id,
            role,
            content,
            content_metadata: self.content_metadata.unwrap_or_default(),
            lineage: self.lineage,
            created_at: self.created_at,
            created_by: self.created_by,
        })
    }
}

// Database row model for conversation_branches table
#[derive(Debug, Clone, FromRow)]
pub struct BranchRow {
    pub conversation_id: Uuid,
    pub branch_id: Uuid,
    pub branch_name: String,
    pub leaf_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub created_by: String,
    pub is_active: bool,
}

impl BranchRow {
    pub fn from_branch(branch: &Branch) -> Self {
        BranchRow {
            conversation_id: branch.conversation_id,
            branch_id: branch.branch_id,
            branch_name: branch.branch_name.clone(),
            leaf_message_id: branch.leaf_message_id,
            created_at: branch.created_at,
            last_updated: branch.last_updated,
            created_by: branch.created_by.clone(),
            is_active: branch.is_active,
        }
    }

    pub fn to_branch(self) -> Branch {
        Branch {
            conversation_id: self.conversation_id,
            branch_id: self.branch_id,
            branch_name: self.branch_name,
            leaf_message_id: self.leaf_message_id,
            created_at: self.created_at,
            last_updated: self.last_updated,
            created_by: self.created_by,
            is_active: self.is_active,
        }
    }
}

// Database row model for conversation_shares table
#[derive(Debug, Clone, FromRow)]
pub struct ShareRow {
    pub conversation_id: Uuid,
    pub shared_with: String,
    pub permission: String,
    pub shared_at: DateTime<Utc>,
    pub shared_by: String,
}

impl ShareRow {
    pub fn from_share(share: &Share) -> Self {
        ShareRow {
            conversation_id: share.conversation_id,
            shared_with: share.shared_with.clone(),
            permission: share.permission.as_str().to_string(),
            shared_at: share.shared_at,
            shared_by: share.shared_by.clone(),
        }
    }

    pub fn to_share(self) -> Result<Share, String> {
        let permission = Permission::parse(&self.permission)
            .ok_or_else(|| format!("Invalid permission: {}", self.permission))?;

        Ok(Share {
            conversation_id: self.conversation_id,
            shared_with: self.shared_with,
            permission,
            shared_at: self.shared_at,
            shared_by: self.shared_by,
        })
    }
}

// Database row model for user_conversations table
#[derive(Debug, Clone, FromRow)]
pub struct UserConversationRow {
    pub user_id: String,
    pub last_activity: DateTime<Utc>,
    pub conversation_id: Uuid,
    pub active_branch_id: Option<Uuid>,
}

// Database row model for branch_by_leaf table
#[derive(Debug, Clone, FromRow)]
pub struct BranchByLeafRow {
    pub leaf_message_id: Uuid,
    pub conversation_id: Uuid,
    pub branch_id: Uuid,
}
