use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::CustomEmoji;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct AddEmojiRequest {
    pub name: String,
    pub file_hash: String,
}

pub async fn add_emoji(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
    Json(body): Json<AddEmojiRequest>,
) -> Result<(StatusCode, Json<CustomEmoji>), (StatusCode, String)> {
    services::emoji::add_emoji(&ctx, &room_id, &body.name, &body.file_hash)
        .map(|e| (StatusCode::CREATED, Json(e)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn remove_emoji(
    State(ctx): State<ServiceContext>,
    Path(emoji_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::emoji::remove_emoji(&ctx, &emoji_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn list_emoji(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<CustomEmoji>>, (StatusCode, String)> {
    services::emoji::list_emoji(&ctx, &room_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
