use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::api::{
    dto::{CreateMessageRequest, MessageResponse, parse_role},
    error::ApiError,
};
use crate::services::{BranchService, ConversationService};
use std::sync::Arc;

pub async fn create_message(
    State(conv_service): State<Arc<ConversationService>>,
    State(branch_service): State<Arc<BranchService>>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let role = parse_role(&payload.role).map_err(ApiError::BadRequest)?;

    let message = conv_service
        .append_message(
            conversation_id,
            payload.parent_message_id,
            role,
            payload.content,
            payload.content_metadata,
            payload.created_by,
        )
        .await?;

    // If a branch_id is provided, extend the branch
    if let Some(branch_id) = payload.branch_id {
        branch_service
            .extend_branch_with_message(conversation_id, branch_id, message.message_id)
            .await?;
    }

    Ok(Json(message.into()))
}

pub async fn get_message(
    State(service): State<Arc<ConversationService>>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MessageResponse>, ApiError> {
    let message = service.get_message(conversation_id, message_id).await?;

    Ok(Json(message.into()))
}

pub async fn get_message_children(
    State(service): State<Arc<ConversationService>>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MessageResponse>>, ApiError> {
    let children = service.get_children(conversation_id, message_id).await?;

    let responses = children.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

pub async fn get_message_lineage(
    State(service): State<Arc<ConversationService>>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MessageResponse>>, ApiError> {
    let lineage = service
        .get_lineage_path(conversation_id, message_id)
        .await?;

    let responses = lineage.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}
