use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::api::{
    dto::{ConversationResponse, ForkConversationRequest},
    error::ApiError,
};
use crate::domain::ContentType;
use crate::services::ForkService;
use std::sync::Arc;

pub async fn fork_conversation(
    State(service): State<Arc<ForkService>>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<ForkConversationRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conversation = service
        .fork_conversation(conversation_id, payload.title, payload.created_by)
        .await?;

    let response = match &conversation.root_message.content {
        ContentType::Metadata(metadata) => ConversationResponse {
            conversation_id: conversation.conversation_id,
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            created_at: conversation.root_message.created_at,
            created_by: conversation.root_message.created_by.clone(),
            is_public: metadata.is_public,
            fork_from_conversation_id: metadata.fork_from_conversation_id,
            fork_from_message_id: metadata.fork_from_message_id,
        },
        _ => {
            return Err(ApiError::Internal(
                "Invalid root message content".to_string(),
            ));
        }
    };

    Ok(Json(response))
}

pub async fn fork_branch(
    State(service): State<Arc<ForkService>>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ForkConversationRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conversation = service
        .fork_branch(
            conversation_id,
            branch_id,
            payload.title,
            payload.created_by,
        )
        .await?;

    let response = match &conversation.root_message.content {
        ContentType::Metadata(metadata) => ConversationResponse {
            conversation_id: conversation.conversation_id,
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            created_at: conversation.root_message.created_at,
            created_by: conversation.root_message.created_by.clone(),
            is_public: metadata.is_public,
            fork_from_conversation_id: metadata.fork_from_conversation_id,
            fork_from_message_id: metadata.fork_from_message_id,
        },
        _ => {
            return Err(ApiError::Internal(
                "Invalid root message content".to_string(),
            ));
        }
    };

    Ok(Json(response))
}

pub async fn fork_from_message(
    State(service): State<Arc<ForkService>>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ForkConversationRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conversation = service
        .fork_from_message(
            conversation_id,
            message_id,
            payload.title,
            payload.created_by,
        )
        .await?;

    let response = match &conversation.root_message.content {
        ContentType::Metadata(metadata) => ConversationResponse {
            conversation_id: conversation.conversation_id,
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            created_at: conversation.root_message.created_at,
            created_by: conversation.root_message.created_by.clone(),
            is_public: metadata.is_public,
            fork_from_conversation_id: metadata.fork_from_conversation_id,
            fork_from_message_id: metadata.fork_from_message_id,
        },
        _ => {
            return Err(ApiError::Internal(
                "Invalid root message content".to_string(),
            ));
        }
    };

    Ok(Json(response))
}
