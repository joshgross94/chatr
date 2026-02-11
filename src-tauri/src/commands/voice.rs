use tauri::State;
use crate::media::{MediaCommand, audio, video};
use crate::network::NetworkCommand;
use crate::state::AppState;

/// Join a voice channel (starts audio capture + WebRTC connections in media engine).
#[tauri::command]
pub async fn join_voice_channel(
    state: State<'_, AppState>,
    room_id: String,
    channel_id: String,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::JoinVoice { room_id, channel_id })
        .await
        .map_err(|e| format!("Failed to join voice: {}", e))
}

/// Leave the current voice channel.
#[tauri::command]
pub async fn leave_voice_channel(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::LeaveVoice)
        .await
        .map_err(|e| format!("Failed to leave voice: {}", e))
}

/// Set muted state.
#[tauri::command]
pub async fn set_muted(
    state: State<'_, AppState>,
    muted: bool,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::SetMuted(muted))
        .await
        .map_err(|e| format!("Failed to set muted: {}", e))
}

/// Set deafened state.
#[tauri::command]
pub async fn set_deafened(
    state: State<'_, AppState>,
    deafened: bool,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::SetDeafened(deafened))
        .await
        .map_err(|e| format!("Failed to set deafened: {}", e))
}

/// List available audio devices.
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<audio::AudioDevice>, String> {
    Ok(audio::list_devices())
}

/// Enable camera.
#[tauri::command]
pub async fn enable_camera(
    state: State<'_, AppState>,
    device_index: Option<u32>,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::EnableCamera { device_index })
        .await
        .map_err(|e| format!("Failed to enable camera: {}", e))
}

/// Disable camera.
#[tauri::command]
pub async fn disable_camera(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::DisableCamera)
        .await
        .map_err(|e| format!("Failed to disable camera: {}", e))
}

/// List available cameras.
#[tauri::command]
pub async fn list_cameras() -> Result<Vec<video::CameraDevice>, String> {
    Ok(video::list_cameras())
}

/// Start screen sharing.
#[tauri::command]
pub async fn start_screen_share(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::StartScreenShare)
        .await
        .map_err(|e| format!("Failed to start screen share: {}", e))
}

/// Stop screen sharing.
#[tauri::command]
pub async fn stop_screen_share(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .ctx
        .media_tx
        .send(MediaCommand::StopScreenShare)
        .await
        .map_err(|e| format!("Failed to stop screen share: {}", e))
}

// --- Existing signaling commands (still used for network-level signaling) ---

#[tauri::command]
pub async fn send_call_offer(
    state: State<'_, AppState>,
    room_id: String,
    to_peer_id: String,
    call_id: String,
    channel_id: String,
    sdp: String,
) -> Result<(), String> {
    state
        .ctx
        .network_tx
        .send(NetworkCommand::SendCallOffer {
            room_id,
            to_peer_id,
            call_id,
            channel_id,
            sdp,
        })
        .await
        .map_err(|e| format!("Failed to send call offer: {}", e))
}

#[tauri::command]
pub async fn send_call_answer(
    state: State<'_, AppState>,
    room_id: String,
    to_peer_id: String,
    call_id: String,
    channel_id: String,
    sdp: String,
) -> Result<(), String> {
    state
        .ctx
        .network_tx
        .send(NetworkCommand::SendCallAnswer {
            room_id,
            to_peer_id,
            call_id,
            channel_id,
            sdp,
        })
        .await
        .map_err(|e| format!("Failed to send call answer: {}", e))
}

#[tauri::command]
pub async fn send_ice_candidate(
    state: State<'_, AppState>,
    room_id: String,
    to_peer_id: String,
    channel_id: String,
    candidate: String,
) -> Result<(), String> {
    state
        .ctx
        .network_tx
        .send(NetworkCommand::SendIceCandidate {
            room_id,
            to_peer_id,
            channel_id,
            candidate,
        })
        .await
        .map_err(|e| format!("Failed to send ICE candidate: {}", e))
}

#[tauri::command]
pub async fn update_voice_state(
    state: State<'_, AppState>,
    room_id: String,
    channel_id: Option<String>,
    muted: bool,
    deafened: bool,
    video: bool,
    screen_sharing: bool,
) -> Result<(), String> {
    state
        .ctx
        .network_tx
        .send(NetworkCommand::SendVoiceState {
            room_id,
            channel_id,
            muted,
            deafened,
            video,
            screen_sharing,
        })
        .await
        .map_err(|e| format!("Failed to update voice state: {}", e))
}
