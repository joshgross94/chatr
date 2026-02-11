import type { PeerInfo } from "../../lib/types";
import OnlineIndicator from "./OnlineIndicator";

interface MemberItemProps {
  peer: PeerInfo;
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

export default function MemberItem({ peer }: MemberItemProps) {
  return (
    <div className="flex items-center gap-3 px-3 py-1.5 rounded-lg hover:bg-gray-700/50 cursor-default">
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
