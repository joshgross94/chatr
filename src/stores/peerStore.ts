import { create } from "zustand";
import { getRoomPeers } from "../lib/tauri";
import type { PeerInfo } from "../lib/types";

interface PeerState {
  peers: PeerInfo[];
  loadPeers: (roomId: string) => Promise<void>;
  addPeer: (peer: PeerInfo) => void;
  removePeer: (peerId: string) => void;
  updatePeer: (peer: PeerInfo) => void;
}

export const usePeerStore = create<PeerState>((set, get) => ({
  peers: [],

  loadPeers: async (roomId: string) => {
    const peers = await getRoomPeers(roomId);
    set({ peers });
  },

  addPeer: (peer: PeerInfo) => {
    const state = get();
    const exists = state.peers.some((p) => p.peer_id === peer.peer_id);
    if (!exists) {
      set({ peers: [...state.peers, peer] });
    } else {
      // Update existing peer
      set({
        peers: state.peers.map((p) =>
          p.peer_id === peer.peer_id ? { ...p, ...peer, is_online: true } : p
        ),
      });
    }
  },

  removePeer: (peerId: string) => {
    set((state) => ({
      peers: state.peers.map((p) =>
        p.peer_id === peerId ? { ...p, is_online: false } : p
      ),
    }));
  },

  updatePeer: (peer: PeerInfo) => {
    set((state) => ({
      peers: state.peers.map((p) =>
        p.peer_id === peer.peer_id ? peer : p
      ),
    }));
  },
}));
