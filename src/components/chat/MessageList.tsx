import { useEffect, useRef, useCallback } from "react";
import { useMessageStore } from "../../stores/messageStore";
import { useChatStore } from "../../stores/chatStore";
import MessageItem from "./MessageItem";
import MessageDateDivider from "./MessageDateDivider";
import type { Message } from "../../lib/types";

function formatDate(timestamp: string): string {
  const date = new Date(timestamp);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) return "Today";
  if (date.toDateString() === yesterday.toDateString()) return "Yesterday";
  return date.toLocaleDateString(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}

function shouldShowHeader(messages: Message[], index: number): boolean {
  if (index === 0) return true;
  const prev = messages[index - 1];
  const curr = messages[index];
  if (prev.sender_peer_id !== curr.sender_peer_id) return true;
  // Show header if messages are more than 5 minutes apart
  const gap = new Date(curr.timestamp).getTime() - new Date(prev.timestamp).getTime();
  if (gap > 5 * 60 * 1000) return true;
  // Show header if this is a reply
  if (curr.reply_to_id) return true;
  return false;
}

function getDateKey(timestamp: string): string {
  return new Date(timestamp).toDateString();
}

export default function MessageList() {
  const { messages, isLoading, loadMessages, loadMoreMessages, reactions, typingPeers } =
    useMessageStore();
  const { selectedChannelId } = useChatStore();
  const bottomRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const shouldScrollRef = useRef(true);

  useEffect(() => {
    if (selectedChannelId) {
      loadMessages(selectedChannelId);
      shouldScrollRef.current = true;
    }
  }, [selectedChannelId, loadMessages]);

  useEffect(() => {
    if (shouldScrollRef.current) {
      bottomRef.current?.scrollIntoView({ behavior: "instant" });
    }
  }, [messages]);

  const handleScroll = useCallback(() => {
    const container = containerRef.current;
    if (!container) return;

    // Auto-scroll if near bottom
    const isNearBottom =
      container.scrollHeight - container.scrollTop - container.clientHeight < 100;
    shouldScrollRef.current = isNearBottom;

    // Load more when scrolled to top
    if (container.scrollTop === 0 && messages.length > 0) {
      const prevHeight = container.scrollHeight;
      loadMoreMessages().then((loaded) => {
        if (loaded) {
          // Maintain scroll position
          requestAnimationFrame(() => {
            container.scrollTop = container.scrollHeight - prevHeight;
          });
        }
      });
    }
  }, [messages.length, loadMoreMessages]);

  // Typing indicator
  const typing = selectedChannelId ? typingPeers[selectedChannelId] ?? [] : [];
  const typingText =
    typing.length === 0
      ? null
      : typing.length === 1
        ? `${typing[0].display_name} is typing...`
        : typing.length === 2
          ? `${typing[0].display_name} and ${typing[1].display_name} are typing...`
          : "Several people are typing...";

  if (!selectedChannelId) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        Select a channel to start chatting
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        Loading messages...
      </div>
    );
  }

  if (messages.length === 0) {
    return (
      <div className="flex-1 flex flex-col">
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <p className="text-gray-400 text-lg mb-1">No messages yet</p>
            <p className="text-gray-500 text-sm">
              Be the first to say something!
            </p>
          </div>
        </div>
        {typingText && (
          <div className="px-4 py-1 text-xs text-gray-400 italic">{typingText}</div>
        )}
      </div>
    );
  }

  // Build message lookup for reply references
  const messageMap = new Map(messages.map((m) => [m.id, m]));

  let lastDateKey = "";

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto"
        onScroll={handleScroll}
      >
        <div className="pt-4 pb-2">
          {messages.map((msg, i) => {
            const dateKey = getDateKey(msg.timestamp);
            const showDate = dateKey !== lastDateKey;
            lastDateKey = dateKey;

            const replyMessage = msg.reply_to_id ? messageMap.get(msg.reply_to_id) ?? null : null;
            const msgReactions = reactions[msg.id] ?? [];

            return (
              <div key={msg.id}>
                {showDate && (
                  <MessageDateDivider date={formatDate(msg.timestamp)} />
                )}
                <MessageItem
                  message={msg}
                  showHeader={shouldShowHeader(messages, i)}
                  replyMessage={replyMessage}
                  reactions={msgReactions}
                />
              </div>
            );
          })}
          <div ref={bottomRef} />
        </div>
      </div>
      {typingText && (
        <div className="px-4 py-1 text-xs text-gray-400 italic shrink-0">{typingText}</div>
      )}
    </div>
  );
}
