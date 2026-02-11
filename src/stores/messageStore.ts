import { create } from "zustand";
import { getMessages, sendMessage as sendMsg } from "../lib/tauri";
import type { Message, Reaction } from "../lib/types";

interface MessageState {
  messages: Message[];
  isLoading: boolean;
  currentChannelId: string | null;
  // Typing indicators: map of channel_id -> array of {peer_id, display_name, timeout}
  typingPeers: Record<string, { peer_id: string; display_name: string }[]>;
  // Reactions: map of message_id -> Reaction[]
  reactions: Record<string, Reaction[]>;
  // Reply state
  replyingTo: Message | null;

  loadMessages: (channelId: string) => Promise<void>;
  sendMessage: (channelId: string, content: string) => Promise<void>;
  addMessage: (message: Message) => void;
  loadMoreMessages: () => Promise<boolean>;
  editMessage: (messageId: string, newContent: string, editedAt: string) => void;
  deleteMessage: (messageId: string) => void;
  addReaction: (messageId: string, reaction: Reaction) => void;
  removeReaction: (messageId: string, peerId: string, emoji: string) => void;
  setTyping: (channelId: string, peerId: string, displayName: string, isTyping: boolean) => void;
  setReplyingTo: (message: Message | null) => void;
}

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: [],
  isLoading: false,
  currentChannelId: null,
  typingPeers: {},
  reactions: {},
  replyingTo: null,

  loadMessages: async (channelId: string) => {
    set({ isLoading: true, currentChannelId: channelId });
    const messages = await getMessages(channelId, 50);
    set({ messages, isLoading: false });
  },

  sendMessage: async (channelId: string, content: string) => {
    const replyToId = get().replyingTo?.id ?? undefined;
    const msg = await sendMsg(channelId, content, replyToId);
    set((state) => ({
      messages: [...state.messages, msg],
      replyingTo: null,
    }));
  },

  addMessage: (message: Message) => {
    const state = get();
    if (message.channel_id === state.currentChannelId) {
      const exists = state.messages.some((m) => m.id === message.id);
      if (!exists) {
        set({ messages: [...state.messages, message] });
      }
    }
  },

  loadMoreMessages: async () => {
    const state = get();
    if (!state.currentChannelId || state.messages.length === 0) return false;

    const oldest = state.messages[0];
    const older = await getMessages(state.currentChannelId, 50, oldest.timestamp);
    if (older.length === 0) return false;

    set({ messages: [...older, ...state.messages] });
    return true;
  },

  editMessage: (messageId: string, newContent: string, editedAt: string) => {
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, content: newContent, edited_at: editedAt } : m
      ),
    }));
  },

  deleteMessage: (messageId: string) => {
    set((state) => ({
      messages: state.messages.filter((m) => m.id !== messageId),
    }));
  },

  addReaction: (messageId: string, reaction: Reaction) => {
    set((state) => {
      const existing = state.reactions[messageId] ?? [];
      const alreadyExists = existing.some(
        (r) => r.peer_id === reaction.peer_id && r.emoji === reaction.emoji
      );
      if (alreadyExists) return state;
      return {
        reactions: {
          ...state.reactions,
          [messageId]: [...existing, reaction],
        },
      };
    });
  },

  removeReaction: (messageId: string, peerId: string, emoji: string) => {
    set((state) => {
      const existing = state.reactions[messageId] ?? [];
      return {
        reactions: {
          ...state.reactions,
          [messageId]: existing.filter(
            (r) => !(r.peer_id === peerId && r.emoji === emoji)
          ),
        },
      };
    });
  },

  setTyping: (channelId: string, peerId: string, displayName: string, isTyping: boolean) => {
    set((state) => {
      const current = state.typingPeers[channelId] ?? [];
      if (isTyping) {
        const exists = current.some((t) => t.peer_id === peerId);
        if (exists) return state;
        return {
          typingPeers: {
            ...state.typingPeers,
            [channelId]: [...current, { peer_id: peerId, display_name: displayName }],
          },
        };
      } else {
        return {
          typingPeers: {
            ...state.typingPeers,
            [channelId]: current.filter((t) => t.peer_id !== peerId),
          },
        };
      }
    });
  },

  setReplyingTo: (message: Message | null) => {
    set({ replyingTo: message });
  },
}));
