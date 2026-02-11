use chrono::Utc;
use uuid::Uuid;

use crate::events::AppEvent;
use crate::models::{Message, Reaction, SearchResult};
use crate::network::NetworkCommand;
use crate::state::ServiceContext;

pub async fn send_message(
    ctx: &ServiceContext,
    channel_id: String,
    content: String,
    reply_to_id: Option<String>,
) -> Result<Message, String> {
    let display_name = ctx.db.get_display_name().map_err(|e| e.to_string())?;

    let msg = Message {
        id: Uuid::new_v4().to_string(),
        channel_id: channel_id.clone(),
        sender_peer_id: ctx.peer_id.clone(),
        sender_display_name: display_name,
        content,
        timestamp: Utc::now().to_rfc3339(),
        edited_at: None,
        deleted_at: None,
        reply_to_id,
    };

    ctx.db.insert_message(&msg).map_err(|e| e.to_string())?;

    let room_id = ctx
        .db
        .get_room_id_for_channel(&channel_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Channel not found".to_string())?;

    ctx.network_tx
        .send(NetworkCommand::SendMessage {
            room_id,
            message: msg.clone(),
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(msg)
}

pub fn get_messages(
    ctx: &ServiceContext,
    channel_id: &str,
    limit: Option<i64>,
    before: Option<&str>,
) -> Result<Vec<Message>, String> {
    let limit = limit.unwrap_or(50);
    ctx.db
        .get_messages(channel_id, limit, before)
        .map_err(|e| e.to_string())
}

pub fn edit_message(
    ctx: &ServiceContext,
    message_id: &str,
    new_content: &str,
) -> Result<bool, String> {
    let edited_at = Utc::now().to_rfc3339();
    let updated = ctx.db.edit_message(message_id, new_content, &edited_at)
        .map_err(|e| e.to_string())?;
    if updated {
        let _ = ctx.event_tx.send(AppEvent::MessageEdited {
            message_id: message_id.to_string(),
            channel_id: String::new(), // caller should provide
            new_content: new_content.to_string(),
            edited_at,
        });
    }
    Ok(updated)
}

pub fn delete_message(
    ctx: &ServiceContext,
    message_id: &str,
) -> Result<bool, String> {
    let deleted_at = Utc::now().to_rfc3339();
    let deleted = ctx.db.delete_message(message_id, &deleted_at)
        .map_err(|e| e.to_string())?;
    if deleted {
        let _ = ctx.event_tx.send(AppEvent::MessageDeleted {
            message_id: message_id.to_string(),
            channel_id: String::new(),
        });
    }
    Ok(deleted)
}

pub fn add_reaction(
    ctx: &ServiceContext,
    message_id: &str,
    emoji: &str,
) -> Result<Reaction, String> {
    let reaction = Reaction {
        id: Uuid::new_v4().to_string(),
        message_id: message_id.to_string(),
        peer_id: ctx.peer_id.clone(),
        emoji: emoji.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };
    ctx.db.add_reaction(&reaction).map_err(|e| e.to_string())?;
    let _ = ctx.event_tx.send(AppEvent::ReactionAdded {
        message_id: message_id.to_string(),
        channel_id: String::new(),
        peer_id: ctx.peer_id.clone(),
        emoji: emoji.to_string(),
    });
    Ok(reaction)
}

pub fn remove_reaction(
    ctx: &ServiceContext,
    message_id: &str,
    emoji: &str,
) -> Result<bool, String> {
    let removed = ctx.db.remove_reaction(message_id, &ctx.peer_id, emoji)
        .map_err(|e| e.to_string())?;
    if removed {
        let _ = ctx.event_tx.send(AppEvent::ReactionRemoved {
            message_id: message_id.to_string(),
            channel_id: String::new(),
            peer_id: ctx.peer_id.clone(),
            emoji: emoji.to_string(),
        });
    }
    Ok(removed)
}

pub fn get_reactions(
    ctx: &ServiceContext,
    message_id: &str,
) -> Result<Vec<Reaction>, String> {
    ctx.db.get_reactions(message_id).map_err(|e| e.to_string())
}

pub fn mark_read(
    ctx: &ServiceContext,
    channel_id: &str,
    last_read_message_id: &str,
) -> Result<(), String> {
    let updated_at = Utc::now().to_rfc3339();
    ctx.db.set_read_receipt(channel_id, &ctx.peer_id, last_read_message_id, &updated_at)
        .map_err(|e| e.to_string())?;
    let _ = ctx.event_tx.send(AppEvent::ReadReceiptUpdated {
        channel_id: channel_id.to_string(),
        peer_id: ctx.peer_id.clone(),
        last_read_message_id: last_read_message_id.to_string(),
    });
    Ok(())
}

pub fn get_read_receipts(
    ctx: &ServiceContext,
    channel_id: &str,
) -> Result<Vec<crate::models::ReadReceipt>, String> {
    ctx.db.get_read_receipts(channel_id).map_err(|e| e.to_string())
}

pub fn typing_indicator(
    ctx: &ServiceContext,
    channel_id: &str,
    typing: bool,
) -> Result<(), String> {
    let display_name = ctx.db.get_display_name().map_err(|e| e.to_string())?;
    if typing {
        let _ = ctx.event_tx.send(AppEvent::TypingStarted {
            channel_id: channel_id.to_string(),
            peer_id: ctx.peer_id.clone(),
            display_name,
        });
    } else {
        let _ = ctx.event_tx.send(AppEvent::TypingStopped {
            channel_id: channel_id.to_string(),
            peer_id: ctx.peer_id.clone(),
        });
    }
    Ok(())
}

pub fn search_messages(
    ctx: &ServiceContext,
    query: &str,
    channel_id: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<SearchResult, String> {
    ctx.db.search_messages(channel_id, query, limit.unwrap_or(20), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

pub fn pin_message(
    ctx: &ServiceContext,
    channel_id: &str,
    message_id: &str,
) -> Result<crate::models::PinnedMessage, String> {
    let pin = crate::models::PinnedMessage {
        id: Uuid::new_v4().to_string(),
        channel_id: channel_id.to_string(),
        message_id: message_id.to_string(),
        pinned_by: ctx.peer_id.clone(),
        pinned_at: Utc::now().to_rfc3339(),
    };
    ctx.db.pin_message(&pin).map_err(|e| e.to_string())?;
    let _ = ctx.event_tx.send(AppEvent::MessagePinned(pin.clone()));
    Ok(pin)
}

pub fn unpin_message(
    ctx: &ServiceContext,
    channel_id: &str,
    message_id: &str,
) -> Result<bool, String> {
    let removed = ctx.db.unpin_message(message_id).map_err(|e| e.to_string())?;
    if removed {
        let _ = ctx.event_tx.send(AppEvent::MessageUnpinned {
            channel_id: channel_id.to_string(),
            message_id: message_id.to_string(),
        });
    }
    Ok(removed)
}

pub fn get_pinned_messages(
    ctx: &ServiceContext,
    channel_id: &str,
) -> Result<Vec<crate::models::PinnedMessage>, String> {
    ctx.db.get_pinned_messages(channel_id).map_err(|e| e.to_string())
}
