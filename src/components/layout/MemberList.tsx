import { useState, useEffect } from "react";
import { usePeerStore } from "../../stores/peerStore";
import { useIdentityStore } from "../../stores/identityStore";
import { useChatStore } from "../../stores/chatStore";
import { useViewStore } from "../../stores/viewStore";
import { rooms, friends, blocked, dms } from "../../lib/api";
import type { PeerInfo } from "../../lib/types";
import OnlineIndicator from "../members/OnlineIndicator";

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

interface MemberItemProps {
  peer: PeerInfo;
  onContextMenu: (e: React.MouseEvent, peer: PeerInfo) => void;
}

function MemberItem({ peer, onContextMenu }: MemberItemProps) {
  return (
    <div
      className="flex items-center gap-3 px-3 py-1.5 rounded-lg hover:bg-gray-700/50 cursor-default"
      onContextMenu={(e) => onContextMenu(e, peer)}
    >
      <div className="relative">
        <div
          className={`w-8 h-8 rounded-full ${getAvatarColor(
            peer.peer_id
          )} flex items-center justify-center text-white text-xs font-bold ${
            !peer.is_online ? "opacity-50" : ""
          }`}
        >
          {peer.display_name.charAt(0).toUpperCase()}
        </div>
        <div className="absolute -bottom-0.5 -right-0.5">
          <OnlineIndicator isOnline={peer.is_online} />
        </div>
      </div>
      <span
        className={`text-sm truncate ${
          peer.is_online ? "text-gray-300" : "text-gray-500"
        }`}
      >
        {peer.display_name}
      </span>
    </div>
  );
}

