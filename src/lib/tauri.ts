import { invoke } from "@tauri-apps/api/core";
import type {
  Message,
  Room,
  Channel,
  PeerInfo,
  Identity,
} from "./types";

// Identity
export async function getMyPeerId(): Promise<string> {
  return invoke("get_my_peer_id");
}

export async function getIdentity(): Promise<Identity> {
  return invoke("get_identity");
}

export async function getDisplayName(): Promise<string> {
  return invoke("get_display_name");
}

export async function setDisplayName(name: string): Promise<void> {
  return invoke("set_display_name", { name });
}

export async function setStatus(statusMessage?: string, statusType?: string): Promise<void> {
  return invoke("set_status", { statusMessage, statusType });
}

// Rooms
export async function createRoom(name: string): Promise<Room> {
  return invoke("create_room", { name });
}

export async function joinRoom(inviteCode: string): Promise<Room> {
  return invoke("join_room", { inviteCode });
}

export async function listRooms(): Promise<Room[]> {
  return invoke("list_rooms");
}

// Channels
export async function getChannels(roomId: string): Promise<Channel[]> {
  return invoke("get_channels", { roomId });
}

// Messages
export async function sendMessage(
  channelId: string,
  content: string,
  replyToId?: string | null
): Promise<Message> {
  return invoke("send_message", { channelId, content, replyToId });
}

export async function getMessages(
  channelId: string,
  limit?: number,
  before?: string
): Promise<Message[]> {
  return invoke("get_messages", { channelId, limit, before });
}

// Peers
export async function getRoomPeers(roomId: string): Promise<PeerInfo[]> {
  return invoke("get_room_peers", { roomId });
}
