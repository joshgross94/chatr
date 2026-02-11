use serde::Serialize;
use tokio::sync::broadcast;

use crate::models::{Message, PeerInfo, PinnedMessage, DmMessage};

/// Transport-agnostic application events.
/// Emitted by the network swarm, consumed by Tauri bridge and WebSocket API.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    // Phase 0
    NewMessage(Message),
    PeerConnected(PeerInfo),
    PeerDiscovered(PeerInfo),
    PeerDisconnected { peer_id: String },
    PeerJoinedRoom { room_id: String, peer: PeerInfo },
    PeerLeftRoom { room_id: String, peer_id: String },
    // Phase 1
    MessageEdited { message_id: String, channel_id: String, new_content: String, edited_at: String },
    MessageDeleted { message_id: String, channel_id: String },
    ReactionAdded { message_id: String, channel_id: String, peer_id: String, emoji: String },
    ReactionRemoved { message_id: String, channel_id: String, peer_id: String, emoji: String },
    TypingStarted { channel_id: String, peer_id: String, display_name: String },
    TypingStopped { channel_id: String, peer_id: String },
    ReadReceiptUpdated { channel_id: String, peer_id: String, last_read_message_id: String },
    // Phase 2
    MessagePinned(PinnedMessage),
    MessageUnpinned { channel_id: String, message_id: String },
    NewDmMessage(DmMessage),
    // Phase 5
    FriendRequestReceived { from_peer_id: String, from_display_name: String },
    FriendRequestAccepted { peer_id: String },
    // Voice/Video
    CallOfferReceived { call_id: String, from_peer_id: String, channel_id: String, sdp: String },
    CallAnswerReceived { call_id: String, from_peer_id: String, channel_id: String, sdp: String },
    IceCandidateReceived { from_peer_id: String, channel_id: String, candidate: String },
    VoiceStateChanged { peer_id: String, display_name: String, channel_id: Option<String>, room_id: String, muted: bool, deafened: bool, video: bool, screen_sharing: bool },
    // Voice connections (media engine)
    VoiceConnected { peer_id: String },
    VoiceDisconnected { peer_id: String },
    SpeakingChanged { peer_id: String, speaking: bool },
    // Channel sync
    ChannelCreated { room_id: String, channel_id: String, name: String, channel_type: String, created_at: String },
    ChannelDeleted { room_id: String, channel_id: String },
}

pub type EventSender = broadcast::Sender<AppEvent>;
pub type EventReceiver = broadcast::Receiver<AppEvent>;

pub fn create_event_bus() -> (EventSender, EventReceiver) {
    broadcast::channel(256)
}