export default function MemberList() {
  const { peers } = usePeerStore();
  const identity = useIdentityStore((s) => s.identity);
  const { selectedRoomId } = useChatStore();
  const { setMode } = useViewStore();
  const [contextMenuPeer, setContextMenuPeer] = useState<PeerInfo | null>(null);
  const [contextMenuPos, setContextMenuPos] = useState({ x: 0, y: 0 });
  const [showRoleSubmenu, setShowRoleSubmenu] = useState(false);

  const onlinePeers = peers.filter((p) => p.is_online);
  const offlinePeers = peers.filter((p) => !p.is_online);

  const handleContextMenu = (e: React.MouseEvent, peer: PeerInfo) => {
    e.preventDefault();
    if (peer.peer_id === identity?.peer_id) return; // Don't show context menu for self
    setContextMenuPeer(peer);
    setContextMenuPos({ x: e.clientX, y: e.clientY });
    setShowRoleSubmenu(false);
  };

  const handleSetRole = async (role: string) => {
    if (!contextMenuPeer || !selectedRoomId) return;
    try {
      await rooms.setRole(selectedRoomId, contextMenuPeer.peer_id, role);
      setContextMenuPeer(null);
    } catch (err) {
      console.error("Failed to set role:", err);
    }
  };

  const handleModerate = async (actionType: string) => {
    if (!contextMenuPeer || !selectedRoomId) return;
    const reason = prompt(`Reason for ${actionType}?`);
    if (reason === null) return;

    try {
      await rooms.moderate(selectedRoomId, actionType, contextMenuPeer.peer_id, reason || undefined);
      setContextMenuPeer(null);
    } catch (err) {
      console.error(`Failed to ${actionType}:`, err);
    }
  };

  const handleSendFriendRequest = async () => {
    if (!contextMenuPeer) return;
    try {
      await friends.sendRequest(contextMenuPeer.peer_id);
      setContextMenuPeer(null);
    } catch (err) {
      console.error("Failed to send friend request:", err);
    }
  };

  const handleBlock = async () => {
    if (!contextMenuPeer) return;
    try {
      await blocked.block(contextMenuPeer.peer_id);
      setContextMenuPeer(null);
    } catch (err) {
      console.error("Failed to block user:", err);
    }
  };

  const handleSendDm = async () => {
    if (!contextMenuPeer) return;
    try {
      await dms.create([contextMenuPeer.peer_id]);
      setMode("dms");
      setContextMenuPeer(null);
    } catch (err) {
      console.error("Failed to create DM:", err);
    }
  };

  useEffect(() => {
    const handleClick = () => {
      setContextMenuPeer(null);
      setShowRoleSubmenu(false);
    };
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, []);

  return (
    <>
      <div className="w-60 bg-gray-800 flex flex-col shrink-0 overflow-y-auto">
        <div className="py-4 px-2">
          {/* Self (always shown) */}
          {identity && (
            <>
              <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide px-3 mb-2">
                You
              </h3>
              <MemberItem
                peer={{
                  peer_id: identity.peer_id,
                  display_name: identity.display_name,
                  is_online: true,
                }}
                onContextMenu={() => {}}
              />
            </>
          )}

          {/* Online members */}
          {onlinePeers.length > 0 && (
            <>
              <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide px-3 mb-2 mt-4">
                Online &mdash; {onlinePeers.length}
              </h3>
              {onlinePeers.map((peer) => (
                <MemberItem key={peer.peer_id} peer={peer} onContextMenu={handleContextMenu} />
              ))}
            </>
          )}

          {/* Offline members */}
          {offlinePeers.length > 0 && (
            <>
              <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wide px-3 mb-2 mt-4">
                Offline &mdash; {offlinePeers.length}
              </h3>
              {offlinePeers.map((peer) => (
                <MemberItem key={peer.peer_id} peer={peer} onContextMenu={handleContextMenu} />
              ))}
            </>
          )}

          {onlinePeers.length === 0 && offlinePeers.length === 0 && (
            <p className="text-gray-500 text-xs px-3 mt-4">
              No other peers connected yet
            </p>
          )}
        </div>
      </div>

      {/* Context Menu */}
      {contextMenuPeer && (
        <div
          className="fixed bg-gray-800 border border-gray-700 rounded shadow-lg py-1 z-50 min-w-[160px]"
          style={{ left: contextMenuPos.x, top: contextMenuPos.y }}
        >
          <button
            onClick={handleSendDm}
            className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
          >
            Send DM
          </button>
          <button
            onClick={handleSendFriendRequest}
            className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
          >
            Send Friend Request
          </button>
          <div className="h-px bg-gray-700 my-1" />
          <div className="relative">
            <button
              onMouseEnter={() => setShowRoleSubmenu(true)}
              className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors flex items-center justify-between"
            >
              Set Role
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
              </svg>
            </button>
            {showRoleSubmenu && (
              <div
                className="absolute left-full top-0 ml-1 bg-gray-800 border border-gray-700 rounded shadow-lg py-1 min-w-[120px]"
                onMouseLeave={() => setShowRoleSubmenu(false)}
              >
                <button
                  onClick={() => handleSetRole("admin")}
                  className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
                >
                  Admin
                </button>
                <button
                  onClick={() => handleSetRole("moderator")}
                  className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
                >
                  Moderator
                </button>
                <button
                  onClick={() => handleSetRole("member")}
                  className="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-indigo-600 hover:text-white transition-colors"
                >
                  Member
                </button>
              </div>
            )}
          </div>
          <button
            onClick={() => handleModerate("kick")}
            className="w-full text-left px-4 py-2 text-sm text-orange-400 hover:bg-orange-600 hover:text-white transition-colors"
          >
            Kick
          </button>
          <button
            onClick={() => handleModerate("ban")}
            className="w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-red-600 hover:text-white transition-colors"
          >
            Ban
          </button>
          <button
            onClick={() => handleModerate("mute")}
            className="w-full text-left px-4 py-2 text-sm text-yellow-400 hover:bg-yellow-600 hover:text-white transition-colors"
          >
            Mute
          </button>
          <div className="h-px bg-gray-700 my-1" />
          <button
            onClick={handleBlock}
            className="w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-red-600 hover:text-white transition-colors"
          >
            Block User
          </button>
        </div>
      )}
    </>
  );
}
