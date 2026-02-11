import { useState, useEffect } from "react";
import { useChatStore } from "../../stores/chatStore";
import { useIdentityStore } from "../../stores/identityStore";
import { useVoiceStore } from "../../stores/voiceStore";
import { channels as channelsApi, rooms } from "../../lib/api";
import VoiceControls from "../voice/VoiceControls";
import VoiceParticipantComponent from "../voice/VoiceParticipant";
import ProfilePopup from "../profile/ProfilePopup";
import type { Channel } from "../../lib/types";

export default function ChannelSidebar() {
  const { rooms: roomsList, channels, selectedRoomId, selectedChannelId, selectChannel } =
    useChatStore();
  const identity = useIdentityStore((s) => s.identity);
  const { currentChannelId: voiceChannelId, channelParticipants, joinVoiceChannel, leaveVoiceChannel, speakingPeers, isMuted, isDeafened } =
    useVoiceStore();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newChannelName, setNewChannelName] = useState("");
  const [newChannelType, setNewChannelType] = useState<"text" | "voice">("text");
  const [createError, setCreateError] = useState("");
  const [contextMenuChannel, setContextMenuChannel] = useState<Channel | null>(null);
  const [contextMenuPos, setContextMenuPos] = useState({ x: 0, y: 0 });
  const [showProfile, setShowProfile] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [editName, setEditName] = useState("");
  const [editTopic, setEditTopic] = useState("");

  const room = roomsList.find((r) => r.id === selectedRoomId);

  const textChannels = channels.filter((c) => (c.channel_type ?? "text") === "text");
  const voiceChannels = channels.filter((c) => c.channel_type === "voice");

  const handleCreateChannel = async () => {
    if (!newChannelName.trim() || !selectedRoomId) {
      setCreateError("Channel name is required");
      return;
    }

    try {
      await channelsApi.create(selectedRoomId, newChannelName.trim(), newChannelType);
      const updatedChannels = await rooms.getChannels(selectedRoomId);
      useChatStore.setState({ channels: updatedChannels });
      setShowCreateModal(false);
      setNewChannelName("");
      setNewChannelType("text");
      setCreateError("");
    } catch (err: any) {
      console.error("Failed to create channel:", err);
      setCreateError(err.message || "Failed to create channel");
    }
  };

  const handleVoiceChannelClick = async (channel: Channel) => {
    if (!selectedRoomId) return;

    if (voiceChannelId === channel.id) {
      // Already in this channel, leave
      await leaveVoiceChannel();
    } else {
      // Join this voice channel
      try {
        await joinVoiceChannel(selectedRoomId, channel.id);
      } catch (err) {
        console.error("Failed to join voice channel:", err);
      }
    }
  };

  const handleContextMenu = (e: React.MouseEvent, channel: Channel) => {
    e.preventDefault();
    setContextMenuChannel(channel);
    setContextMenuPos({ x: e.clientX, y: e.clientY });
  };

  const handleEditChannel = () => {
    if (!contextMenuChannel) return;
    setEditName(contextMenuChannel.name);
    setEditTopic(contextMenuChannel.topic || "");
    setShowEditModal(true);
    setContextMenuChannel(null);
  };

  const handleSaveEdit = async () => {
    if (!contextMenuChannel || !editName.trim()) return;

    try {
      await channelsApi.update(
        contextMenuChannel.id,
        editName.trim(),
        editTopic.trim() || undefined
      );
      if (selectedRoomId) {
        const updatedChannels = await rooms.getChannels(selectedRoomId);
        useChatStore.setState({ channels: updatedChannels });
      }
      setShowEditModal(false);
      setEditName("");
      setEditTopic("");
    } catch (err) {
      console.error("Failed to update channel:", err);
    }
  };

  const handleDeleteChannel = async () => {
    if (!contextMenuChannel) return;

    if (!confirm(`Delete channel #${contextMenuChannel.name}?`)) {
      setContextMenuChannel(null);
      return;
    }

    try {
      await channelsApi.delete(contextMenuChannel.id);
      if (selectedRoomId) {
        const updatedChannels = await rooms.getChannels(selectedRoomId);
        useChatStore.setState({ channels: updatedChannels });
        if (contextMenuChannel.id === selectedChannelId && updatedChannels.length > 0) {
          selectChannel(updatedChannels[0].id);
        }
      }
      setContextMenuChannel(null);
    } catch (err) {
      console.error("Failed to delete channel:", err);
    }
  };

  useEffect(() => {
    const handleClick = () => setContextMenuChannel(null);
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, []);

  if (!room) {
    return (
      <div className="w-60 bg-gray-800 flex flex-col shrink-0">
        <div className="h-12 border-b border-gray-700 flex items-center px-4">
          <span className="text-gray-400 text-sm">No room selected</span>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="w-60 bg-gray-800 flex flex-col shrink-0">
        {/* Room name header */}
        <div className="h-12 border-b border-gray-700 flex items-center px-4 shrink-0">
          <h2 className="text-white font-semibold truncate">{room.name}</h2>
        </div>

        {/* Channel list */}
        <div className="flex-1 overflow-y-auto py-2">
          {/* Text Channels */}
          <div className="px-2 mb-1 flex items-center justify-between">
            <span className="text-xs font-semibold text-gray-400 uppercase tracking-wide px-2">
              Text Channels
            </span>
            <button
              onClick={() => { setNewChannelType("text"); setShowCreateModal(true); }}
              className="text-gray-400 hover:text-white"
              title="Create Channel"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>
          {textChannels.map((channel) => (
            <button
              key={channel.id}
              onClick={() => selectChannel(channel.id)}
              onContextMenu={(e) => handleContextMenu(e, channel)}
              className={`w-full text-left px-2 py-1 mx-1 rounded flex items-center gap-1.5 text-sm transition-colors ${
                channel.id === selectedChannelId
                  ? "bg-gray-700/70 text-white"
                  : "text-gray-400 hover:text-gray-200 hover:bg-gray-700/30"
              }`}
              style={{ width: "calc(100% - 8px)" }}
            >
              <span className="text-gray-500">#</span>
              <span className="truncate">{channel.name}</span>
            </button>
          ))}

          {/* Voice Channels */}
          <div className="px-2 mb-1 mt-4 flex items-center justify-between">
            <span className="text-xs font-semibold text-gray-400 uppercase tracking-wide px-2">
              Voice Channels
            </span>
            <button
              onClick={() => { setNewChannelType("voice"); setShowCreateModal(true); }}
              className="text-gray-400 hover:text-white"
              title="Create Voice Channel"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>
          {voiceChannels.map((channel) => {
            const isConnected = voiceChannelId === channel.id;
            const remoteParticipants = Object.values(channelParticipants[channel.id] ?? {});
            const hasParticipants = isConnected || remoteParticipants.length > 0;

            return (
              <div key={channel.id}>
                <button
                  onClick={() => handleVoiceChannelClick(channel)}
                  onContextMenu={(e) => handleContextMenu(e, channel)}
                  className={`w-full text-left px-2 py-1 mx-1 rounded flex items-center gap-1.5 text-sm transition-colors ${
                    isConnected
                      ? "bg-green-900/30 text-green-400"
                      : "text-gray-400 hover:text-gray-200 hover:bg-gray-700/30"
                  }`}
                  style={{ width: "calc(100% - 8px)" }}
                >
                  {/* Speaker icon */}
                  <svg className={`w-4 h-4 shrink-0 ${isConnected ? "text-green-400" : "text-gray-500"}`} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                  </svg>
                  <span className="truncate">{channel.name}</span>
                </button>

                {/* Show participants in this voice channel (visible to everyone) */}
                {hasParticipants && (
                  <div className="ml-2">
                    {/* Show self when connected */}
                    {isConnected && (
                      <div className="px-4 py-1">
                        <div className="flex items-center gap-2 text-sm text-gray-400">
                          <div className={`w-6 h-6 rounded-full bg-indigo-600 flex items-center justify-center text-xs font-bold shrink-0 ${
                            speakingPeers.has("local") ? "ring-2 ring-green-400" : ""
                          }`}>
                            {identity?.display_name?.charAt(0)?.toUpperCase() ?? "?"}
                          </div>
                          <span className="truncate text-green-300 text-xs">
                            {identity?.display_name ?? "You"}
                          </span>
                          <div className="flex items-center gap-1 ml-auto shrink-0">
                            {isMuted && (
                              <svg className="w-3.5 h-3.5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                                <path strokeLinecap="round" strokeLinejoin="round" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
                              </svg>
                            )}
                            {isDeafened && (
                              <svg className="w-3.5 h-3.5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                                <path strokeLinecap="round" strokeLinejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636" />
                              </svg>
                            )}
                          </div>
                        </div>
                      </div>
                    )}
                    {remoteParticipants.map((p) => (
                      <VoiceParticipantComponent key={p.peerId} participant={p} />
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Voice controls (shown when connected) */}
        <VoiceControls />

        {/* User panel at bottom */}
        <div
          className="h-14 bg-gray-850 border-t border-gray-700 flex items-center px-3 shrink-0 cursor-pointer hover:bg-gray-700/50 transition-colors"
          style={{ backgroundColor: "rgba(17, 24, 39, 0.5)" }}
          onClick={() => setShowProfile(true)}
        >
          <div className="w-8 h-8 rounded-full bg-indigo-600 flex items-center justify-center text-white text-sm font-bold">
            {identity?.display_name?.charAt(0)?.toUpperCase() ?? "?"}
          </div>
          <div className="ml-2 min-w-0">
            <p className="text-sm text-white font-medium truncate">
              {identity?.display_name ?? "Anonymous"}
            </p>
            <p className="text-xs text-gray-500 truncate font-mono">
              {identity?.peer_id?.slice(0, 12) ?? ""}...
            </p>
          </div>
        </div>
      </div>

      {/* Create Channel Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-96 border border-gray-700">
            <h3 className="text-xl font-semibold text-white mb-4">Create Channel</h3>

            {/* Channel type selector */}
            <div className="mb-4">
              <label className="block text-sm text-gray-300 mb-2">Channel Type</label>
              <div className="flex gap-2">
                <button
                  onClick={() => setNewChannelType("text")}
                  className={`flex-1 py-2 px-3 rounded text-sm font-medium transition-colors ${
                    newChannelType === "text"
                      ? "bg-indigo-600 text-white"
                      : "bg-gray-700 text-gray-400 hover:text-white"
                  }`}
                >
                  <span className="mr-1">#</span> Text
                </button>
                <button
                  onClick={() => setNewChannelType("voice")}
                  className={`flex-1 py-2 px-3 rounded text-sm font-medium transition-colors ${
                    newChannelType === "voice"
                      ? "bg-indigo-600 text-white"
                      : "bg-gray-700 text-gray-400 hover:text-white"
                  }`}
                >
                  <svg className="w-4 h-4 inline mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                  </svg>
                  Voice
                </button>
              </div>
            </div>

            <div className="mb-4">
              <label className="block text-sm text-gray-300 mb-2">Channel Name</label>
              <input
                type="text"
                value={newChannelName}
                onChange={(e) => {
                  setNewChannelName(e.target.value);
                  setCreateError("");
                }}
                placeholder={newChannelType === "text" ? "general" : "General Voice"}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none"
                autoFocus
              />
              {createError && <p className="text-red-400 text-xs mt-1">{createError}</p>}
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setNewChannelName("");
                  setNewChannelType("text");
                  setCreateError("");
                }}
                className="px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateChannel}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Channel Modal */}
      {showEditModal && contextMenuChannel && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-96 border border-gray-700">
            <h3 className="text-xl font-semibold text-white mb-4">Edit Channel</h3>
            <div className="mb-4">
              <label className="block text-sm text-gray-300 mb-2">Channel Name</label>
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none"
              />
            </div>
            <div className="mb-4">
              <label className="block text-sm text-gray-300 mb-2">Topic (optional)</label>
              <input
                type="text"
                value={editTopic}
                onChange={(e) => setEditTopic(e.target.value)}
                placeholder="Channel topic"
                className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none"
              />
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => {
                  setShowEditModal(false);
                  setEditName("");
                  setEditTopic("");
                }}
                className="px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveEdit}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Profile Popup */}
      <ProfilePopup isOpen={showProfile} onClose={() => setShowProfile(false)} />

      {/* Context Menu */}
      {contextMenuChannel && (
        <div
          className="fixed bg-gray-800 border border-gray-700 rounded shadow-lg py-1 z-50"
          style={{ left: contextMenuPos.x, top: contextMenuPos.y }}
        >
          <button
            onClick={handleEditChannel}
            className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
          >
            Edit Channel
          </button>
          <button
            onClick={handleDeleteChannel}
            className="w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-red-600 hover:text-white transition-colors"
          >
            Delete Channel
          </button>
        </div>
      )}
    </>
  );
}
