use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::{DmConversation, DmMessage, DmParticipant};
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct CreateDmRequest {
    pub peer_ids: Vec<String>,
    pub name: Option<String>,
}

pub async fn create_dm(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CreateDmRequest>,
) -> Result<(StatusCode, Json<DmConversation>), (StatusCode, String)> {
    services::dms::create_dm(&ctx, body.peer_ids, body.name)
        .map(|dm| (StatusCode::CREATED, Json(dm)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn list_dms(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<DmConversation>>, (StatusCode, String)> {
    services::dms::list_dms(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_dm_participants(
    State(ctx): State<ServiceContext>,
    Path(conversation_id): Path<String>,
) -> Result<Json<Vec<DmParticipant>>, (StatusCode, String)> {
    services::dms::get_dm_participants(&ctx, &conversation_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SendDmRequest {
    pub content: String,
}

pub async fn send_dm_message(
    State(ctx): State<ServiceContext>,
    Path(conversation_id): Path<String>,
    Json(body): Json<SendDmRequest>,
) -> Result<(StatusCode, Json<DmMessage>), (StatusCode, String)> {
    services::dms::send_dm_message(&ctx, &conversation_id, &body.content)
        .map(|msg| (StatusCode::CREATED, Json(msg)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct GetDmMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
}

pub async fn get_dm_messages(
    State(ctx): State<ServiceContext>,
    Path(conversation_id): Path<String>,
    Query(params): Query<GetDmMessagesQuery>,
) -> Result<Json<Vec<DmMessage>>, (StatusCode, String)> {
    services::dms::get_dm_messages(&ctx, &conversation_id, params.limit, params.before.as_deref())
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
