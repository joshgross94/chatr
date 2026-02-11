use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::media::{audio, video, MediaCommand};
use crate::network::NetworkCommand;
use crate::state::ServiceContext;

// --- Camera & Screen share routes ---

#[derive(Deserialize)]
pub struct EnableCameraRequest {
    pub device_index: Option<u32>,
}

pub async fn enable_camera(
    State(ctx): State<ServiceContext>,
    Json(body): Json<EnableCameraRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::EnableCamera {
            device_index: body.device_index,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to enable camera: {}", e)))
}

pub async fn disable_camera(
    State(ctx): State<ServiceContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::DisableCamera)
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to disable camera: {}", e)))
}

pub async fn list_cameras(
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cameras = video::list_cameras();
    Ok(Json(serde_json::json!({ "cameras": cameras })))
}

pub async fn start_screen_share(
    State(ctx): State<ServiceContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::StartScreenShare)
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start screen share: {}", e)))
}

pub async fn stop_screen_share(
    State(ctx): State<ServiceContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::StopScreenShare)
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to stop screen share: {}", e)))
}

// --- Media engine voice routes ---

#[derive(Deserialize)]
pub struct JoinVoiceRequest {
    pub room_id: String,
    pub channel_id: String,
}

pub async fn join_voice(
    State(ctx): State<ServiceContext>,
    Json(body): Json<JoinVoiceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::JoinVoice {
            room_id: body.room_id,
            channel_id: body.channel_id,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to join voice: {}", e)))
}

pub async fn leave_voice(
    State(ctx): State<ServiceContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::LeaveVoice)
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to leave voice: {}", e)))
}

#[derive(Deserialize)]
pub struct MutedRequest {
    pub muted: bool,
}

pub async fn set_muted(
    State(ctx): State<ServiceContext>,
    Json(body): Json<MutedRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::SetMuted(body.muted))
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to set muted: {}", e)))
}

#[derive(Deserialize)]
pub struct DeafenedRequest {
    pub deafened: bool,
}

pub async fn set_deafened(
    State(ctx): State<ServiceContext>,
    Json(body): Json<DeafenedRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.media_tx
        .send(MediaCommand::SetDeafened(body.deafened))
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to set deafened: {}", e)))
}

pub async fn list_devices(
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let devices = audio::list_devices();
    Ok(Json(serde_json::json!({ "devices": devices })))
}

pub async fn get_voice_state(
    State(ctx): State<ServiceContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let state = ctx.voice_state_rx.borrow().clone();
    Ok(Json(serde_json::to_value(state).unwrap_or_default()))
}

// --- Existing signaling routes (kept for backwards compatibility) ---

#[derive(Deserialize)]
pub struct CallOfferRequest {
    pub room_id: String,
    pub to_peer_id: String,
    pub call_id: String,
    pub channel_id: String,
    pub sdp: String,
}

pub async fn send_call_offer(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CallOfferRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.network_tx
        .send(NetworkCommand::SendCallOffer {
            room_id: body.room_id,
            to_peer_id: body.to_peer_id,
            call_id: body.call_id,
            channel_id: body.channel_id,
            sdp: body.sdp,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send call offer: {}", e)))
}

#[derive(Deserialize)]
pub struct CallAnswerRequest {
    pub room_id: String,
    pub to_peer_id: String,
    pub call_id: String,
    pub channel_id: String,
    pub sdp: String,
}

pub async fn send_call_answer(
    State(ctx): State<ServiceContext>,
    Json(body): Json<CallAnswerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.network_tx
        .send(NetworkCommand::SendCallAnswer {
            room_id: body.room_id,
            to_peer_id: body.to_peer_id,
            call_id: body.call_id,
            channel_id: body.channel_id,
            sdp: body.sdp,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send call answer: {}", e)))
}

#[derive(Deserialize)]
pub struct IceCandidateRequest {
    pub room_id: String,
    pub to_peer_id: String,
    pub channel_id: String,
    pub candidate: String,
}

pub async fn send_ice_candidate(
    State(ctx): State<ServiceContext>,
    Json(body): Json<IceCandidateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.network_tx
        .send(NetworkCommand::SendIceCandidate {
            room_id: body.room_id,
            to_peer_id: body.to_peer_id,
            channel_id: body.channel_id,
            candidate: body.candidate,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send ICE candidate: {}", e)))
}

#[derive(Deserialize)]
pub struct VoiceStateRequest {
    pub room_id: String,
    pub channel_id: Option<String>,
    pub muted: bool,
    pub deafened: bool,
    pub video: bool,
    pub screen_sharing: bool,
}

pub async fn update_voice_state(
    State(ctx): State<ServiceContext>,
    Json(body): Json<VoiceStateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ctx.network_tx
        .send(NetworkCommand::SendVoiceState {
            room_id: body.room_id,
            channel_id: body.channel_id,
            muted: body.muted,
            deafened: body.deafened,
            video: body.video,
            screen_sharing: body.screen_sharing,
        })
        .await
        .map(|_| Json(serde_json::json!({"ok": true})))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update voice state: {}", e)))
}
