import type { VoiceParticipant as VoiceParticipantType } from "../../lib/types";

interface Props {
  participant: VoiceParticipantType;
}

export default function VoiceParticipant({ participant }: Props) {
  return (
    <div className="px-4 py-1">
      <div className="flex items-center gap-2 text-sm text-gray-400">
        {/* Avatar with speaking indicator */}
        <div
          className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold shrink-0 ${
            participant.speaking
              ? "bg-green-600 ring-2 ring-green-400"
              : "bg-gray-600"
          }`}
        >
          {participant.displayName.charAt(0).toUpperCase()}
        </div>

        <span className="truncate text-gray-300 text-xs">
          {participant.displayName}
        </span>

        {/* Status icons */}
        <div className="flex items-center gap-1 ml-auto shrink-0">
          {participant.muted && (
            <svg className="w-3.5 h-3.5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
            </svg>
          )}
          {participant.deafened && (
            <svg className="w-3.5 h-3.5 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
          )}
          {participant.video && (
            <svg className="w-3.5 h-3.5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          )}
          {participant.screenSharing && (
            <svg className="w-3.5 h-3.5 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          )}
        </div>
      </div>
      {/* Streaming banner */}
      {participant.screenSharing && (
        <div className="ml-8 mt-0.5 flex items-center gap-1">
          <span className="text-[10px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded bg-red-600 text-white">LIVE</span>
          <span className="text-[10px] text-gray-500">Screen sharing</span>
        </div>
      )}
    </div>
  );
}
