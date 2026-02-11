export interface Message {
  id: string;
  channel_id: string;
  sender_peer_id: string;
  sender_display_name: string;
  content: string;
  timestamp: string;
  edited_at?: string | null;
  deleted_at?: string | null;
  reply_to_id?: string | null;
}

export interface Room {
  id: string;
  name: string;
  invite_code: string;
  created_at: string;
  owner_peer_id?: string | null;
}

export interface Channel {
  id: string;
  room_id: string;
  name: string;
  created_at: string;
  channel_type?: string;
  topic?: string | null;
  position?: number;
}

export interface PeerInfo {
  peer_id: string;
  display_name: string;
  is_online: boolean;
}

export interface Identity {
  peer_id: string;
  display_name: string;
  avatar_hash?: string | null;
  status_message?: string | null;
  status_type?: string | null;
}

export interface Reaction {
  id: string;
  message_id: string;
  peer_id: string;
  emoji: string;
  created_at: string;
}

export interface DmConversation {
  id: string;
  is_group: boolean;
  name: string | null;
  created_at: string;
}

export interface DmMessage {
  id: string;
  conversation_id: string;
  sender_peer_id: string;
  sender_display_name: string;
  content: string;
  timestamp: string;
}

export interface PinnedMessage {
  id: string;
  channel_id: string;
  message_id: string;
  pinned_by: string;
  pinned_at: string;
}

export interface RoomRole {
  id: string;
  room_id: string;
  peer_id: string;
  role: string;
  assigned_by: string;
  assigned_at: string;
}

export interface Friend {
  peer_id: string;
  display_name: string;
  status: string;
  created_at: string;
}

export interface SearchResult {
  messages: Message[];
  total: number;
}

export interface VoiceParticipant {
  peerId: string;
  displayName: string;
  muted: boolean;
  deafened: boolean;
  video: boolean;
  screenSharing: boolean;
  speaking: boolean;
}
