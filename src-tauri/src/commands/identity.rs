use tauri::State;
use crate::models::Identity;
use crate::services;
use crate::state::AppState;

#[tauri::command]
pub fn get_api_port(state: State<'_, AppState>) -> Result<u16, String> {
    Ok(state.api_port)
}

#[tauri::command]
pub fn get_my_peer_id(state: State<'_, AppState>) -> Result<String, String> {
    services::identity::get_peer_id(&state.ctx)
}

#[tauri::command]
pub fn get_identity(state: State<'_, AppState>) -> Result<Identity, String> {
    services::identity::get_identity(&state.ctx)
}

#[tauri::command]
pub fn get_display_name(state: State<'_, AppState>) -> Result<String, String> {
    services::identity::get_display_name(&state.ctx)
}

#[tauri::command]
pub fn set_display_name(state: State<'_, AppState>, name: String) -> Result<(), String> {
    services::identity::set_display_name(&state.ctx, &name)
}
