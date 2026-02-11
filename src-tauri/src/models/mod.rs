use serde::{Deserialize, Serialize};

// ============================================================
// Core Models (Phase 0)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub sender_peer_id: String,
    pub sender_display_name: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub invite_code: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub room_id: String,
    pub name: String,
    pub created_at: String,
    #[serde(default = "default_channel_type")]
    pub channel_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default)]
    pub position: i32,
}

fn default_channel_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub display_name: String,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub peer_id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_type: Option<String>,
}

// ============================================================
// Phase 1: Reactions, Read Receipts, Search
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: String,
    pub message_id: String,
    pub peer_id: String,
    pub emoji: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    pub channel_id: String,
    pub peer_id: String,
    pub last_read_message_id: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub messages: Vec<Message>,
    pub total: i64,
}

// ============================================================
// Phase 2: DMs, Roles, Moderation, Pins
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmConversation {
    pub id: String,
    pub is_group: bool,
    pub name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmParticipant {
    pub conversation_id: String,
    pub peer_id: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMessage {
    pub id: String,
    pub conversation_id: String,
    pub sender_peer_id: String,
    pub sender_display_name: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomRole {
    pub id: String,
    pub room_id: String,
    pub peer_id: String,
    pub role: String, // "owner", "admin", "moderator", "member"
    pub assigned_by: String,
    pub assigned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationAction {
    pub id: String,
    pub room_id: String,
    pub action_type: String, // "kick", "ban", "mute", "warn"
    pub target_peer_id: String,
    pub moderator_peer_id: String,
    pub reason: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessage {
    pub id: String,
    pub channel_id: String,
    pub message_id: String,
    pub pinned_by: String,
    pub pinned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedPeer {
    pub peer_id: String,
    pub blocked_at: String,
}

// ============================================================
// Phase 4: File Sharing
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub filename: String,
    pub size: i64,
    pub mime_type: String,
    pub sha256_hash: String,
    pub chunk_count: i32,
    pub uploader_peer_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub message_id: String,
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub site_name: Option<String>,
}

// ============================================================
// Phase 5: Friends
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub peer_id: String,
    pub display_name: String,
    pub status: String, // "pending_outgoing", "pending_incoming", "accepted", "blocked"
    pub created_at: String,
}

// ============================================================
// Phase 6: Settings, Custom Emoji
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEmoji {
    pub id: String,
    pub room_id: String,
    pub name: String,
    pub file_hash: String,
    pub uploaded_by: String,
    pub created_at: String,
}

// ============================================================
// Phase 7: Notification Settings
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSetting {
    pub target_id: String,   // channel_id or room_id
    pub target_type: String, // "channel" or "room"
    pub level: String,       // "all", "mentions", "none"
}

// ============================================================
// Network Messages (over GossipSub)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub channel_id: String,
    pub sender_peer_id: String,
    pub sender_display_name: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAnnouncement {
    pub peer_id: String,
    pub display_name: String,
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomLookupRequest {
    pub invite_code: String,
    pub requester_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomLookupResponse {
    pub invite_code: String,
    pub room_id: String,
    pub room_name: String,
    pub target_peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEditNet {
    pub message_id: String,
    pub channel_id: String,
    pub sender_peer_id: String,
    pub new_content: String,
    pub edited_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteNet {
    pub message_id: String,
    pub channel_id: String,
    pub sender_peer_id: String,
    pub deleted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionNet {
    pub message_id: String,
    pub channel_id: String,
    pub peer_id: String,
    pub emoji: String,
    pub add: bool, // true = add, false = remove
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicatorNet {
    pub channel_id: String,
    pub peer_id: String,
    pub display_name: String,
    pub typing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceiptNet {
    pub channel_id: String,
    pub peer_id: String,
    pub last_read_message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMessageNet {
    pub id: String,
    pub conversation_id: String,
    pub sender_peer_id: String,
    pub sender_display_name: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRequestNet {
    pub from_peer_id: String,
    pub from_display_name: String,
    pub to_peer_id: String,
    pub action: String, // "request", "accept", "reject", "remove"
}

// ============================================================
// WebRTC Voice/Video Signaling
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallOfferNet {
    pub call_id: String,
    pub from_peer_id: String,
    pub to_peer_id: String,
    pub channel_id: String,
    pub sdp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallAnswerNet {
    pub call_id: String,
    pub from_peer_id: String,
    pub to_peer_id: String,
    pub channel_id: String,
    pub sdp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidateNet {
    pub from_peer_id: String,
    pub to_peer_id: String,
    pub channel_id: String,
    pub candidate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStateNet {
    pub peer_id: String,
    pub display_name: String,
    pub channel_id: Option<String>,
    pub room_id: String,
    pub muted: bool,
    pub deafened: bool,
    pub video: bool,
    pub screen_sharing: bool,
}

/// Wrapper for all network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    Chat(ChatMessage),
    PeerAnnounce(PeerAnnouncement),
    RoomLookup(RoomLookupRequest),
    RoomFound(RoomLookupResponse),
    MessageEdit(MessageEditNet),
    MessageDelete(MessageDeleteNet),
    Reaction(ReactionNet),
    TypingIndicator(TypingIndicatorNet),
    ReadReceipt(ReadReceiptNet),
    DmMessage(DmMessageNet),
    FriendRequest(FriendRequestNet),
    CallOffer(CallOfferNet),
    CallAnswer(CallAnswerNet),
    IceCandidate(IceCandidateNet),
    VoiceState(VoiceStateNet),
    ChannelCreated(ChannelCreatedNet),
    ChannelDeleted(ChannelDeletedNet),
    ChannelSync { room_id: String, channels: Vec<ChannelSyncNet> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCreatedNet {
    pub room_id: String,
    pub channel_id: String,
    pub name: String,
    pub channel_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeletedNet {
    pub room_id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSyncNet {
    pub channel_id: String,
    pub name: String,
    pub channel_type: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(default)]
    pub position: i32,
}
