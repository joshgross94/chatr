import { useState, useRef, useEffect, useCallback } from "react";
import { useMessageStore } from "../../stores/messageStore";
import { useChatStore } from "../../stores/chatStore";
import { messages } from "../../lib/api";

export default function MessageInput() {
  const [content, setContent] = useState("");
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);
  const { sendMessage, replyingTo, setReplyingTo } = useMessageStore();
  const { selectedChannelId, channels } = useChatStore();

  const channelName =
    channels.find((c) => c.id === selectedChannelId)?.name ?? "general";

  const sendTypingIndicator = useCallback(
    async (typing: boolean) => {
      if (!selectedChannelId) return;
      try {
        await messages.sendTyping(selectedChannelId, typing);
        isTypingRef.current = typing;
      } catch (err) {
        console.error("Failed to send typing indicator:", err);
      }
    },
    [selectedChannelId]
  );

  const handleContentChange = (newContent: string) => {
    setContent(newContent);

    if (!newContent.trim()) {
      // Clear typing if input is empty
      if (isTypingRef.current) {
        sendTypingIndicator(false);
      }
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
      return;
    }

    // Start typing if not already typing
    if (!isTypingRef.current) {
      sendTypingIndicator(true);
    }

    // Reset the timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    typingTimeoutRef.current = setTimeout(() => {
      sendTypingIndicator(false);
    }, 3000);
  };

  useEffect(() => {
    return () => {
      // Cleanup on unmount
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      if (isTypingRef.current) {
        sendTypingIndicator(false);
      }
    };
  }, [sendTypingIndicator]);

  const handleSubmit = async () => {
    if (!content.trim() || !selectedChannelId) return;

    const msg = content.trim();
    setContent("");

    // Clear typing indicator
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }
    if (isTypingRef.current) {
      sendTypingIndicator(false);
    }

    try {
      await sendMessage(selectedChannelId, msg);
    } catch (err) {
      console.error("Failed to send message:", err);
      setContent(msg);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
    if (e.key === "Escape" && replyingTo) {
      setReplyingTo(null);
    }
  };

  if (!selectedChannelId) return null;

  return (
    <div className="px-4 pb-4 pt-1 shrink-0">
      {/* Reply preview bar */}
      {replyingTo && (
        <div className="flex items-center gap-2 px-4 py-2 bg-gray-600/50 rounded-t-lg border-b border-gray-500/50">
          <svg className="w-4 h-4 text-gray-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
          </svg>
          <span className="text-xs text-gray-400">Replying to</span>
          <span className="text-xs text-indigo-400 font-medium">{replyingTo.sender_display_name}</span>
          <span className="text-xs text-gray-500 truncate flex-1">{replyingTo.content}</span>
          <button
            onClick={() => setReplyingTo(null)}
            className="text-gray-400 hover:text-white"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}
      <div className={`bg-gray-600 flex items-end ${replyingTo ? "rounded-b-lg" : "rounded-lg"}`}>
        <textarea
          ref={inputRef}
          value={content}
          onChange={(e) => handleContentChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={`Message #${channelName}`}
          className="flex-1 bg-transparent text-white placeholder-gray-400 px-4 py-3 outline-none resize-none text-sm max-h-48"
          rows={1}
        />
        <button
          onClick={handleSubmit}
          disabled={!content.trim()}
          className="px-4 py-3 text-gray-400 hover:text-white disabled:text-gray-600 transition-colors"
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
          </svg>
        </button>
      </div>
    </div>
  );
}
