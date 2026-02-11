use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::{Channel, Room};
use crate::services;
use crate::state::ServiceContext;

pub async fn list_rooms(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<Room>>, (StatusCode, String)> {
    services::rooms::list_rooms(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

pub async fn create_room(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<Room>), (StatusCode, String)> {
    services::rooms::create_room(&ctx, body.name)
        .await
        .map(|room| (StatusCode::CREATED, Json(room)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct JoinRoomRequest {
    pub invite_code: String,
}

pub async fn join_room(
    State(ctx): State<ServiceContext>,
    Json(body): Json<JoinRoomRequest>,
) -> Result<Json<Room>, (StatusCode, String)> {
    services::rooms::join_room(&ctx, body.invite_code)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn get_channels(
    State(ctx): State<ServiceContext>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
    services::rooms::get_channels(&ctx, &room_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
