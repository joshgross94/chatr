import { useState } from "react";
import { useIdentityStore } from "../../stores/identityStore";

export default function SetupScreen() {
  const [name, setName] = useState("");
  const { setDisplayName, identity } = useIdentityStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      await setDisplayName(name.trim());
    }
  };

  return (
    <div className="h-full bg-gray-900 flex items-center justify-center">
      <div className="bg-gray-800 rounded-xl p-8 max-w-md w-full mx-4 shadow-2xl">
        <div className="text-center mb-6">
          <h1 className="text-3xl font-bold text-white mb-2">
            Welcome to Chatr
          </h1>
          <p className="text-gray-400">
            Decentralized peer-to-peer chat. No servers, no tracking.
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label
              htmlFor="displayName"
              className="block text-sm font-medium text-gray-300 mb-1"
            >
              Choose a display name
            </label>
            <input
              id="displayName"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter your name..."
              className="w-full px-4 py-3 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none placeholder-gray-500"
              maxLength={32}
              autoFocus
            />
          </div>

          <button
            type="submit"
            disabled={!name.trim()}
            className="w-full py-3 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-600 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors"
          >
            Get Started
          </button>
        </form>

        {identity && (
          <p className="mt-4 text-xs text-gray-500 text-center font-mono truncate">
            Peer ID: {identity.peer_id}
          </p>
        )}
      </div>
    </div>
  );
}
