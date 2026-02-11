use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::RoomRole;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct SetRoleRequest {
    pub peer_id: String,
    pub role: String,
}

pub async fn set_role(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
    Json(body): Json<SetRoleRequest>,
) -> Result<(StatusCode, Json<RoomRole>), (StatusCode, String)> {
    services::roles::set_role(&ctx, &room_id, &body.peer_id, &body.role)
        .map(|r| (StatusCode::CREATED, Json(r)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_room_roles(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<RoomRole>>, (StatusCode, String)> {
    services::roles::get_room_roles(&ctx, &room_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn remove_role(
    State(ctx): State<ServiceContext>,
    Path((room_id, peer_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::roles::remove_role(&ctx, &room_id, &peer_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
