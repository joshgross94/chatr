use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::NotificationSetting;
use crate::services;
use crate::state::ServiceContext;

pub async fn get_all_notification_settings(
    State(ctx): State<ServiceContext>,
) -> Result<Json<Vec<NotificationSetting>>, (StatusCode, String)> {
    services::notifications::get_all_notification_settings(&ctx)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

#[derive(Deserialize)]
pub struct SetNotificationRequest {
    pub level: String,
}

pub async fn set_notification_setting(
    State(ctx): State<ServiceContext>,
    Path((target_type, target_id)): Path<(String, String)>,
    Json(body): Json<SetNotificationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::notifications::set_notification_setting(&ctx, &target_id, &target_type, &body.level)
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_notification_setting(
    State(ctx): State<ServiceContext>,
    Path((target_type, target_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    services::notifications::get_notification_setting(&ctx, &target_id, &target_type)
        .map(|level| Json(serde_json::json!({"target_id": target_id, "target_type": target_type, "level": level})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
