use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Branch, ContentType, Message, MessageRole, Permission};

// Request DTOs
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConversationRequest {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub parent_message_id: Uuid,
    pub role: String,
    pub content: ContentType,
    #[serde(default)]
    pub content_metadata: HashMap<String, String>,
    pub created_by: String,
    pub branch_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    pub branch_name: String,
    pub leaf_message_id: Uuid,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBranchRequest {
    pub branch_name: Option<String>,
    pub leaf_message_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ForkConversationRequest {
    pub title: String,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct ShareConversationRequest {
    pub shared_with: String,
    pub permission: String,
    pub shared_by: String,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub conversation_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub is_public: bool,
    pub fork_from_conversation_id: Option<Uuid>,
    pub fork_from_message_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub role: String,
    pub content: ContentType,
    pub content_metadata: HashMap<String, String>,
    pub lineage: Vec<Uuid>,
    pub depth: usize,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        let depth = msg.depth();
        let lineage = msg.lineage.clone();
        MessageResponse {
            conversation_id: msg.conversation_id,
            message_id: msg.message_id,
            parent_message_id: msg.parent_message_id,
            role: msg.role.as_str().to_string(),
            content: msg.content,
            content_metadata: msg.content_metadata,
            lineage,
            depth,
            created_at: msg.created_at,
            created_by: msg.created_by,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BranchResponse {
    pub conversation_id: Uuid,
    pub branch_id: Uuid,
    pub branch_name: String,
    pub leaf_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub created_by: String,
    pub is_active: bool,
}

impl From<Branch> for BranchResponse {
    fn from(branch: Branch) -> Self {
        BranchResponse {
            conversation_id: branch.conversation_id,
            branch_id: branch.branch_id,
            branch_name: branch.branch_name,
            leaf_message_id: branch.leaf_message_id,
            created_at: branch.created_at,
            last_updated: branch.last_updated,
            created_by: branch.created_by,
            is_active: branch.is_active,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub conversation_id: Uuid,
    pub shared_with: String,
    pub permission: String,
    pub shared_at: DateTime<Utc>,
    pub shared_by: String,
}

#[derive(Debug, Serialize)]
pub struct TreeResponse {
    pub conversation_id: Uuid,
    pub messages: Vec<MessageResponse>,
    pub total_messages: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

// Helper to parse role from string
pub fn parse_role(role_str: &str) -> Result<MessageRole, String> {
    MessageRole::parse(role_str).ok_or_else(|| format!("Invalid role: {}", role_str))
}

// Helper to parse permission from string
pub fn parse_permission(permission_str: &str) -> Result<Permission, String> {
    Permission::parse(permission_str)
        .ok_or_else(|| format!("Invalid permission: {}", permission_str))
}
