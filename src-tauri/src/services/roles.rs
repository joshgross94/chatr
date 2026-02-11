use chrono::Utc;
use uuid::Uuid;

use crate::models::RoomRole;
use crate::state::ServiceContext;

pub fn set_role(
    ctx: &ServiceContext,
    room_id: &str,
    peer_id: &str,
    role: &str,
) -> Result<RoomRole, String> {
    let r = RoomRole {
        id: Uuid::new_v4().to_string(),
        room_id: room_id.to_string(),
        peer_id: peer_id.to_string(),
        role: role.to_string(),
        assigned_by: ctx.peer_id.clone(),
        assigned_at: Utc::now().to_rfc3339(),
    };
    ctx.db.set_role(&r).map_err(|e| e.to_string())?;
    Ok(r)
}

pub fn get_role(ctx: &ServiceContext, room_id: &str, peer_id: &str) -> Result<Option<RoomRole>, String> {
    ctx.db.get_role(room_id, peer_id).map_err(|e| e.to_string())
}

pub fn get_room_roles(ctx: &ServiceContext, room_id: &str) -> Result<Vec<RoomRole>, String> {
    ctx.db.get_room_roles(room_id).map_err(|e| e.to_string())
}

pub fn remove_role(ctx: &ServiceContext, room_id: &str, peer_id: &str) -> Result<(), String> {
    ctx.db.remove_role(room_id, peer_id).map_err(|e| e.to_string())
}
