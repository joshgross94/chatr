use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::models::Identity;
use crate::services;
use crate::state::ServiceContext;

pub async fn get_identity(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Identity>, (StatusCode, String)> {
    services::identity::get_identity(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SetDisplayNameRequest {
    pub name: String,
}

pub async fn set_display_name(
    State(ctx): State<ServiceContext>,
    Json(body): Json<SetDisplayNameRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::identity::set_display_name(&ctx, &body.name)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SetStatusRequest {
    pub message: Option<String>,
    pub status_type: Option<String>,
}

pub async fn set_status(
    State(ctx): State<ServiceContext>,
    Json(body): Json<SetStatusRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::identity::set_status(&ctx, body.message.as_deref(), body.status_type.as_deref())
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SetAvatarRequest {
    pub hash: Option<String>,
}

pub async fn set_avatar(
    State(ctx): State<ServiceContext>,
    Json(body): Json<SetAvatarRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::identity::set_avatar_hash(&ctx, body.hash.as_deref())
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
