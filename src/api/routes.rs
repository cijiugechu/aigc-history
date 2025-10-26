use axum::{
    Router,
    routing::{delete, get, post},
};
use std::sync::Arc;

use crate::services::{BranchService, ConversationService, ForkService, ShareService};

use super::handlers;

#[derive(Clone)]
pub struct AppState {
    pub conversation_service: Arc<ConversationService>,
    pub branch_service: Arc<BranchService>,
    pub fork_service: Arc<ForkService>,
    pub share_service: Arc<ShareService>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Conversations
        .route(
            "/api/v1/conversations",
            post(handlers::create_conversation).with_state(state.conversation_service.clone()),
        )
        .route(
            "/api/v1/conversations/{id}",
            get(handlers::get_conversation)
                .with_state(state.conversation_service.clone())
                .put(handlers::update_conversation)
                .with_state(state.conversation_service.clone())
                .delete(handlers::delete_conversation)
                .with_state(state.conversation_service.clone()),
        )
        .route(
            "/api/v1/conversations/{id}/tree",
            get(handlers::get_conversation_tree).with_state(state.conversation_service.clone()),
        )
        // Messages
        .route(
            "/api/v1/conversations/{id}/messages",
            post({
                let conv_service = state.conversation_service.clone();
                let branch_service = state.branch_service.clone();
                move |path, json| {
                    handlers::create_message(
                        axum::extract::State(conv_service.clone()),
                        axum::extract::State(branch_service.clone()),
                        path,
                        json,
                    )
                }
            }),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/messages/{message_id}",
            get(handlers::get_message).with_state(state.conversation_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/messages/{message_id}/children",
            get(handlers::get_message_children).with_state(state.conversation_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/messages/{message_id}/lineage",
            get(handlers::get_message_lineage).with_state(state.conversation_service.clone()),
        )
        // Branches
        .route(
            "/api/v1/conversations/{id}/branches",
            post(handlers::create_branch)
                .with_state(state.branch_service.clone())
                .get(handlers::get_branches)
                .with_state(state.branch_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/branches/{branch_id}",
            get(handlers::get_branch)
                .with_state(state.branch_service.clone())
                .put(handlers::update_branch)
                .with_state(state.branch_service.clone())
                .delete(handlers::delete_branch)
                .with_state(state.branch_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/branches/{branch_id}/messages",
            get(handlers::get_branch_messages).with_state(state.branch_service.clone()),
        )
        // Forking
        .route(
            "/api/v1/conversations/{id}/fork",
            post(handlers::fork_conversation).with_state(state.fork_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/branches/{branch_id}/fork",
            post(handlers::fork_branch).with_state(state.fork_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/messages/{message_id}/fork",
            post(handlers::fork_from_message).with_state(state.fork_service.clone()),
        )
        // Sharing
        .route(
            "/api/v1/conversations/{id}/share",
            post(handlers::share_conversation).with_state(state.share_service.clone()),
        )
        .route(
            "/api/v1/conversations/{id}/shares",
            get(handlers::get_shares).with_state(state.share_service.clone()),
        )
        .route(
            "/api/v1/conversations/{conversation_id}/shares/{user_id}",
            delete(handlers::revoke_share).with_state(state.share_service.clone()),
        )
        .route(
            "/api/v1/users/{user_id}/conversations",
            get(handlers::get_user_conversations).with_state(state.share_service.clone()),
        )
}

async fn health_check() -> axum::Json<crate::api::dto::HealthResponse> {
    axum::Json(crate::api::dto::HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now(),
    })
}
