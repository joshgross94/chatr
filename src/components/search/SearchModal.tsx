import { useState, useEffect, useRef } from "react";
import { useViewStore } from "../../stores/viewStore";
import { useChatStore } from "../../stores/chatStore";
import { messages } from "../../lib/api";
import type { Message } from "../../lib/types";

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleDateString([], {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function SearchModal() {
  const { searchOpen, setSearchOpen } = useViewStore();
  const { selectChannel, channels } = useChatStore();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Message[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [error, setError] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (searchOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [searchOpen]);

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      setError("");
      return;
    }

    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }

    searchTimeoutRef.current = setTimeout(async () => {
      setIsSearching(true);
      setError("");

      try {
        const response = await messages.search(query.trim(), undefined, 20);
        setResults(response.messages);
      } catch (err: any) {
        console.error("Search failed:", err);
        setError(err.message || "Search failed");
        setResults([]);
      } finally {
        setIsSearching(false);
      }
    }, 300);

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [query]);

  const handleResultClick = (message: Message) => {
    selectChannel(message.channel_id);
    setSearchOpen(false);
    setQuery("");
    setResults([]);
  };

  const handleClose = () => {
    setSearchOpen(false);
    setQuery("");
    setResults([]);
    setError("");
  };

  if (!searchOpen) return null;

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-start justify-center pt-20 z-50"
      onClick={handleClose}
    >
      <div
        className="bg-gray-800 rounded-lg w-full max-w-2xl border border-gray-700 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="p-4 border-b border-gray-700">
          <div className="flex items-center gap-2">
            <svg className="w-5 h-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search messages..."
              className="flex-1 bg-transparent text-white placeholder-gray-400 outline-none text-sm"
            />
            <kbd className="px-2 py-1 bg-gray-700 text-gray-400 text-xs rounded">ESC</kbd>
          </div>
        </div>

        {/* Results */}
        <div className="max-h-96 overflow-y-auto">
          {isSearching && (
            <div className="p-8 text-center">
              <div className="w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto" />
              <p className="text-gray-400 text-sm mt-2">Searching...</p>
            </div>
          )}

          {!isSearching && error && (
            <div className="p-8 text-center">
              <p className="text-red-400 text-sm">{error}</p>
            </div>
          )}

          {!isSearching && !error && query && results.length === 0 && (
            <div className="p-8 text-center">
              <p className="text-gray-400 text-sm">No messages found for "{query}"</p>
            </div>
          )}

          {!isSearching && results.length > 0 && (
            <div className="divide-y divide-gray-700">
              {results.map((message) => {
                const channel = channels.find((c) => c.id === message.channel_id);
                return (
                  <button
                    key={message.id}
                    onClick={() => handleResultClick(message)}
                    className="w-full text-left p-4 hover:bg-gray-700/50 transition-colors"
                  >
                    <div className="flex items-start justify-between gap-4 mb-1">
                      <div className="flex items-center gap-2">
                        <span className="text-white font-medium text-sm">
                          {message.sender_display_name}
                        </span>
                        <span className="text-gray-500 text-xs">in</span>
                        <span className="text-gray-400 text-xs">
                          #{channel?.name || "unknown"}
                        </span>
                      </div>
                      <span className="text-gray-500 text-xs shrink-0">
                        {formatTime(message.timestamp)}
                      </span>
                    </div>
                    <p className="text-gray-300 text-sm line-clamp-2">
                      {message.content}
                    </p>
                  </button>
                );
              })}
            </div>
          )}

          {!query && (
            <div className="p-8 text-center">
              <svg className="w-12 h-12 text-gray-600 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <p className="text-gray-400 text-sm">Search for messages across all channels</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
