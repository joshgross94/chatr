import { create } from "zustand";
import { listRooms, getChannels, createRoom, joinRoom } from "../lib/tauri";
import type { Room, Channel } from "../lib/types";

interface ChatState {
  rooms: Room[];
  channels: Channel[];
  selectedRoomId: string | null;
  selectedChannelId: string | null;
  isLoading: boolean;

  loadRooms: () => Promise<void>;
  selectRoom: (roomId: string) => Promise<void>;
  selectChannel: (channelId: string) => void;
  addRoom: (name: string) => Promise<Room>;
  joinRoomByInvite: (inviteCode: string) => Promise<Room>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  rooms: [],
  channels: [],
  selectedRoomId: null,
  selectedChannelId: null,
  isLoading: false,

  loadRooms: async () => {
    const rooms = await listRooms();
    set({ rooms });
    // If we have rooms but none selected, select the first one
    if (rooms.length > 0 && !get().selectedRoomId) {
      await get().selectRoom(rooms[0].id);
    }
  },

  selectRoom: async (roomId: string) => {
    set({ selectedRoomId: roomId, isLoading: true });
    const channels = await getChannels(roomId);
    set({
      channels,
      isLoading: false,
      selectedChannelId: channels.length > 0 ? channels[0].id : null,
    });
  },

  selectChannel: (channelId: string) => {
    set({ selectedChannelId: channelId });
  },

  addRoom: async (name: string) => {
    const room = await createRoom(name);
    const rooms = [...get().rooms, room];
    set({ rooms });
    await get().selectRoom(room.id);
    return room;
  },

  joinRoomByInvite: async (inviteCode: string) => {
    const room = await joinRoom(inviteCode);
    const existingRoom = get().rooms.find((r) => r.id === room.id);
    if (!existingRoom) {
      const rooms = [...get().rooms, room];
      set({ rooms });
    }
    await get().selectRoom(room.id);
    return room;
  },
}));
