use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::api::{
    dto::{ShareConversationRequest, ShareResponse, parse_permission},
    error::ApiError,
};
use crate::services::ShareService;
use std::sync::Arc;

pub async fn share_conversation(
    State(service): State<Arc<ShareService>>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<ShareConversationRequest>,
) -> Result<Json<ShareResponse>, ApiError> {
    let permission = parse_permission(&payload.permission).map_err(ApiError::BadRequest)?;

    let share = service
        .share_conversation(
            conversation_id,
            payload.shared_with,
            permission,
            payload.shared_by,
        )
        .await?;

    Ok(Json(ShareResponse {
        conversation_id: share.conversation_id,
        shared_with: share.shared_with,
        permission: share.permission.as_str().to_string(),
        shared_at: share.shared_at,
        shared_by: share.shared_by,
    }))
}

pub async fn get_shares(
    State(service): State<Arc<ShareService>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ShareResponse>>, ApiError> {
    let shares = service.get_conversation_shares(conversation_id).await?;

    let responses = shares
        .into_iter()
        .map(|share| ShareResponse {
            conversation_id: share.conversation_id,
            shared_with: share.shared_with,
            permission: share.permission.as_str().to_string(),
            shared_at: share.shared_at,
            shared_by: share.shared_by,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn revoke_share(
    State(service): State<Arc<ShareService>>,
    Path((conversation_id, user_id)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    service.revoke_share(conversation_id, &user_id).await?;

    Ok(Json(serde_json::json!({
        "message": "Share revoked successfully"
    })))
}

pub async fn get_user_conversations(
    State(service): State<Arc<ShareService>>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Uuid>>, ApiError> {
    let conversations = service.get_user_conversations(&user_id, 50).await?;

    Ok(Json(conversations))
}
