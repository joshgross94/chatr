import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useChatStore } from "../../stores/chatStore";
import { useMessageStore } from "../../stores/messageStore";
import { usePeerStore } from "../../stores/peerStore";
import { useViewStore } from "../../stores/viewStore";
import { useVoiceStore } from "../../stores/voiceStore";
import RoomSidebar from "./RoomSidebar";
import ChannelSidebar from "./ChannelSidebar";
import ChatArea from "./ChatArea";
import MemberList from "./MemberList";
import DmPanel from "../dms/DmPanel";
import FriendsPanel from "../friends/FriendsPanel";
import SearchModal from "../search/SearchModal";
import { rooms as roomsApi } from "../../lib/api";
import type { Message, PeerInfo } from "../../lib/types";

export default function AppLayout() {
  const { loadRooms, rooms } = useChatStore();
  const { addMessage, editMessage, deleteMessage, addReaction, removeReaction, setTyping } =
    useMessageStore();
  const { addPeer, removePeer } = usePeerStore();
  const { mode, setSearchOpen } = useViewStore();
  const { handleVoiceStateChanged, handleVoiceConnected, handleVoiceDisconnected, handleSpeakingChanged } =
    useVoiceStore();

  useEffect(() => {
    loadRooms();
  }, [loadRooms]);

  // Ctrl+K handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
      if (e.key === "Escape") {
        setSearchOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setSearchOpen]);

  // Listen for Tauri events from the Rust backend
  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    // Core events
    listen<Message>("new-message", (event) => {
      addMessage(event.payload);
    }).then((u) => unlisteners.push(u));

    listen<PeerInfo>("peer-connected", (event) => {
      addPeer(event.payload);
    }).then((u) => unlisteners.push(u));

    listen<PeerInfo>("peer-discovered", (event) => {
      addPeer(event.payload);
    }).then((u) => unlisteners.push(u));

    listen<{ peer_id: string }>("peer-disconnected", (event) => {
      removePeer(event.payload.peer_id);
    }).then((u) => unlisteners.push(u));

    listen<{ room_id: string; peer: PeerInfo }>("peer-joined-room", (event) => {
      addPeer(event.payload.peer);
    }).then((u) => unlisteners.push(u));

    listen<{ room_id: string; peer_id: string }>("peer-left-room", (event) => {
      removePeer(event.payload.peer_id);
    }).then((u) => unlisteners.push(u));

    // Phase 1: Enhanced messaging events
    listen<{ message_id: string; channel_id: string; new_content: string; edited_at: string }>(
      "message-edited",
      (event) => {
        editMessage(event.payload.message_id, event.payload.new_content, event.payload.edited_at);
      }
    ).then((u) => unlisteners.push(u));

    listen<{ message_id: string; channel_id: string }>("message-deleted", (event) => {
      deleteMessage(event.payload.message_id);
    }).then((u) => unlisteners.push(u));

    listen<{ message_id: string; channel_id: string; peer_id: string; emoji: string }>(
      "reaction-added",
      (event) => {
        const { message_id, peer_id, emoji } = event.payload;
        addReaction(message_id, {
          id: `${message_id}-${peer_id}-${emoji}`,
          message_id,
          peer_id,
          emoji,
          created_at: new Date().toISOString(),
        });
      }
    ).then((u) => unlisteners.push(u));

    listen<{ message_id: string; channel_id: string; peer_id: string; emoji: string }>(
      "reaction-removed",
      (event) => {
        removeReaction(event.payload.message_id, event.payload.peer_id, event.payload.emoji);
      }
    ).then((u) => unlisteners.push(u));

    listen<{ channel_id: string; peer_id: string; display_name: string }>(
      "typing-started",
      (event) => {
        setTyping(event.payload.channel_id, event.payload.peer_id, event.payload.display_name, true);
      }
    ).then((u) => unlisteners.push(u));

    listen<{ channel_id: string; peer_id: string }>("typing-stopped", (event) => {
      setTyping(event.payload.channel_id, event.payload.peer_id, "", false);
    }).then((u) => unlisteners.push(u));

    // Phase 5: Friends
    listen<{ from_peer_id: string; from_display_name: string }>(
      "friend-request-received",
      (event) => {
        console.log("Friend request from:", event.payload.from_display_name);
      }
    ).then((u) => unlisteners.push(u));

    // Voice state and media engine events
    listen<{ peer_id: string; display_name: string; channel_id: string | null; room_id: string; muted: boolean; deafened: boolean; video: boolean; screen_sharing: boolean }>(
      "voice-state-changed",
      (event) => { handleVoiceStateChanged(event.payload); }
    ).then((u) => unlisteners.push(u));

    listen<{ peer_id: string }>(
      "voice-connected",
      (event) => { handleVoiceConnected(event.payload); }
    ).then((u) => unlisteners.push(u));

    listen<{ peer_id: string }>(
      "voice-disconnected",
      (event) => { handleVoiceDisconnected(event.payload); }
    ).then((u) => unlisteners.push(u));

    listen<{ peer_id: string; speaking: boolean }>(
      "speaking-changed",
      (event) => { handleSpeakingChanged(event.payload); }
    ).then((u) => unlisteners.push(u));

    // Channel sync events
    listen<{ room_id: string; channel_id: string; name: string; channel_type: string; created_at: string }>(
      "channel-created",
      async (event) => {
        const selectedRoomId = useChatStore.getState().selectedRoomId;
        if (event.payload.room_id === selectedRoomId) {
          const channels = await roomsApi.getChannels(selectedRoomId);
          useChatStore.setState({ channels });
        }
      }
    ).then((u) => unlisteners.push(u));

    listen<{ room_id: string; channel_id: string }>(
      "channel-deleted",
      async (event) => {
        const selectedRoomId = useChatStore.getState().selectedRoomId;
        if (event.payload.room_id === selectedRoomId) {
          const channels = await roomsApi.getChannels(selectedRoomId);
          useChatStore.setState({ channels });
        }
      }
    ).then((u) => unlisteners.push(u));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [addMessage, addPeer, removePeer, editMessage, deleteMessage, addReaction, removeReaction, setTyping, handleVoiceStateChanged, handleVoiceConnected, handleVoiceDisconnected, handleSpeakingChanged]);

  // No rooms state (only for rooms mode)
  if (mode === "rooms" && rooms.length === 0) {
    return (
      <div className="h-full flex">
        <RoomSidebar />
        <div className="flex-1 bg-gray-700 flex items-center justify-center">
          <div className="text-center max-w-md px-4">
            <h2 className="text-2xl font-bold text-white mb-2">
              Welcome to Chatr
            </h2>
            <p className="text-gray-400 mb-6">
              Create a room to start chatting, or join an existing room with an
              invite code.
            </p>
            <div className="flex gap-4 justify-center">
              <p className="text-gray-500 text-sm">
                Use the <span className="text-green-400 font-bold text-lg">+</span> and{" "}
                <span className="text-indigo-400">join</span> buttons in the left sidebar
              </p>
            </div>
          </div>
        </div>
        <SearchModal />
      </div>
    );
  }

  return (
    <div className="h-full flex">
      <RoomSidebar />
      {mode === "rooms" && (
        <>
          <ChannelSidebar />
          <ChatArea />
          <MemberList />
        </>
      )}
      {mode === "dms" && <DmPanel />}
      {mode === "friends" && <FriendsPanel />}
      <SearchModal />
    </div>
  );
}
