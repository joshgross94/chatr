import { useState, useEffect, useRef } from "react";
import { useIdentityStore } from "../../stores/identityStore";
import { dms } from "../../lib/api";
import type { DmConversation, DmMessage } from "../../lib/types";

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function getAvatarColor(peerId: string): string {
  const colors = [
    "bg-red-500",
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-purple-500",
    "bg-pink-500",
    "bg-indigo-500",
    "bg-teal-500",
  ];
  let hash = 0;
  for (let i = 0; i < peerId.length; i++) {
    hash = peerId.charCodeAt(i) + ((hash << 5) - hash);
  }
  return colors[Math.abs(hash) % colors.length];
}

export default function DmPanel() {
  const identity = useIdentityStore((s) => s.identity);
  const [conversations, setConversations] = useState<DmConversation[]>([]);
  const [selectedConversation, setSelectedConversation] = useState<string | null>(null);
  const [messages, setMessages] = useState<DmMessage[]>([]);
  const [messageContent, setMessageContent] = useState("");
  const [showNewDmModal, setShowNewDmModal] = useState(false);
  const [newDmPeerId, setNewDmPeerId] = useState("");
  const [error, setError] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadConversations();
  }, []);

  useEffect(() => {
    if (selectedConversation) {
      loadMessages(selectedConversation);
    }
  }, [selectedConversation]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const loadConversations = async () => {
    try {
      const convos = await dms.list();
      setConversations(convos);
      if (convos.length > 0 && !selectedConversation) {
        setSelectedConversation(convos[0].id);
      }
    } catch (err) {
      console.error("Failed to load DM conversations:", err);
    }
  };

  const loadMessages = async (conversationId: string) => {
    try {
      const msgs = await dms.getMessages(conversationId);
      setMessages(msgs);
    } catch (err) {
      console.error("Failed to load DM messages:", err);
    }
  };

  const handleSendMessage = async () => {
    if (!messageContent.trim() || !selectedConversation) return;

    const msg = messageContent.trim();
    setMessageContent("");

    try {
      const newMsg = await dms.sendMessage(selectedConversation, msg);
      setMessages([...messages, newMsg]);
    } catch (err) {
      console.error("Failed to send DM:", err);
      setMessageContent(msg);
    }
  };

  const handleCreateDm = async () => {
    if (!newDmPeerId.trim()) {
      setError("Please enter a peer ID");
      return;
    }

    try {
      const conversation = await dms.create([newDmPeerId.trim()]);
      setConversations([...conversations, conversation]);
      setSelectedConversation(conversation.id);
      setShowNewDmModal(false);
      setNewDmPeerId("");
      setError("");
    } catch (err: any) {
      console.error("Failed to create DM:", err);
      setError(err.message || "Failed to create DM");
    }
  };

  const selectedConvo = conversations.find((c) => c.id === selectedConversation);

  return (
    <div className="flex-1 flex bg-gray-700">
      {/* Conversations list */}
      <div className="w-60 bg-gray-800 flex flex-col shrink-0">
        <div className="h-12 border-b border-gray-700 flex items-center justify-between px-4 shrink-0">
          <h2 className="text-white font-semibold">Direct Messages</h2>
          <button
            onClick={() => setShowNewDmModal(true)}
            className="text-gray-400 hover:text-white"
            title="New DM"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-y-auto py-2">
          {conversations.length === 0 ? (
            <p className="text-gray-500 text-xs px-4 mt-4">
              No conversations yet. Start a new DM.
            </p>
          ) : (
            conversations.map((convo) => (
              <button
                key={convo.id}
                onClick={() => setSelectedConversation(convo.id)}
                className={`w-full text-left px-3 py-2 mx-1 rounded flex items-center gap-3 transition-colors ${
                  convo.id === selectedConversation
                    ? "bg-gray-700/70 text-white"
                    : "text-gray-400 hover:text-gray-200 hover:bg-gray-700/30"
                }`}
                style={{ width: "calc(100% - 8px)" }}
              >
                <div className={`w-8 h-8 rounded-full ${getAvatarColor(convo.id)} flex items-center justify-center text-white text-xs font-bold shrink-0`}>
                  {(convo.name || "DM").charAt(0).toUpperCase()}
                </div>
                <span className="truncate text-sm">{convo.name || "Direct Message"}</span>
              </button>
            ))
          )}
        </div>
      </div>

      {/* Chat area */}
      <div className="flex-1 flex flex-col">
        {selectedConvo ? (
          <>
            {/* Header */}
            <div className="h-12 border-b border-gray-600 flex items-center px-4 shrink-0">
              <span className="text-white font-medium">
                {selectedConvo.name || "Direct Message"}
              </span>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto">
              {messages.length === 0 ? (
                <div className="h-full flex items-center justify-center">
                  <p className="text-gray-500 text-sm">No messages yet. Say hi!</p>
                </div>
              ) : (
                <div className="py-4">
                  {messages.map((msg, idx) => {
                    const isMe = identity?.peer_id === msg.sender_peer_id;
                    const prevMsg = idx > 0 ? messages[idx - 1] : null;
                    const showHeader =
                      !prevMsg || prevMsg.sender_peer_id !== msg.sender_peer_id;

                    if (!showHeader) {
                      return (
                        <div
                          key={msg.id}
                          className="flex items-start px-4 py-0.5 hover:bg-gray-650/30"
                        >
                          <div className="w-10 shrink-0" />
                          <div className="ml-4 min-w-0">
                            <div className="text-gray-200 text-sm break-words">
                              {msg.content}
                            </div>
                          </div>
                        </div>
                      );
                    }

                    return (
                      <div
                        key={msg.id}
                        className="flex items-start px-4 pt-3 pb-0.5 hover:bg-gray-650/30"
                      >
                        <div
                          className={`w-10 h-10 rounded-full ${getAvatarColor(
                            msg.sender_peer_id
                          )} flex items-center justify-center text-white text-sm font-bold shrink-0`}
                        >
                          {msg.sender_display_name.charAt(0).toUpperCase()}
                        </div>
                        <div className="ml-4 min-w-0">
                          <div className="flex items-baseline gap-2">
                            <span
                              className={`font-medium text-sm ${
                                isMe ? "text-indigo-400" : "text-white"
                              }`}
                            >
                              {msg.sender_display_name}
                            </span>
                            <span className="text-xs text-gray-500">
                              {formatTime(msg.timestamp)}
                            </span>
                          </div>
                          <div className="text-gray-200 text-sm break-words">
                            {msg.content}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                  <div ref={messagesEndRef} />
                </div>
              )}
            </div>

            {/* Input */}
            <div className="px-4 pb-4 pt-1 shrink-0">
              <div className="bg-gray-600 flex items-end rounded-lg">
                <textarea
                  value={messageContent}
                  onChange={(e) => setMessageContent(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                      e.preventDefault();
                      handleSendMessage();
                    }
                  }}
                  placeholder={`Message ${selectedConvo.name || "DM"}`}
                  className="flex-1 bg-transparent text-white placeholder-gray-400 px-4 py-3 outline-none resize-none text-sm max-h-48"
                  rows={1}
                />
                <button
                  onClick={handleSendMessage}
                  disabled={!messageContent.trim()}
                  className="px-4 py-3 text-gray-400 hover:text-white disabled:text-gray-600 transition-colors"
                >
                  <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
                  </svg>
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="h-full flex items-center justify-center">
            <div className="text-center">
              <p className="text-gray-400 mb-4">Select a conversation or start a new one</p>
              <button
                onClick={() => setShowNewDmModal(true)}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
              >
                New Direct Message
              </button>
            </div>
          </div>
        )}
      </div>

      {/* New DM Modal */}
      {showNewDmModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-96 border border-gray-700">
            <h3 className="text-xl font-semibold text-white mb-4">New Direct Message</h3>
            <div className="mb-4">
              <label className="block text-sm text-gray-300 mb-2">Peer ID</label>
              <input
                type="text"
                value={newDmPeerId}
                onChange={(e) => {
                  setNewDmPeerId(e.target.value);
                  setError("");
                }}
                placeholder="Enter peer ID"
                className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none"
              />
              {error && <p className="text-red-400 text-xs mt-1">{error}</p>}
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => {
                  setShowNewDmModal(false);
                  setNewDmPeerId("");
                  setError("");
                }}
                className="px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateDm}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
              >
                Start DM
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
