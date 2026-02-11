import { invoke } from "@tauri-apps/api/core";
import { useVoiceStore } from "../../stores/voiceStore";
import { useEffect, useRef, useState, useCallback } from "react";

/** Get the API port from Tauri and construct the base URL for frame endpoints. */
function useFrameBaseUrl() {
  const [baseUrl, setBaseUrl] = useState<string | null>(null);
  useEffect(() => {
    invoke<number>("get_api_port").then((port) => {
      setBaseUrl(`http://127.0.0.1:${port}`);
    });
  }, []);
  return baseUrl;
}

export default function VideoGrid() {
  const participants = useVoiceStore((s) => s.participants);
  const isCameraEnabled = useVoiceStore((s) => s.isCameraEnabled);
  const isScreenSharing = useVoiceStore((s) => s.isScreenSharing);
  const currentChannelId = useVoiceStore((s) => s.currentChannelId);
  const baseUrl = useFrameBaseUrl();

  if (!currentChannelId || !baseUrl) return null;

  // Collect all video/screen streams to display
  const videoStreams: { peerId: string; type: "video" | "screen"; label: string }[] = [];

  for (const p of Object.values(participants)) {
    if (p.video) {
      videoStreams.push({ peerId: p.peerId, type: "video", label: `${p.displayName} (Camera)` });
    }
    if (p.screenSharing) {
      videoStreams.push({ peerId: p.peerId, type: "screen", label: `${p.displayName} (Screen)` });
    }
  }

  // If nothing to show, return null
  if (videoStreams.length === 0 && !isCameraEnabled && !isScreenSharing) return null;

  return (
    <div className="bg-gray-900 p-2 grid gap-2 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 auto-rows-fr">
      {/* Local camera preview */}
      {isCameraEnabled && (
        <FrameTile
          baseUrl={baseUrl}
          type="video"
          label="You (Camera)"
          isLocal
        />
      )}

      {/* Local screen share preview */}
      {isScreenSharing && (
        <FrameTile
          baseUrl={baseUrl}
          type="screen"
          label="You (Screen)"
          isLocal
        />
      )}

      {/* Remote streams */}
      {videoStreams.map((s) => (
        <FrameTile
          key={`${s.type}-${s.peerId}`}
          baseUrl={baseUrl}
          peerId={s.peerId}
          type={s.type}
          label={s.label}
        />
      ))}
    </div>
  );
}

/** Polls single JPEG frames from the frame server and renders them. */
function FrameTile({
  baseUrl,
  peerId,
  type,
  label,
  isLocal,
}: {
  baseUrl: string;
  peerId?: string;
  type: "video" | "screen";
  label: string;
  isLocal?: boolean;
}) {
  const [localPeerId, setLocalPeerId] = useState<string | null>(null);
  const imgRef = useRef<HTMLImageElement>(null);
  const blobUrlRef = useRef<string | null>(null);

  useEffect(() => {
    if (isLocal && !peerId) {
      invoke<string>("get_my_peer_id").then(setLocalPeerId);
    }
  }, [isLocal, peerId]);

  const effectivePeerId = peerId || localPeerId;

  // Poll frames at ~15fps
  const fetchFrame = useCallback(async () => {
    if (!effectivePeerId || !imgRef.current) return;
    const endpoint = type === "video" ? "media/video" : "media/screen";
    const url = `${baseUrl}/${endpoint}/${effectivePeerId}/frame`;
    try {
      const resp = await fetch(url);
      if (!resp.ok) return;
      const blob = await resp.blob();
      const newUrl = URL.createObjectURL(blob);
      // Revoke old blob URL to prevent memory leak
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
      }
      blobUrlRef.current = newUrl;
      if (imgRef.current) {
        imgRef.current.src = newUrl;
      }
    } catch {
      // Network error, skip this frame
    }
  }, [effectivePeerId, baseUrl, type]);

  useEffect(() => {
    if (!effectivePeerId) return;
    const interval = setInterval(fetchFrame, 66); // ~15fps
    return () => {
      clearInterval(interval);
      if (blobUrlRef.current) {
        URL.revokeObjectURL(blobUrlRef.current);
        blobUrlRef.current = null;
      }
    };
  }, [effectivePeerId, fetchFrame]);

  if (!effectivePeerId) return null;

  return (
    <div className="relative bg-gray-800 rounded-lg overflow-hidden aspect-video">
      <img
        ref={imgRef}
        alt={label}
        className="w-full h-full object-contain"
      />
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/60 to-transparent px-2 py-1">
        <span className="text-xs text-white/90">{label}</span>
      </div>
    </div>
  );
}
