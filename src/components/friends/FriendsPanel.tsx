import { useState, useEffect } from "react";
import { friends, dms, blocked } from "../../lib/api";
import { useViewStore } from "../../stores/viewStore";
import type { Friend } from "../../lib/types";

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

export default function FriendsPanel() {
  const { setMode } = useViewStore();
  const [friendsList, setFriendsList] = useState<Friend[]>([]);
  const [activeTab, setActiveTab] = useState<"all" | "pending" | "add">("all");
  const [newFriendPeerId, setNewFriendPeerId] = useState("");
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  useEffect(() => {
    loadFriends();
  }, []);

  const loadFriends = async () => {
    try {
      const list = await friends.list();
      setFriendsList(list);
    } catch (err) {
      console.error("Failed to load friends:", err);
    }
  };

  const handleSendRequest = async () => {
    if (!newFriendPeerId.trim()) {
      setError("Please enter a peer ID");
      return;
    }

    try {
      await friends.sendRequest(newFriendPeerId.trim());
      setSuccess("Friend request sent!");
      setNewFriendPeerId("");
      setError("");
      setTimeout(() => setSuccess(""), 3000);
      loadFriends();
    } catch (err: any) {
      console.error("Failed to send friend request:", err);
      setError(err.message || "Failed to send request");
    }
  };

  const handleAccept = async (peerId: string) => {
    try {
      await friends.accept(peerId);
      loadFriends();
    } catch (err) {
      console.error("Failed to accept friend request:", err);
    }
  };

  const handleRemove = async (peerId: string) => {
    try {
      await friends.remove(peerId);
      setFriendsList(friendsList.filter((f) => f.peer_id !== peerId));
    } catch (err) {
      console.error("Failed to remove friend:", err);
    }
  };

  const handleMessage = async (peerId: string) => {
    try {
      await dms.create([peerId]);
      setMode("dms");
    } catch (err) {
      console.error("Failed to create DM:", err);
    }
  };

  const handleBlock = async (peerId: string) => {
    try {
      await blocked.block(peerId);
      await handleRemove(peerId);
    } catch (err) {
      console.error("Failed to block user:", err);
    }
  };

  const allFriends = friendsList.filter((f) => f.status === "accepted");
  const pendingIncoming = friendsList.filter(
    (f) => f.status === "pending_incoming"
  );
  const pendingOutgoing = friendsList.filter(
    (f) => f.status === "pending_outgoing"
  );

  return (
    <div className="flex-1 flex flex-col bg-gray-700">
      {/* Header */}
      <div className="h-12 border-b border-gray-600 flex items-center px-4 shrink-0">
        <h2 className="text-white font-semibold">Friends</h2>
        <div className="ml-8 flex gap-4">
          <button
            onClick={() => setActiveTab("all")}
            className={`text-sm font-medium transition-colors ${
              activeTab === "all"
                ? "text-white"
                : "text-gray-400 hover:text-gray-200"
            }`}
          >
            All Friends
          </button>
          <button
            onClick={() => setActiveTab("pending")}
            className={`text-sm font-medium transition-colors ${
              activeTab === "pending"
                ? "text-white"
                : "text-gray-400 hover:text-gray-200"
            }`}
          >
            Pending
            {(pendingIncoming.length > 0 || pendingOutgoing.length > 0) && (
              <span className="ml-1 px-1.5 py-0.5 bg-red-600 text-white text-xs rounded-full">
                {pendingIncoming.length + pendingOutgoing.length}
              </span>
            )}
          </button>
          <button
            onClick={() => setActiveTab("add")}
            className={`text-sm font-medium transition-colors ${
              activeTab === "add"
                ? "text-white"
                : "text-gray-400 hover:text-gray-200"
            }`}
          >
            Add Friend
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {activeTab === "all" && (
          <div>
            {allFriends.length === 0 ? (
              <div className="text-center py-12">
                <p className="text-gray-400 mb-4">No friends yet</p>
                <button
                  onClick={() => setActiveTab("add")}
                  className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
                >
                  Add a Friend
                </button>
              </div>
            ) : (
              <div className="space-y-2">
                {allFriends.map((friend) => (
                  <div
                    key={friend.peer_id}
                    className="flex items-center justify-between p-3 bg-gray-800 rounded-lg hover:bg-gray-750 transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-10 h-10 rounded-full ${getAvatarColor(
                          friend.peer_id
                        )} flex items-center justify-center text-white text-sm font-bold`}
                      >
                        {friend.display_name.charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <p className="text-white font-medium">
                          {friend.display_name}
                        </p>
                        <p className="text-xs text-gray-500 font-mono">
                          {friend.peer_id.slice(0, 16)}...
                        </p>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleMessage(friend.peer_id)}
                        className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-sm rounded transition-colors"
                        title="Send message"
                      >
                        Message
                      </button>
                      <button
                        onClick={() => handleRemove(friend.peer_id)}
                        className="px-3 py-1.5 bg-gray-600 hover:bg-gray-500 text-white text-sm rounded transition-colors"
                        title="Remove friend"
                      >
                        Remove
                      </button>
                      <button
                        onClick={() => handleBlock(friend.peer_id)}
                        className="px-3 py-1.5 bg-red-600 hover:bg-red-500 text-white text-sm rounded transition-colors"
                        title="Block user"
                      >
                        Block
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === "pending" && (
          <div>
            {pendingIncoming.length === 0 && pendingOutgoing.length === 0 ? (
              <div className="text-center py-12">
                <p className="text-gray-400">No pending friend requests</p>
              </div>
            ) : (
              <div className="space-y-6">
                {pendingIncoming.length > 0 && (
                  <div>
                    <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3">
                      Incoming Requests
                    </h3>
                    <div className="space-y-2">
                      {pendingIncoming.map((friend) => (
                        <div
                          key={friend.peer_id}
                          className="flex items-center justify-between p-3 bg-gray-800 rounded-lg"
                        >
                          <div className="flex items-center gap-3">
                            <div
                              className={`w-10 h-10 rounded-full ${getAvatarColor(
                                friend.peer_id
                              )} flex items-center justify-center text-white text-sm font-bold`}
                            >
                              {friend.display_name.charAt(0).toUpperCase()}
                            </div>
                            <div>
                              <p className="text-white font-medium">
                                {friend.display_name}
                              </p>
                              <p className="text-xs text-gray-500 font-mono">
                                {friend.peer_id.slice(0, 16)}...
                              </p>
                            </div>
                          </div>
                          <div className="flex gap-2">
                            <button
                              onClick={() => handleAccept(friend.peer_id)}
                              className="px-3 py-1.5 bg-green-600 hover:bg-green-500 text-white text-sm rounded transition-colors"
                            >
                              Accept
                            </button>
                            <button
                              onClick={() => handleRemove(friend.peer_id)}
                              className="px-3 py-1.5 bg-red-600 hover:bg-red-500 text-white text-sm rounded transition-colors"
                            >
                              Reject
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {pendingOutgoing.length > 0 && (
                  <div>
                    <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3">
                      Outgoing Requests
                    </h3>
                    <div className="space-y-2">
                      {pendingOutgoing.map((friend) => (
                        <div
                          key={friend.peer_id}
                          className="flex items-center justify-between p-3 bg-gray-800 rounded-lg"
                        >
                          <div className="flex items-center gap-3">
                            <div
                              className={`w-10 h-10 rounded-full ${getAvatarColor(
                                friend.peer_id
                              )} flex items-center justify-center text-white text-sm font-bold`}
                            >
                              {friend.display_name.charAt(0).toUpperCase()}
                            </div>
                            <div>
                              <p className="text-white font-medium">
                                {friend.display_name}
                              </p>
                              <p className="text-xs text-gray-500 font-mono">
                                {friend.peer_id.slice(0, 16)}...
                              </p>
                            </div>
                          </div>
                          <button
                            onClick={() => handleRemove(friend.peer_id)}
                            className="px-3 py-1.5 bg-gray-600 hover:bg-gray-500 text-white text-sm rounded transition-colors"
                          >
                            Cancel
                          </button>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === "add" && (
          <div className="max-w-2xl">
            <h3 className="text-lg font-semibold text-white mb-4">
              Add Friend
            </h3>
            <p className="text-gray-400 text-sm mb-4">
              You can add a friend by entering their peer ID.
            </p>
            <div className="bg-gray-800 rounded-lg p-4">
              <div className="mb-3">
                <input
                  type="text"
                  value={newFriendPeerId}
                  onChange={(e) => {
                    setNewFriendPeerId(e.target.value);
                    setError("");
                  }}
                  placeholder="Enter peer ID"
                  className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none"
                />
              </div>
              {error && <p className="text-red-400 text-sm mb-3">{error}</p>}
              {success && (
                <p className="text-green-400 text-sm mb-3">{success}</p>
              )}
              <button
                onClick={handleSendRequest}
                className="w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded transition-colors"
              >
                Send Friend Request
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
