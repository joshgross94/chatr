import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { VoiceParticipant } from "../lib/types";

interface VoiceState {
  currentChannelId: string | null;
  currentRoomId: string | null;
  // Participants in OUR voice channel
  participants: Record<string, VoiceParticipant>;
  // ALL voice channel participants across all channels in the room (channelId -> peerId -> participant)
  channelParticipants: Record<string, Record<string, VoiceParticipant>>;
  isMuted: boolean;
  isDeafened: boolean;
  isCameraEnabled: boolean;
  isScreenSharing: boolean;
  speakingPeers: Set<string>;

  joinVoiceChannel: (roomId: string, channelId: string) => Promise<void>;
  leaveVoiceChannel: () => Promise<void>;

  handleVoiceStateChanged: (payload: {
    peer_id: string;
    display_name: string;
    channel_id: string | null;
    room_id: string;
    muted: boolean;
    deafened: boolean;
    video: boolean;
    screen_sharing: boolean;
  }) => void;

  handleVoiceConnected: (payload: { peer_id: string }) => void;
  handleVoiceDisconnected: (payload: { peer_id: string }) => void;
  handleSpeakingChanged: (payload: {
    peer_id: string;
    speaking: boolean;
  }) => void;

  toggleMute: () => Promise<void>;
  toggleDeafen: () => Promise<void>;
  toggleCamera: () => Promise<void>;
  toggleScreenShare: () => Promise<void>;
}

export const useVoiceStore = create<VoiceState>((set, get) => ({
  currentChannelId: null,
  currentRoomId: null,
  participants: {},
  channelParticipants: {},
  isMuted: false,
  isDeafened: false,
  isCameraEnabled: false,
  isScreenSharing: false,
  speakingPeers: new Set(),

  joinVoiceChannel: async (roomId: string, channelId: string) => {
    const state = get();

    // If already in this channel, do nothing
    if (state.currentChannelId === channelId) return;

    // If in another channel, leave first
    if (state.currentChannelId) {
      await get().leaveVoiceChannel();
    }

    // Tell Rust media engine to join voice
    await invoke("join_voice_channel", { roomId, channelId });

    set({
      currentChannelId: channelId,
      currentRoomId: roomId,
      isMuted: false,
      isDeafened: false,
      isCameraEnabled: false,
      isScreenSharing: false,
      participants: {},
      speakingPeers: new Set(),
    });
  },

  leaveVoiceChannel: async () => {
    // Tell Rust media engine to leave voice
    await invoke("leave_voice_channel").catch(console.error);

    set({
      currentChannelId: null,
      currentRoomId: null,
      participants: {},
      isMuted: false,
      isDeafened: false,
      isCameraEnabled: false,
      isScreenSharing: false,
      speakingPeers: new Set(),
    });
  },

  handleVoiceStateChanged: (payload) => {
    const state = get();

    if (payload.channel_id) {
      // Peer is in a voice channel — update global tracking
      set((s) => {
        const channelParticipants = { ...s.channelParticipants };
        const channelPeers = {
          ...(channelParticipants[payload.channel_id!] ?? {}),
        };

        channelPeers[payload.peer_id] = {
          peerId: payload.peer_id,
          displayName: payload.display_name,
          muted: payload.muted,
          deafened: payload.deafened,
          video: payload.video,
          screenSharing: payload.screen_sharing,
          speaking: false,
        };
        channelParticipants[payload.channel_id!] = channelPeers;

        // Remove from any other channel they were previously in
        for (const chId of Object.keys(channelParticipants)) {
          if (
            chId !== payload.channel_id &&
            channelParticipants[chId]?.[payload.peer_id]
          ) {
            const { [payload.peer_id]: _, ...rest } = channelParticipants[chId];
            channelParticipants[chId] = rest;
          }
        }

        return { channelParticipants };
      });

      // If they're in OUR channel, update participants
      // (WebRTC negotiation is now handled by the Rust media engine)
      if (payload.channel_id === state.currentChannelId) {
        set((s) => ({
          participants: {
            ...s.participants,
            [payload.peer_id]: {
              peerId: payload.peer_id,
              displayName: payload.display_name,
              muted: payload.muted,
              deafened: payload.deafened,
              video: payload.video,
              screenSharing: payload.screen_sharing,
              speaking: false,
            },
          },
        }));
      }
    } else {
      // Peer left voice (channel_id is null) — remove from all tracking
      set((s) => {
        const channelParticipants = { ...s.channelParticipants };
        for (const chId of Object.keys(channelParticipants)) {
          if (channelParticipants[chId]?.[payload.peer_id]) {
            const { [payload.peer_id]: _, ...rest } = channelParticipants[chId];
            channelParticipants[chId] = rest;
          }
        }

        // Remove from local participants
        const { [payload.peer_id]: _, ...restParticipants } = s.participants;

        return { channelParticipants, participants: restParticipants };
      });
    }
  },

  handleVoiceConnected: (payload) => {
    console.log(`Voice WebRTC connected to peer: ${payload.peer_id}`);
  },

  handleVoiceDisconnected: (payload) => {
    console.log(`Voice WebRTC disconnected from peer: ${payload.peer_id}`);
  },

  handleSpeakingChanged: (payload) => {
    set((s) => {
      const next = new Set(s.speakingPeers);
      if (payload.speaking) {
        next.add(payload.peer_id);
      } else {
        next.delete(payload.peer_id);
      }
      return { speakingPeers: next };
    });
  },

  toggleMute: async () => {
    const state = get();
    if (!state.currentRoomId) return;

    const nowMuted = !state.isMuted;
    set({ isMuted: nowMuted });

    await invoke("set_muted", { muted: nowMuted }).catch(console.error);
  },

  toggleDeafen: async () => {
    const state = get();
    if (!state.currentRoomId) return;

    const nowDeafened = !state.isDeafened;
    set({ isDeafened: nowDeafened });

    await invoke("set_deafened", { deafened: nowDeafened }).catch(
      console.error
    );
  },

  toggleCamera: async () => {
    const state = get();
    if (!state.currentChannelId) return;

    if (state.isCameraEnabled) {
      await invoke("disable_camera").catch(console.error);
      set({ isCameraEnabled: false });
    } else {
      await invoke("enable_camera", { deviceIndex: null }).catch(console.error);
      set({ isCameraEnabled: true });
    }
  },

  toggleScreenShare: async () => {
    const state = get();
    if (!state.currentChannelId) return;

    if (state.isScreenSharing) {
      await invoke("stop_screen_share").catch(console.error);
      set({ isScreenSharing: false });
    } else {
      await invoke("start_screen_share").catch(console.error);
      set({ isScreenSharing: true });
    }
  },
}));
