use chrono::Utc;
use uuid::Uuid;

use crate::events::AppEvent;
use crate::models::{DmConversation, DmMessage, DmParticipant};
use crate::state::ServiceContext;

pub fn create_dm(
    ctx: &ServiceContext,
    peer_ids: Vec<String>,
    name: Option<String>,
) -> Result<DmConversation, String> {
    let is_group = peer_ids.len() > 1;
    let conv = DmConversation {
        id: Uuid::new_v4().to_string(),
        is_group,
        name,
        created_at: Utc::now().to_rfc3339(),
    };
    ctx.db.create_dm_conversation(&conv).map_err(|e| e.to_string())?;

    // Add self as participant
    let now = Utc::now().to_rfc3339();
    let self_participant = DmParticipant {
        conversation_id: conv.id.clone(),
        peer_id: ctx.peer_id.clone(),
        joined_at: now.clone(),
    };
    ctx.db.add_dm_participant(&self_participant).map_err(|e| e.to_string())?;

    // Add other participants
    for pid in peer_ids {
        let participant = DmParticipant {
            conversation_id: conv.id.clone(),
            peer_id: pid,
            joined_at: now.clone(),
        };
        ctx.db.add_dm_participant(&participant).map_err(|e| e.to_string())?;
    }

    Ok(conv)
}

pub fn list_dms(ctx: &ServiceContext) -> Result<Vec<DmConversation>, String> {
    ctx.db.list_dm_conversations().map_err(|e| e.to_string())
}

pub fn get_dm_participants(ctx: &ServiceContext, conversation_id: &str) -> Result<Vec<DmParticipant>, String> {
    ctx.db.get_dm_participants(conversation_id).map_err(|e| e.to_string())
}

pub fn send_dm_message(
    ctx: &ServiceContext,
    conversation_id: &str,
    content: &str,
) -> Result<DmMessage, String> {
    let display_name = ctx.db.get_display_name().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    ctx.db.insert_dm_message(&id, conversation_id, &ctx.peer_id, &display_name, content, &now)
        .map_err(|e| e.to_string())?;

    let msg = DmMessage {
        id,
        conversation_id: conversation_id.to_string(),
        sender_peer_id: ctx.peer_id.clone(),
        sender_display_name: display_name,
        content: content.to_string(),
        timestamp: now,
    };

    let _ = ctx.event_tx.send(AppEvent::NewDmMessage(msg.clone()));
    Ok(msg)
}

pub fn get_dm_messages(
    ctx: &ServiceContext,
    conversation_id: &str,
    limit: Option<i64>,
    before: Option<&str>,
) -> Result<Vec<DmMessage>, String> {
    ctx.db.get_dm_messages(conversation_id, limit.unwrap_or(50), before)
        .map_err(|e| e.to_string())
}
