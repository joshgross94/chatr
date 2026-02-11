import { useState } from "react";
import Modal from "../common/Modal";
import { useChatStore } from "../../stores/chatStore";

interface JoinRoomModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function JoinRoomModal({ isOpen, onClose }: JoinRoomModalProps) {
  const [inviteCode, setInviteCode] = useState("");
  const [isJoining, setIsJoining] = useState(false);
  const [error, setError] = useState("");
  const { joinRoomByInvite } = useChatStore();

  const handleJoin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inviteCode.trim()) return;

    setIsJoining(true);
    setError("");
    try {
      await joinRoomByInvite(inviteCode.trim().toUpperCase());
      setInviteCode("");
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsJoining(false);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Join a Room">
      <form onSubmit={handleJoin} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-1">
            Invite Code
          </label>
          <input
            type="text"
            value={inviteCode}
            onChange={(e) => setInviteCode(e.target.value.toUpperCase())}
            placeholder="ABCD1234"
            className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none font-mono tracking-wider text-center text-lg"
            maxLength={8}
            autoFocus
          />
        </div>
        {error && (
          <p className="text-red-400 text-sm">{error}</p>
        )}
        <div className="flex gap-3 justify-end">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-gray-300 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={inviteCode.trim().length < 8 || isJoining}
            className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-600 text-white rounded-lg transition-colors"
          >
            {isJoining ? "Joining..." : "Join Room"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
