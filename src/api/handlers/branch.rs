use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::api::{
    dto::{BranchResponse, CreateBranchRequest, MessageResponse, UpdateBranchRequest},
    error::ApiError,
};
use crate::services::BranchService;
use std::sync::Arc;

pub async fn create_branch(
    State(service): State<Arc<BranchService>>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<CreateBranchRequest>,
) -> Result<Json<BranchResponse>, ApiError> {
    let branch = service
        .create_branch(
            conversation_id,
            payload.branch_name,
            payload.leaf_message_id,
            payload.created_by,
        )
        .await?;

    Ok(Json(branch.into()))
}

pub async fn get_branch(
    State(service): State<Arc<BranchService>>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BranchResponse>, ApiError> {
    let branch = service.get_branch(conversation_id, branch_id).await?;

    Ok(Json(branch.into()))
}

pub async fn get_branches(
    State(service): State<Arc<BranchService>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Vec<BranchResponse>>, ApiError> {
    let branches = service.get_branches(conversation_id).await?;

    let responses = branches.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

pub async fn get_branch_messages(
    State(service): State<Arc<BranchService>>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MessageResponse>>, ApiError> {
    let messages = service
        .get_branch_messages(conversation_id, branch_id)
        .await?;

    let responses = messages.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

pub async fn update_branch(
    State(service): State<Arc<BranchService>>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateBranchRequest>,
) -> Result<Json<BranchResponse>, ApiError> {
    if let Some(new_name) = payload.branch_name {
        service
            .update_branch_name(conversation_id, branch_id, new_name)
            .await?;
    }

    if let Some(new_leaf_id) = payload.leaf_message_id {
        service
            .update_branch_leaf(conversation_id, branch_id, new_leaf_id)
            .await?;
    }

    // Fetch updated branch
    get_branch(State(service), Path((conversation_id, branch_id))).await
}

pub async fn delete_branch(
    State(service): State<Arc<BranchService>>,
    Path((conversation_id, branch_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    service.delete_branch(conversation_id, branch_id).await?;

    Ok(Json(serde_json::json!({
        "message": "Branch deleted successfully"
    })))
}
