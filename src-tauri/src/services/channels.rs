use chrono::Utc;

use crate::models::Channel;
use crate::network::NetworkCommand;
use crate::services::rooms::deterministic_channel_id;
use crate::state::ServiceContext;

pub fn create_channel(
    ctx: &ServiceContext,
    room_id: &str,
    name: &str,
    channel_type: Option<&str>,
) -> Result<Channel, String> {
    let channel = Channel {
        id: deterministic_channel_id(room_id, name),
        room_id: room_id.to_string(),
        name: name.to_string(),
        created_at: Utc::now().to_rfc3339(),
        channel_type: channel_type.unwrap_or("text").to_string(),
        topic: None,
        position: 0,
    };
    ctx.db.create_channel(&channel).map_err(|e| e.to_string())?;

    // Broadcast channel creation to other peers in this room (try_send for sync context)
    let _ = ctx.network_tx.try_send(NetworkCommand::BroadcastChannelCreated {
        room_id: channel.room_id.clone(),
        channel_id: channel.id.clone(),
        name: channel.name.clone(),
        channel_type: channel.channel_type.clone(),
        created_at: channel.created_at.clone(),
    });

    Ok(channel)
}

pub fn update_channel(
    ctx: &ServiceContext,
    channel_id: &str,
    name: Option<&str>,
    topic: Option<&str>,
    position: Option<i32>,
) -> Result<(), String> {
    ctx.db.update_channel(channel_id, name, topic, position)
        .map_err(|e| e.to_string())
}

pub fn delete_channel(ctx: &ServiceContext, channel_id: &str, room_id: Option<&str>) -> Result<(), String> {
    ctx.db.delete_channel(channel_id).map_err(|e| e.to_string())?;

    // Broadcast channel deletion if room_id is known
    if let Some(rid) = room_id {
        let _ = ctx.network_tx.try_send(NetworkCommand::BroadcastChannelDeleted {
            room_id: rid.to_string(),
            channel_id: channel_id.to_string(),
        });
    }

    Ok(())
}

pub fn get_channels(ctx: &ServiceContext, room_id: &str) -> Result<Vec<Channel>, String> {
    ctx.db.get_channels(room_id).map_err(|e| e.to_string())
}
