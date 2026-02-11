use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::FileMetadata;
use crate::services;
use crate::state::ServiceContext;

#[derive(Deserialize)]
pub struct RegisterFileRequest {
    pub filename: String,
    pub size: i64,
    pub mime_type: String,
    pub sha256_hash: String,
    pub chunk_count: i32,
}

pub async fn register_file(
    State(ctx): State<ServiceContext>,
    Json(body): Json<RegisterFileRequest>,
) -> Result<(StatusCode, Json<FileMetadata>), (StatusCode, String)> {
    services::files::register_file(
        &ctx,
        &body.filename,
        body.size,
        &body.mime_type,
        &body.sha256_hash,
        body.chunk_count,
    )
    .map(|f| (StatusCode::CREATED, Json(f)))
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_file(
    State(ctx): State<ServiceContext>,
    Path(file_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::files::get_file(&ctx, &file_id)
        .map(|f| Json(serde_json::json!(f)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct AttachFileRequest {
    pub file_id: String,
}

pub async fn attach_file(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
    Json(body): Json<AttachFileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::files::attach_file(&ctx, &message_id, &body.file_id)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_attachments(
    State(ctx): State<ServiceContext>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<FileMetadata>>, (StatusCode, String)> {
    services::files::get_attachments(&ctx, &message_id)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
