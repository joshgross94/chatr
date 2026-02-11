use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::{BlockedPeer, ModerationAction};
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct ModerateRequest {
    pub action_type: String,
    pub target_peer_id: String,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
}

pub async fn moderate(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
    Json(body): Json<ModerateRequest>,
) -> Result<(StatusCode, Json<ModerationAction>), (StatusCode, String)> {
    services::moderation::moderate(
        &ctx,
        &room_id,
        &body.action_type,
        &body.target_peer_id,
        body.reason.as_deref(),
        body.expires_at.as_deref(),
    )
    .map(|a| (StatusCode::CREATED, Json(a)))
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_audit_log(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<ModerationAction>>, (StatusCode, String)> {
    services::moderation::get_audit_log(&ctx, &room_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct BlockRequest {
    pub peer_id: String,
}

pub async fn block_peer(
    State(ctx): State<ServiceContext>,
    Json(body): Json<BlockRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::moderation::block_peer(&ctx, &body.peer_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn unblock_peer(
    State(ctx): State<ServiceContext>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::moderation::unblock_peer(&ctx, &peer_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_blocked_peers(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<BlockedPeer>>, (StatusCode, String)> {
    services::moderation::get_blocked_peers(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
