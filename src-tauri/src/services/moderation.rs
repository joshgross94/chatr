use chrono::Utc;
use uuid::Uuid;

use crate::models::{BlockedPeer, ModerationAction};
use crate::state::ServiceContext;

pub fn moderate(
    ctx: &ServiceContext,
    room_id: &str,
    action_type: &str,
    target_peer_id: &str,
    reason: Option<&str>,
    expires_at: Option<&str>,
) -> Result<ModerationAction, String> {
    let action = ModerationAction {
        id: Uuid::new_v4().to_string(),
        room_id: room_id.to_string(),
        action_type: action_type.to_string(),
        target_peer_id: target_peer_id.to_string(),
        moderator_peer_id: ctx.peer_id.clone(),
        reason: reason.map(|s| s.to_string()),
        created_at: Utc::now().to_rfc3339(),
        expires_at: expires_at.map(|s| s.to_string()),
    };
    ctx.db.add_moderation_action(&action).map_err(|e| e.to_string())?;
    Ok(action)
}

pub fn get_audit_log(ctx: &ServiceContext, room_id: &str) -> Result<Vec<ModerationAction>, String> {
    ctx.db.get_moderation_actions(room_id).map_err(|e| e.to_string())
}

pub fn block_peer(ctx: &ServiceContext, peer_id: &str) -> Result<(), String> {
    ctx.db.block_peer(peer_id, &Utc::now().to_rfc3339()).map_err(|e| e.to_string())
}

pub fn unblock_peer(ctx: &ServiceContext, peer_id: &str) -> Result<(), String> {
    ctx.db.unblock_peer(peer_id).map_err(|e| e.to_string())
}

pub fn get_blocked_peers(ctx: &ServiceContext) -> Result<Vec<BlockedPeer>, String> {
    ctx.db.get_blocked_peers().map_err(|e| e.to_string())
}
