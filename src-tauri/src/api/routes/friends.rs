use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::Friend;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct FriendRequest {
    pub peer_id: String,
    pub display_name: String,
}

pub async fn send_friend_request(
    State(ctx): State<ServiceContext>,
    Json(body): Json<FriendRequest>,
) -> Result<(StatusCode, Json<Friend>), (StatusCode, String)> {
    services::friends::send_friend_request(&ctx, &body.peer_id, &body.display_name)
        .map(|f| (StatusCode::CREATED, Json(f)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn accept_friend_request(
    State(ctx): State<ServiceContext>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::friends::accept_friend_request(&ctx, &peer_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn remove_friend(
    State(ctx): State<ServiceContext>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::friends::remove_friend(&ctx, &peer_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn list_friends(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<Friend>>, (StatusCode, String)> {
    services::friends::list_friends(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_friend(
    State(ctx): State<ServiceContext>,
    Path(peer_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::friends::get_friend(&ctx, &peer_id)
        .map(|f| Json(serde_json::json!(f)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
