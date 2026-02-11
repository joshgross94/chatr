use crate::models::Identity;
use crate::state::ServiceContext;

pub fn get_peer_id(ctx: &ServiceContext) -> Result<String, String> {
    Ok(ctx.peer_id.clone())
}

pub fn get_identity(ctx: &ServiceContext) -> Result<Identity, String> {
    let (display_name, avatar_hash, status_message, status_type) =
        ctx.db.get_identity_profile().map_err(|e| e.to_string())?;
    Ok(Identity {
        peer_id: ctx.peer_id.clone(),
        display_name,
        avatar_hash,
        status_message,
        status_type,
    })
}

pub fn get_display_name(ctx: &ServiceContext) -> Result<String, String> {
    ctx.db.get_display_name().map_err(|e| e.to_string())
}

pub fn set_display_name(ctx: &ServiceContext, name: &str) -> Result<(), String> {
    ctx.db.set_display_name(name).map_err(|e| e.to_string())
}

pub fn set_status(ctx: &ServiceContext, message: Option<&str>, status_type: Option<&str>) -> Result<(), String> {
    ctx.db.set_status(message, status_type).map_err(|e| e.to_string())
}

pub fn set_avatar_hash(ctx: &ServiceContext, hash: Option<&str>) -> Result<(), String> {
    ctx.db.set_avatar_hash(hash).map_err(|e| e.to_string())
}
