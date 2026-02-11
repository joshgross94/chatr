use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::Message;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before: Option<String>,
}

pub async fn get_messages(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Query(params): Query<GetMessagesQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    services::messaging::get_messages(&ctx, &channel_id, params.limit, params.before.as_deref())
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub reply_to_id: Option<String>,
}

pub async fn send_message(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Json(body): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<Message>), (StatusCode, String)> {
    services::messaging::send_message(&ctx, channel_id, body.content, body.reply_to_id)
        .await
        .map(|msg| (StatusCode::CREATED, Json(msg)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

pub async fn edit_message(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
    Json(body): Json<EditMessageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::edit_message(&ctx, &message_id, &body.content)
        .map(|updated| Json(serde_json::json!({"updated": updated})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn delete_message(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::delete_message(&ctx, &message_id)
        .map(|deleted| Json(serde_json::json!({"deleted": deleted})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct ReactionRequest {
    pub emoji: String,
}

pub async fn add_reaction(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
    Json(body): Json<ReactionRequest>,
) -> Result<(StatusCode, Json<crate::models::Reaction>), (StatusCode, String)> {
    services::messaging::add_reaction(&ctx, &message_id, &body.emoji)
        .map(|r| (StatusCode::CREATED, Json(r)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn remove_reaction(
    State(ctx): State<ServiceContext>,
    Path((message_id, emoji)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::remove_reaction(&ctx, &message_id, &emoji)
        .map(|removed| Json(serde_json::json!({"removed": removed})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_reactions(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<crate::models::Reaction>>, (StatusCode, String)> {
    services::messaging::get_reactions(&ctx, &message_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct TypingRequest {
    pub typing: bool,
}

pub async fn typing_indicator(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Json(body): Json<TypingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::typing_indicator(&ctx, &channel_id, body.typing)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct MarkReadRequest {
    pub last_read_message_id: String,
}

pub async fn mark_read(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Json(body): Json<MarkReadRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::mark_read(&ctx, &channel_id, &body.last_read_message_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_read_receipts(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<crate::models::ReadReceipt>>, (StatusCode, String)> {
    services::messaging::get_read_receipts(&ctx, &channel_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct PinRequest {
    pub message_id: String,
}

pub async fn pin_message(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
    Json(body): Json<PinRequest>,
) -> Result<(StatusCode, Json<crate::models::PinnedMessage>), (StatusCode, String)> {
    services::messaging::pin_message(&ctx, &channel_id, &body.message_id)
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn unpin_message(
    State(ctx): State<ServiceContext>,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::messaging::unpin_message(&ctx, &channel_id, &message_id)
        .map(|removed| Json(serde_json::json!({"removed": removed})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_pinned_messages(
    State(ctx): State<ServiceContext>,
    Path(channel_id): Path<String>,
) -> Result<Json<Vec<crate::models::PinnedMessage>>, (StatusCode, String)> {
    services::messaging::get_pinned_messages(&ctx, &channel_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub channel_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn search_messages(
    State(ctx): State<ServiceContext>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<crate::models::SearchResult>, (StatusCode, String)> {
    services::messaging::search_messages(&ctx, &params.q, params.channel_id.as_deref(), params.limit, params.offset)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
