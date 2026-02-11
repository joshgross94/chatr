use tauri::State;
use crate::models::Message;
use crate::services;
use crate::state::AppState;

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    channel_id: String,
    content: String,
    reply_to_id: Option<String>,
) -> Result<Message, String> {
    services::messaging::send_message(&state.ctx, channel_id, content, reply_to_id).await
}

#[tauri::command]
pub fn get_messages(
    state: State<'_, AppState>,
    channel_id: String,
    limit: Option<i64>,
    before: Option<String>,
) -> Result<Vec<Message>, String> {
    services::messaging::get_messages(&state.ctx, &channel_id, limit, before.as_deref())
}

#[tauri::command]
pub async fn get_room_peers(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<Vec<crate::models::PeerInfo>, String> {
    services::peers::get_room_peers(&state.ctx, &room_id).await
}
