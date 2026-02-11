import { create } from "zustand";
import { getIdentity, setDisplayName as setName, setStatus as setStatusApi } from "../lib/tauri";
import type { Identity } from "../lib/types";

interface IdentityState {
  identity: Identity | null;
  isLoading: boolean;
  isSetup: boolean;
  loadIdentity: () => Promise<void>;
  setDisplayName: (name: string) => Promise<void>;
  setStatus: (statusMessage?: string, statusType?: string) => Promise<void>;
}

export const useIdentityStore = create<IdentityState>((set, get) => ({
  identity: null,
  isLoading: true,
  isSetup: false,

  loadIdentity: async () => {
    try {
      const identity = await getIdentity();
      const isSetup = identity.display_name !== "Anonymous";
      set({ identity, isLoading: false, isSetup });
    } catch (e) {
      console.error("Failed to load identity:", e);
      set({ isLoading: false });
    }
  },

  setDisplayName: async (name: string) => {
    await setName(name);
    const identity = get().identity;
    if (identity) {
      set({ identity: { ...identity, display_name: name }, isSetup: true });
    }
  },

  setStatus: async (statusMessage?: string, statusType?: string) => {
    await setStatusApi(statusMessage, statusType);
    const identity = get().identity;
    if (identity) {
      set({ identity: { ...identity, status_message: statusMessage, status_type: statusType } });
    }
  },
}));
