import { invoke } from "@tauri-apps/api/core";
import type {
  Message,
  Room,
  Channel,
  PeerInfo,
  Identity,
  Reaction,
  DmConversation,
  DmMessage,
  PinnedMessage,
  RoomRole,
  Friend,
  SearchResult,
} from "./types";

let _apiPort: number | null = null;

async function getApiPort(): Promise<number> {
  if (_apiPort) return _apiPort;
  _apiPort = await invoke<number>("get_api_port");
  return _apiPort;
}

async function api<T>(path: string, options?: RequestInit): Promise<T> {
  const port = await getApiPort();
  const res = await fetch(`http://127.0.0.1:${port}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `API error ${res.status}`);
  }
  const text = await res.text();
  return text ? JSON.parse(text) : ({} as T);
}

// ============================================================
// Identity
// ============================================================
export const identity = {
  get: () => api<Identity>("/api/v1/identity"),
  setDisplayName: (name: string) =>
    api<void>("/api/v1/identity/display-name", {
      method: "PUT",
      body: JSON.stringify({ name }),
    }),
  setStatus: (status_message?: string, status_type?: string) =>
    api<void>("/api/v1/identity/status", {
      method: "PUT",
      body: JSON.stringify({ status_message, status_type }),
    }),
};

// ============================================================
// Rooms
// ============================================================
export const rooms = {
  list: () => api<Room[]>("/api/v1/rooms"),
  create: (name: string) =>
    api<Room>("/api/v1/rooms", { method: "POST", body: JSON.stringify({ name }) }),
  join: (invite_code: string) =>
    api<Room>("/api/v1/rooms/join", { method: "POST", body: JSON.stringify({ invite_code }) }),
  getChannels: (roomId: string) => api<Channel[]>(`/api/v1/rooms/${roomId}/channels`),
  getPeers: (roomId: string) => api<PeerInfo[]>(`/api/v1/rooms/${roomId}/peers`),
  getRoles: (roomId: string) => api<RoomRole[]>(`/api/v1/rooms/${roomId}/roles`),
  setRole: (roomId: string, peer_id: string, role: string) =>
    api<RoomRole>(`/api/v1/rooms/${roomId}/roles`, {
      method: "POST",
      body: JSON.stringify({ peer_id, role }),
    }),
  removeRole: (roomId: string, peerId: string) =>
    api<void>(`/api/v1/rooms/${roomId}/roles/${peerId}`, { method: "DELETE" }),
  moderate: (roomId: string, action_type: string, target_peer_id: string, reason?: string) =>
    api<void>(`/api/v1/rooms/${roomId}/moderate`, {
      method: "POST",
      body: JSON.stringify({ action_type, target_peer_id, reason }),
    }),
  getAuditLog: (roomId: string) => api<any[]>(`/api/v1/rooms/${roomId}/audit-log`),
};

// ============================================================
// Channels
// ============================================================
export const channels = {
  create: (roomId: string, name: string, channel_type?: string) =>
    api<Channel>(`/api/v1/rooms/${roomId}/channels`, {
      method: "POST",
      body: JSON.stringify({ name, channel_type: channel_type ?? "text" }),
    }),
  update: (channelId: string, name?: string, topic?: string) =>
    api<void>(`/api/v1/channels/${channelId}`, {
      method: "PUT",
      body: JSON.stringify({ name, topic }),
    }),
  delete: (channelId: string) =>
    api<void>(`/api/v1/channels/${channelId}`, { method: "DELETE" }),
};

// ============================================================
// Messages
// ============================================================
export const messages = {
  list: (channelId: string, limit?: number, before?: string) => {
    const params = new URLSearchParams();
    if (limit) params.set("limit", String(limit));
    if (before) params.set("before", before);
    const qs = params.toString();
    return api<Message[]>(`/api/v1/channels/${channelId}/messages${qs ? `?${qs}` : ""}`);
  },
  send: (channelId: string, content: string, reply_to_id?: string) =>
    api<Message>(`/api/v1/channels/${channelId}/messages`, {
      method: "POST",
      body: JSON.stringify({ content, reply_to_id }),
    }),
  edit: (messageId: string, content: string) =>
    api<void>(`/api/v1/messages/${messageId}`, {
      method: "PUT",
      body: JSON.stringify({ content }),
    }),
  delete: (messageId: string) =>
    api<void>(`/api/v1/messages/${messageId}`, { method: "DELETE" }),
  // Reactions
  getReactions: (messageId: string) => api<Reaction[]>(`/api/v1/messages/${messageId}/reactions`),
  addReaction: (messageId: string, emoji: string) =>
    api<Reaction>(`/api/v1/messages/${messageId}/reactions`, {
      method: "POST",
      body: JSON.stringify({ emoji }),
    }),
  removeReaction: (messageId: string, emoji: string) =>
    api<void>(`/api/v1/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`, {
      method: "DELETE",
    }),
  // Typing
  sendTyping: (channelId: string, typing: boolean) =>
    api<void>(`/api/v1/channels/${channelId}/typing`, {
      method: "POST",
      body: JSON.stringify({ typing }),
    }),
  // Read
  markRead: (channelId: string, last_read_message_id: string) =>
    api<void>(`/api/v1/channels/${channelId}/read`, {
      method: "POST",
      body: JSON.stringify({ last_read_message_id }),
    }),
  // Pins
  getPinned: (channelId: string) => api<PinnedMessage[]>(`/api/v1/channels/${channelId}/pins`),
  pin: (channelId: string, message_id: string) =>
    api<PinnedMessage>(`/api/v1/channels/${channelId}/pins`, {
      method: "POST",
      body: JSON.stringify({ message_id }),
    }),
  unpin: (channelId: string, messageId: string) =>
    api<void>(`/api/v1/channels/${channelId}/pins/${messageId}`, { method: "DELETE" }),
  // Search
  search: (query: string, channel_id?: string, limit?: number) => {
    const params = new URLSearchParams({ q: query });
    if (channel_id) params.set("channel_id", channel_id);
    if (limit) params.set("limit", String(limit));
    return api<SearchResult>(`/api/v1/search/messages?${params}`);
  },
};

// ============================================================
// DMs
// ============================================================
export const dms = {
  list: () => api<DmConversation[]>("/api/v1/dms"),
  create: (peer_ids: string[], name?: string) =>
    api<DmConversation>("/api/v1/dms", {
      method: "POST",
      body: JSON.stringify({ peer_ids, name }),
    }),
  getParticipants: (conversationId: string) =>
    api<any[]>(`/api/v1/dms/${conversationId}/participants`),
  getMessages: (conversationId: string) =>
    api<DmMessage[]>(`/api/v1/dms/${conversationId}/messages`),
  sendMessage: (conversationId: string, content: string) =>
    api<DmMessage>(`/api/v1/dms/${conversationId}/messages`, {
      method: "POST",
      body: JSON.stringify({ content }),
    }),
};

// ============================================================
// Friends
// ============================================================
export const friends = {
  list: () => api<Friend[]>("/api/v1/friends"),
  get: (peerId: string) => api<Friend>(`/api/v1/friends/${peerId}`),
  sendRequest: (peer_id: string) =>
    api<Friend>("/api/v1/friends", {
      method: "POST",
      body: JSON.stringify({ peer_id }),
    }),
  accept: (peerId: string) =>
    api<void>(`/api/v1/friends/${peerId}/accept`, { method: "POST" }),
  remove: (peerId: string) =>
    api<void>(`/api/v1/friends/${peerId}`, { method: "DELETE" }),
};

// ============================================================
// Settings
// ============================================================
export const settings = {
  getAll: () => api<any[]>("/api/v1/settings"),
  get: (key: string) => api<any>(`/api/v1/settings/${key}`),
  set: (key: string, value: string) =>
    api<void>(`/api/v1/settings/${key}`, {
      method: "PUT",
      body: JSON.stringify({ value }),
    }),
  delete: (key: string) =>
    api<void>(`/api/v1/settings/${key}`, { method: "DELETE" }),
};

// ============================================================
// Blocked
// ============================================================
export const blocked = {
  list: () => api<any[]>("/api/v1/blocked"),
  block: (peer_id: string) =>
    api<void>("/api/v1/blocked", {
      method: "POST",
      body: JSON.stringify({ peer_id }),
    }),
  unblock: (peerId: string) =>
    api<void>(`/api/v1/blocked/${peerId}`, { method: "DELETE" }),
};
