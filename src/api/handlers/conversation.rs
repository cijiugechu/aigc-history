use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::api::{
    dto::{
        ConversationResponse, CreateConversationRequest, TreeResponse, UpdateConversationRequest,
    },
    error::ApiError,
};
use crate::domain::ContentType;
use crate::services::ConversationService;
use std::sync::Arc;

pub async fn create_conversation(
    State(service): State<Arc<ConversationService>>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conversation = service
        .create_conversation(payload.title, payload.created_by)
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

pub async fn get_conversation(
    State(service): State<Arc<ConversationService>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ConversationResponse>, ApiError> {
    let conversation = service.get_conversation(conversation_id).await?;

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

pub async fn update_conversation(
    State(service): State<Arc<ConversationService>>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<Json<ConversationResponse>, ApiError> {
    service
        .update_conversation(conversation_id, payload.title, payload.description)
        .await?;

    // Fetch updated conversation
    get_conversation(State(service), Path(conversation_id)).await
}

pub async fn delete_conversation(
    State(service): State<Arc<ConversationService>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    service.delete_conversation(conversation_id).await?;

    Ok(Json(serde_json::json!({
        "message": "Conversation deleted successfully"
    })))
}

pub async fn get_conversation_tree(
    State(service): State<Arc<ConversationService>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<TreeResponse>, ApiError> {
    let messages = service.get_conversation_tree(conversation_id).await?;

    let total = messages.len();
    let message_responses = messages.into_iter().map(Into::into).collect();

    Ok(Json(TreeResponse {
        conversation_id,
        messages: message_responses,
        total_messages: total,
    }))
}
