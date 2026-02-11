use chrono::Utc;
use uuid::Uuid;

use crate::models::CustomEmoji;
use crate::state::ServiceContext;

pub fn add_emoji(
    ctx: &ServiceContext,
    room_id: &str,
    name: &str,
    file_hash: &str,
) -> Result<CustomEmoji, String> {
    let emoji = CustomEmoji {
        id: Uuid::new_v4().to_string(),
        room_id: room_id.to_string(),
        name: name.to_string(),
        file_hash: file_hash.to_string(),
        uploaded_by: ctx.peer_id.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    ctx.db.add_custom_emoji(&emoji).map_err(|e| e.to_string())?;
    Ok(emoji)
}

pub fn remove_emoji(ctx: &ServiceContext, emoji_id: &str) -> Result<(), String> {
    ctx.db.remove_custom_emoji(emoji_id).map_err(|e| e.to_string())
}

pub fn list_emoji(ctx: &ServiceContext, room_id: &str) -> Result<Vec<CustomEmoji>, String> {
    ctx.db.list_custom_emoji(room_id).map_err(|e| e.to_string())
}
