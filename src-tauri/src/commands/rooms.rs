use tauri::State;
use crate::models::{Channel, Room};
use crate::services;
use crate::state::AppState;

#[tauri::command]
pub async fn create_room(state: State<'_, AppState>, name: String) -> Result<Room, String> {
    services::rooms::create_room(&state.ctx, name).await
}

#[tauri::command]
pub async fn join_room(state: State<'_, AppState>, invite_code: String) -> Result<Room, String> {
    services::rooms::join_room(&state.ctx, invite_code).await
}

#[tauri::command]
pub fn list_rooms(state: State<'_, AppState>) -> Result<Vec<Room>, String> {
    services::rooms::list_rooms(&state.ctx)
}

#[tauri::command]
pub fn get_channels(state: State<'_, AppState>, room_id: String) -> Result<Vec<Channel>, String> {
    services::rooms::get_channels(&state.ctx, &room_id)
}
