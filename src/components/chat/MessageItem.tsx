import { useState, useRef, useEffect } from "react";
import type { Message, Reaction } from "../../lib/types";
import { useIdentityStore } from "../../stores/identityStore";
import { useMessageStore } from "../../stores/messageStore";
import { useChatStore } from "../../stores/chatStore";
import { messages } from "../../lib/api";

interface MessageItemProps {
  message: Message;
  showHeader: boolean;
  replyMessage?: Message | null;
  reactions?: Reaction[];
}

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

// Group reactions by emoji
function groupReactions(reactions: Reaction[]): { emoji: string; count: number; peers: string[] }[] {
  const groups: Record<string, { count: number; peers: string[] }> = {};
  for (const r of reactions) {
    if (!groups[r.emoji]) {
      groups[r.emoji] = { count: 0, peers: [] };
    }
    groups[r.emoji].count++;
    groups[r.emoji].peers.push(r.peer_id);
  }
  return Object.entries(groups).map(([emoji, data]) => ({
    emoji,
    count: data.count,
    peers: data.peers,
  }));
}

const EMOJI_PICKER = ["👍", "❤️", "😂", "😢", "🔥", "💯"];

export default function MessageItem({ message, showHeader, replyMessage, reactions = [] }: MessageItemProps) {
  const identity = useIdentityStore((s) => s.identity);
  const setReplyingTo = useMessageStore((s) => s.setReplyingTo);
  const { selectedChannelId } = useChatStore();
  const isMe = identity?.peer_id === message.sender_peer_id;
  const [showActions, setShowActions] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const editRef = useRef<HTMLTextAreaElement>(null);

  const reactionGroups = groupReactions(reactions);

  useEffect(() => {
    if (isEditing && editRef.current) {
      editRef.current.focus();
      editRef.current.setSelectionRange(editRef.current.value.length, editRef.current.value.length);
    }
  }, [isEditing]);

  const handleEdit = async () => {
    if (!editContent.trim() || editContent === message.content) {
      setIsEditing(false);
      return;
    }
    try {
      await messages.edit(message.id, editContent.trim());
      setIsEditing(false);
    } catch (err) {
      console.error("Failed to edit message:", err);
    }
  };

  const handleDelete = async () => {
    try {
      await messages.delete(message.id);
      setShowDeleteConfirm(false);
    } catch (err) {
      console.error("Failed to delete message:", err);
    }
  };

  const handleReaction = async (emoji: string) => {
    try {
      await messages.addReaction(message.id, emoji);
      setShowEmojiPicker(false);
    } catch (err) {
      console.error("Failed to add reaction:", err);
    }
  };

  const handlePin = async () => {
    if (!selectedChannelId) return;
    try {
      await messages.pin(selectedChannelId, message.id);
    } catch (err) {
      console.error("Failed to pin message:", err);
    }
  };

  const handleReactionClick = async (emoji: string) => {
    const myReaction = reactions.find(
      (r) => r.emoji === emoji && r.peer_id === identity?.peer_id
    );
    try {
      if (myReaction) {
        await messages.removeReaction(message.id, emoji);
      } else {
        await messages.addReaction(message.id, emoji);
      }
    } catch (err) {
      console.error("Failed to toggle reaction:", err);
    }
  };

  // Reply preview
  const replyPreview = replyMessage ? (
    <div className="flex items-center gap-1.5 mb-1 text-xs text-gray-400">
      <svg className="w-3 h-3 text-gray-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
      </svg>
      <span className="font-medium text-indigo-400">{replyMessage.sender_display_name}</span>
      <span className="truncate opacity-75">{replyMessage.content}</span>
    </div>
  ) : null;

  // Reaction bar
  const reactionBar = reactionGroups.length > 0 ? (
    <div className="flex gap-1 mt-1">
      {reactionGroups.map(({ emoji, count, peers }) => {
        const iHaveReacted = identity && peers.includes(identity.peer_id);
        return (
          <button
            key={emoji}
            onClick={() => handleReactionClick(emoji)}
            className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs transition-colors ${
              iHaveReacted
                ? "bg-indigo-600/50 text-white border border-indigo-500"
                : "bg-gray-600/50 text-gray-300 hover:bg-gray-600"
            }`}
          >
            {emoji} {count > 1 && <span className="text-gray-400">{count}</span>}
          </button>
        );
      })}
    </div>
  ) : null;

  // Action buttons on hover
  const actionButtons = showActions && !isEditing ? (
    <div className="absolute right-2 -top-3 flex gap-0.5 bg-gray-700 border border-gray-600 rounded shadow-lg px-0.5 py-0.5 z-10">
      <button
        onClick={() => setReplyingTo(message)}
        className="p-1 text-gray-400 hover:text-white rounded hover:bg-gray-600"
        title="Reply"
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
        </svg>
      </button>
      {isMe && (
        <button
          onClick={() => {
            setIsEditing(true);
            setEditContent(message.content);
          }}
          className="p-1 text-gray-400 hover:text-white rounded hover:bg-gray-600"
          title="Edit"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
        </button>
      )}
      {isMe && (
        <button
          onClick={() => setShowDeleteConfirm(true)}
          className="p-1 text-gray-400 hover:text-red-400 rounded hover:bg-gray-600"
          title="Delete"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      )}
      <div className="relative">
        <button
          onClick={() => setShowEmojiPicker(!showEmojiPicker)}
          className="p-1 text-gray-400 hover:text-white rounded hover:bg-gray-600"
          title="React"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
        {showEmojiPicker && (
          <div className="absolute top-full right-0 mt-1 bg-gray-700 border border-gray-600 rounded shadow-lg p-2 flex gap-1 z-20">
            {EMOJI_PICKER.map((emoji) => (
              <button
                key={emoji}
                onClick={() => handleReaction(emoji)}
                className="text-xl hover:bg-gray-600 rounded p-1 transition-colors"
              >
                {emoji}
              </button>
            ))}
          </div>
        )}
      </div>
      <button
        onClick={handlePin}
        className="p-1 text-gray-400 hover:text-white rounded hover:bg-gray-600"
        title="Pin"
      >
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
        </svg>
      </button>
    </div>
  ) : null;

  // Delete confirmation
  const deleteConfirm = showDeleteConfirm ? (
    <div className="absolute top-full left-0 right-0 mt-1 bg-gray-800 border border-gray-600 rounded shadow-lg p-3 z-20">
      <p className="text-sm text-gray-300 mb-2">Delete this message?</p>
      <div className="flex gap-2">
        <button
          onClick={handleDelete}
          className="px-3 py-1 bg-red-600 hover:bg-red-500 text-white text-sm rounded transition-colors"
        >
          Delete
        </button>
        <button
          onClick={() => setShowDeleteConfirm(false)}
          className="px-3 py-1 bg-gray-600 hover:bg-gray-500 text-white text-sm rounded transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  ) : null;

  const contentElement = isEditing ? (
    <div className="mt-1">
      <textarea
        ref={editRef}
        value={editContent}
        onChange={(e) => setEditContent(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleEdit();
          }
          if (e.key === "Escape") {
            setIsEditing(false);
            setEditContent(message.content);
          }
        }}
        className="w-full bg-gray-600 text-white text-sm rounded px-2 py-1 outline-none resize-none"
        rows={2}
      />
      <div className="flex gap-2 mt-1 text-xs text-gray-400">
        <span>Enter to save • Escape to cancel</span>
      </div>
    </div>
  ) : (
    <>
      <div className="text-gray-200 text-sm break-words">
        {message.content}
        {message.edited_at && (
          <span className="text-xs text-gray-500 ml-1">(edited)</span>
        )}
      </div>
      {reactionBar}
    </>
  );

  if (!showHeader) {
    return (
      <div
        className="group relative flex items-start px-4 py-0.5 hover:bg-gray-650/30"
        onMouseEnter={() => setShowActions(true)}
        onMouseLeave={() => {
          setShowActions(false);
          setShowEmojiPicker(false);
        }}
      >
        {actionButtons}
        {deleteConfirm}
        <div className="w-10 shrink-0 flex items-center justify-center">
          <span className="text-xs text-gray-500 opacity-0 group-hover:opacity-100 transition-opacity">
            {formatTime(message.timestamp)}
          </span>
        </div>
        <div className="ml-4 min-w-0 flex-1">
          {contentElement}
        </div>
      </div>
    );
  }

  return (
    <div
      className="group relative flex items-start px-4 pt-3 pb-0.5 hover:bg-gray-650/30"
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => {
        setShowActions(false);
        setShowEmojiPicker(false);
      }}
    >
      {actionButtons}
      {deleteConfirm}
      {replyPreview && !showHeader && (
        <div className="absolute top-0 left-0 right-0">
          {replyPreview}
        </div>
      )}
      <div
        className={`w-10 h-10 rounded-full ${getAvatarColor(
          message.sender_peer_id
        )} flex items-center justify-center text-white text-sm font-bold shrink-0 ${
          replyMessage ? "mt-5" : ""
        }`}
      >
        {message.sender_display_name.charAt(0).toUpperCase()}
      </div>
      <div className={`ml-4 min-w-0 flex-1 ${replyMessage ? "mt-5" : ""}`}>
        {replyPreview}
        <div className="flex items-baseline gap-2">
          <span
            className={`font-medium text-sm ${
              isMe ? "text-indigo-400" : "text-white"
            }`}
          >
            {message.sender_display_name}
          </span>
          <span className="text-xs text-gray-500">
            {formatTime(message.timestamp)}
          </span>
          {message.edited_at && (
            <span className="text-xs text-gray-500">(edited)</span>
          )}
        </div>
        {contentElement}
      </div>
    </div>
  );
}
