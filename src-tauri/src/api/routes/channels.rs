use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::Channel;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: Option<String>,
}

pub async fn create_channel(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), (StatusCode, String)> {
    services::channels::create_channel(&ctx, &room_id, &body.name, body.channel_type.as_deref())
        .map(|ch| (StatusCode::CREATED, Json(ch)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<i32>,
}

pub async fn update_channel(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Json(body): Json<UpdateChannelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::channels::update_channel(
        &ctx,
        &channel_id,
        body.name.as_deref(),
        body.topic.as_deref(),
        body.position,
    )
    .map(|_| Json(serde_json::json!({"ok": true})))
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn delete_channel(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Look up room_id for broadcast before deleting
    let room_id = ctx.db.get_channel_room_id(&channel_id).ok().flatten();
    services::channels::delete_channel(&ctx, &channel_id, room_id.as_deref())
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
