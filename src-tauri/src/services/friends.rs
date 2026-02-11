use chrono::Utc;

use crate::events::AppEvent;
use crate::models::Friend;
use crate::state::ServiceContext;

pub fn send_friend_request(ctx: &ServiceContext, peer_id: &str, display_name: &str) -> Result<Friend, String> {
    let friend = Friend {
        peer_id: peer_id.to_string(),
        display_name: display_name.to_string(),
        status: "pending_outgoing".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    ctx.db.add_friend(&friend).map_err(|e| e.to_string())?;
    Ok(friend)
}

pub fn accept_friend_request(ctx: &ServiceContext, peer_id: &str) -> Result<(), String> {
    ctx.db.update_friend_status(peer_id, "accepted").map_err(|e| e.to_string())?;
    let _ = ctx.event_tx.send(AppEvent::FriendRequestAccepted {
        peer_id: peer_id.to_string(),
    });
    Ok(())
}

pub fn remove_friend(ctx: &ServiceContext, peer_id: &str) -> Result<(), String> {
    ctx.db.remove_friend(peer_id).map_err(|e| e.to_string())
}

pub fn list_friends(ctx: &ServiceContext) -> Result<Vec<Friend>, String> {
    ctx.db.list_friends().map_err(|e| e.to_string())
}

pub fn get_friend(ctx: &ServiceContext, peer_id: &str) -> Result<Option<Friend>, String> {
    ctx.db.get_friend(peer_id).map_err(|e| e.to_string())
}
