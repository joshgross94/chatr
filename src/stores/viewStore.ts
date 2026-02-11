import { create } from "zustand";

interface ViewState {
  mode: "rooms" | "dms" | "friends";
  setMode: (mode: "rooms" | "dms" | "friends") => void;
  searchOpen: boolean;
  setSearchOpen: (open: boolean) => void;
}

export const useViewStore = create<ViewState>((set) => ({
  mode: "rooms",
  setMode: (mode) => set({ mode }),
  searchOpen: false,
  setSearchOpen: (open) => set({ searchOpen: open }),
}));
