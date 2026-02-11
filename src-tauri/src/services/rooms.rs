use chrono::Utc;
use uuid::Uuid;

use crate::models::{Channel, Room};
use crate::network::NetworkCommand;
use crate::state::ServiceContext;

fn generate_invite_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKMNPQRSTUVWXYZ23456789".chars().collect();
    (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

/// Generate a deterministic channel ID from room_id + channel name
/// so all peers create the same channel ID for the same room/channel pair.
pub fn deterministic_channel_id(room_id: &str, channel_name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    room_id.hash(&mut hasher);
    channel_name.hash(&mut hasher);
    let hash = hasher.finish();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hash >> 32) as u32,
        (hash >> 16) as u16 & 0xffff,
        hash as u16,
        (hash >> 48) as u16,
        hash & 0xffffffffffff
    )
}

pub async fn create_room(ctx: &ServiceContext, name: String) -> Result<Room, String> {
    let room_id = Uuid::new_v4().to_string();
    let invite_code = generate_invite_code();
    let now = Utc::now().to_rfc3339();

    let room = Room {
        id: room_id.clone(),
        name: name.clone(),
        invite_code: invite_code.clone(),
        created_at: now.clone(),
        owner_peer_id: Some(ctx.peer_id.clone()),
    };

    ctx.db.create_room(&room).map_err(|e| e.to_string())?;

    // Auto-create #general channel with deterministic ID
    let channel = Channel {
        id: deterministic_channel_id(&room_id, "general"),
        room_id: room_id.clone(),
        name: "general".to_string(),
        created_at: now,
        channel_type: "text".to_string(),
        topic: None,
        position: 0,
    };
    ctx.db
        .create_channel(&channel)
        .map_err(|e| e.to_string())?;

    // Subscribe to the room's topics in the network
    ctx.network_tx
        .send(NetworkCommand::SubscribeRoom {
            room_id: room_id.clone(),
        })
        .await
        .map_err(|e| e.to_string())?;

    // Publish room info to DHT for discovery
    ctx.network_tx
        .send(NetworkCommand::PublishRoomToDHT {
            room_id: room_id.clone(),
            invite_code: invite_code.clone(),
            room_name: name,
        })
        .await
        .map_err(|e| e.to_string())?;

    Ok(room)
}

pub async fn join_room(ctx: &ServiceContext, invite_code: String) -> Result<Room, String> {
    // Check if we already have this room locally
    if let Some(room) = ctx
        .db
        .get_room_by_invite(&invite_code)
        .map_err(|e| e.to_string())?
    {
        return Ok(room);
    }

    // Try GossipSub-based lookup first (works on LAN without DHT)
    let (tx, rx) = tokio::sync::oneshot::channel();
    ctx.network_tx
        .send(NetworkCommand::LookupRoomViaGossip {
            invite_code: invite_code.clone(),
            reply: tx,
        })
        .await
        .map_err(|e| e.to_string())?;

    // Wait up to 3 seconds for a GossipSub response
    let room_info =
        match tokio::time::timeout(std::time::Duration::from_secs(3), rx).await {
            Ok(Ok(info)) => info,
            _ => {
                // GossipSub lookup timed out or failed, try DHT
                let (tx2, rx2) = tokio::sync::oneshot::channel();
                ctx.network_tx
                    .send(NetworkCommand::LookupRoomInDHT {
                        invite_code: invite_code.clone(),
                        reply: tx2,
                    })
                    .await
                    .map_err(|e| e.to_string())?;

                match tokio::time::timeout(std::time::Duration::from_secs(5), rx2).await {
                    Ok(Ok(info)) => info,
                    _ => None,
                }
            }
        };

    match room_info {
        Some((room_id, room_name)) => {
            let now = Utc::now().to_rfc3339();
            let room = Room {
                id: room_id.clone(),
                name: room_name,
                invite_code: invite_code.clone(),
                created_at: now.clone(),
                owner_peer_id: None,
            };
            ctx.db.create_room(&room).map_err(|e| e.to_string())?;

            // Create #general channel with deterministic ID (matches room creator)
            let channel = Channel {
                id: deterministic_channel_id(&room_id, "general"),
                room_id: room_id.clone(),
                name: "general".to_string(),
                created_at: now,
                channel_type: "text".to_string(),
                topic: None,
                position: 0,
            };
            ctx.db
                .create_channel(&channel)
                .map_err(|e| e.to_string())?;

            // Subscribe to room topics
            ctx.network_tx
                .send(NetworkCommand::SubscribeRoom { room_id })
                .await
                .map_err(|e| e.to_string())?;

            Ok(room)
        }
        None => Err("Room not found. Make sure someone with the room is online.".to_string()),
    }
}

pub fn list_rooms(ctx: &ServiceContext) -> Result<Vec<Room>, String> {
    ctx.db.list_rooms().map_err(|e| e.to_string())
}

pub fn get_channels(ctx: &ServiceContext, room_id: &str) -> Result<Vec<Channel>, String> {
    ctx.db.get_channels(room_id).map_err(|e| e.to_string())
}
