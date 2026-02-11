use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::Setting;
use crate::services;
use crate::state::ServiceContext;

pub async fn get_all_settings(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<Setting>>, (StatusCode, String)> {
    services::settings::get_all_settings(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_setting(
    State(ctx): State<ServiceContext>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::settings::get_setting(&ctx, &key)
        .map(|v| Json(serde_json::json!({"key": key, "value": v})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SetSettingRequest {
    pub value: String,
}

pub async fn set_setting(
    State(ctx): State<ServiceContext>,
    Path(key): Path<String>,
    Json(body): Json<SetSettingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::settings::set_setting(&ctx, &key, &body.value)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn delete_setting(
    State(ctx): State<ServiceContext>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::settings::delete_setting(&ctx, &key)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
