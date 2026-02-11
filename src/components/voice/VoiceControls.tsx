import { useVoiceStore } from "../../stores/voiceStore";
import { useChatStore } from "../../stores/chatStore";

export default function VoiceControls() {
  const {
    currentChannelId,
    isMuted,
    isDeafened,
    isCameraEnabled,
    isScreenSharing,
    toggleMute,
    toggleDeafen,
    toggleCamera,
    toggleScreenShare,
    leaveVoiceChannel,
  } = useVoiceStore();

  const channels = useChatStore((s) => s.channels);
  const channelName = channels.find((c) => c.id === currentChannelId)?.name;

  if (!currentChannelId) return null;

  return (
    <div className="border-t border-gray-700 px-3 py-2 bg-gray-800/80">
      {/* Connection info */}
      <div className="flex items-center gap-2 mb-2">
        <div className="w-2 h-2 rounded-full bg-green-500" />
        <div className="min-w-0">
          <p className="text-xs font-semibold text-green-500">Voice Connected</p>
          <p className="text-xs text-gray-400 truncate">{channelName ?? "Voice Channel"}</p>
        </div>
      </div>

      {/* Control buttons */}
      <div className="flex items-center gap-1">
        {/* Mute */}
        <button
          onClick={toggleMute}
          className={`p-1.5 rounded transition-colors ${
            isMuted
              ? "bg-red-500/20 text-red-400 hover:bg-red-500/30"
              : "text-gray-400 hover:text-white hover:bg-gray-700"
          }`}
          title={isMuted ? "Unmute" : "Mute"}
        >
          {isMuted ? (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
            </svg>
          )}
        </button>

        {/* Deafen */}
        <button
          onClick={toggleDeafen}
          className={`p-1.5 rounded transition-colors ${
            isDeafened
              ? "bg-red-500/20 text-red-400 hover:bg-red-500/30"
              : "text-gray-400 hover:text-white hover:bg-gray-700"
          }`}
          title={isDeafened ? "Undeafen" : "Deafen"}
        >
          {isDeafened ? (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
            </svg>
          )}
        </button>

        {/* Camera */}
        <button
          onClick={toggleCamera}
          className={`p-1.5 rounded transition-colors ${
            isCameraEnabled
              ? "bg-green-500/20 text-green-400 hover:bg-green-500/30"
              : "text-gray-400 hover:text-white hover:bg-gray-700"
          }`}
          title={isCameraEnabled ? "Turn Off Camera" : "Turn On Camera"}
        >
          {isCameraEnabled ? (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M3 3l18 18" />
            </svg>
          )}
        </button>

        {/* Screen Share */}
        <button
          onClick={toggleScreenShare}
          className={`p-1.5 rounded transition-colors ${
            isScreenSharing
              ? "bg-green-500/20 text-green-400 hover:bg-green-500/30"
              : "text-gray-400 hover:text-white hover:bg-gray-700"
          }`}
          title={isScreenSharing ? "Stop Screen Share" : "Share Screen"}
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        </button>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Disconnect */}
        <button
          onClick={leaveVoiceChannel}
          className="p-1.5 rounded bg-red-600/20 text-red-400 hover:bg-red-600/40 transition-colors"
          title="Disconnect"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M16 8l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M5 3a2 2 0 00-2 2v1c0 8.284 6.716 15 15 15h1a2 2 0 002-2v-3.28a1 1 0 00-.684-.948l-4.493-1.498a1 1 0 00-1.21.502l-1.13 2.257a11.042 11.042 0 01-5.516-5.517l2.257-1.128a1 1 0 00.502-1.21L9.228 3.683A1 1 0 008.279 3H5z" />
          </svg>
        </button>
      </div>
    </div>
  );
}
