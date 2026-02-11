pub mod behaviour;
pub mod swarm;
pub mod bootstrap;

use crate::models::Message;

/// The global discovery topic for room lookups
pub const DISCOVERY_TOPIC: &str = "chatr/discovery";

/// Commands sent from Tauri commands to the network event loop
#[derive(Debug)]
pub enum NetworkCommand {
    SendMessage {
        room_id: String,
        message: Message,
    },
    SubscribeRoom {
        room_id: String,
    },
    PublishRoomToDHT {
        room_id: String,
        invite_code: String,
        room_name: String,
    },
    LookupRoomInDHT {
        invite_code: String,
        reply: tokio::sync::oneshot::Sender<Option<(String, String)>>,
    },
    /// GossipSub-based room lookup (works on LAN without DHT)
    LookupRoomViaGossip {
        invite_code: String,
        reply: tokio::sync::oneshot::Sender<Option<(String, String)>>,
    },
    AnnouncePresence {
        room_id: String,
        display_name: String,
    },
    SendCallOffer {
        room_id: String,
        to_peer_id: String,
        call_id: String,
        channel_id: String,
        sdp: String,
    },
    SendCallAnswer {
        room_id: String,
        to_peer_id: String,
        call_id: String,
        channel_id: String,
        sdp: String,
    },
    SendIceCandidate {
        room_id: String,
        to_peer_id: String,
        channel_id: String,
        candidate: String,
    },
    SendVoiceState {
        room_id: String,
        channel_id: Option<String>,
        muted: bool,
        deafened: bool,
        video: bool,
        screen_sharing: bool,
    },
    BroadcastChannelCreated {
        room_id: String,
        channel_id: String,
        name: String,
        channel_type: String,
        created_at: String,
    },
    BroadcastChannelDeleted {
        room_id: String,
        channel_id: String,
    },
}
