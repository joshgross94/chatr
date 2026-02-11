import { useState, useEffect } from "react";
import { useChatStore } from "../../stores/chatStore";
import { usePeerStore } from "../../stores/peerStore";
import { useViewStore } from "../../stores/viewStore";
import { messages } from "../../lib/api";
import type { PinnedMessage } from "../../lib/types";

export default function RoomHeader() {
  const { rooms, selectedRoomId, selectedChannelId, channels } = useChatStore();
  const { peers } = usePeerStore();
  const { setSearchOpen } = useViewStore();
  const [copied, setCopied] = useState(false);
  const [showPinnedDrawer, setShowPinnedDrawer] = useState(false);
  const [pinnedMessages, setPinnedMessages] = useState<PinnedMessage[]>([]);

  const room = rooms.find((r) => r.id === selectedRoomId);
  const currentChannel = channels.find((c) => c.id === selectedChannelId);

  useEffect(() => {
    if (showPinnedDrawer && selectedChannelId) {
      loadPinnedMessages();
    }
  }, [showPinnedDrawer, selectedChannelId]);

  const loadPinnedMessages = async () => {
    if (!selectedChannelId) return;
    try {
      const pins = await messages.getPinned(selectedChannelId);
      setPinnedMessages(pins);
    } catch (err) {
      console.error("Failed to load pinned messages:", err);
    }
  };

  const handleUnpin = async (messageId: string) => {
    if (!selectedChannelId) return;
    try {
      await messages.unpin(selectedChannelId, messageId);
      setPinnedMessages(pinnedMessages.filter((p) => p.message_id !== messageId));
    } catch (err) {
      console.error("Failed to unpin message:", err);
    }
  };

  if (!room) return null;

  const copyInviteCode = async () => {
    await navigator.clipboard.writeText(room.invite_code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const memberCount = peers.length + 1; // +1 for self

  return (
    <>
      <div className="h-12 border-b border-gray-600 flex items-center px-4 shrink-0">
        <span className="text-gray-400 mr-2">#</span>
        <span className="text-white font-medium">
          {currentChannel?.name ?? "general"}
        </span>
        {currentChannel?.topic && (
          <>
            <span className="text-gray-600 mx-2">|</span>
            <span className="text-gray-400 text-sm truncate">{currentChannel.topic}</span>
          </>
        )}
        <div className="ml-auto flex items-center gap-2">
          <span className="text-gray-400 text-sm mr-2">
            {memberCount} {memberCount === 1 ? "member" : "members"}
          </span>
          <button
            onClick={() => setSearchOpen(true)}
            className="p-2 text-gray-400 hover:text-white rounded hover:bg-gray-600 transition-colors"
            title="Search (Ctrl+K)"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          <button
            onClick={() => setShowPinnedDrawer(true)}
            className="p-2 text-gray-400 hover:text-white rounded hover:bg-gray-600 transition-colors"
            title="Pinned messages"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
            </svg>
          </button>
          <button
            onClick={copyInviteCode}
            className="text-xs px-3 py-1 bg-gray-600 hover:bg-gray-500 text-gray-300 hover:text-white rounded transition-colors"
            title="Copy invite code"
          >
            {copied ? "Copied!" : `Invite: ${room.invite_code}`}
          </button>
        </div>
      </div>

      {/* Pinned messages drawer */}
      {showPinnedDrawer && (
        <div
          className="fixed inset-0 bg-black/50 flex justify-end z-50"
          onClick={() => setShowPinnedDrawer(false)}
        >
          <div
            className="w-96 bg-gray-800 border-l border-gray-700 shadow-xl overflow-y-auto"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-4 border-b border-gray-700 flex items-center justify-between">
              <h3 className="text-white font-semibold">Pinned Messages</h3>
              <button
                onClick={() => setShowPinnedDrawer(false)}
                className="text-gray-400 hover:text-white"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div className="p-4">
              {pinnedMessages.length === 0 ? (
                <p className="text-gray-400 text-sm text-center py-8">
                  No pinned messages in this channel
                </p>
              ) : (
                <div className="space-y-3">
                  {pinnedMessages.map((pin) => (
                    <div
                      key={pin.id}
                      className="bg-gray-700 rounded p-3 border border-gray-600"
                    >
                      <div className="flex items-start justify-between gap-2 mb-2">
                        <span className="text-xs text-gray-400">
                          Pinned by {pin.pinned_by}
                        </span>
                        <button
                          onClick={() => handleUnpin(pin.message_id)}
                          className="text-gray-400 hover:text-red-400"
                          title="Unpin"
                        >
                          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                      <p className="text-sm text-gray-300">Message ID: {pin.message_id}</p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}
