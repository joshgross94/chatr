import { useState, useEffect } from "react";
import Modal from "../common/Modal";
import { useIdentityStore } from "../../stores/identityStore";

interface ProfilePopupProps {
  isOpen: boolean;
  onClose: () => void;
}

const STATUS_TYPES = [
  { value: "online", label: "Online" },
  { value: "away", label: "Away" },
  { value: "busy", label: "Busy" },
  { value: "dnd", label: "Do Not Disturb" },
  { value: "invisible", label: "Invisible" },
];

export default function ProfilePopup({ isOpen, onClose }: ProfilePopupProps) {
  const { identity, setDisplayName, setStatus } = useIdentityStore();
  const [name, setName] = useState("");
  const [statusType, setStatusType] = useState("online");
  const [statusMessage, setStatusMessage] = useState("");
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [selectedMic, setSelectedMic] = useState("");
  const [selectedCamera, setSelectedCamera] = useState("");
  const [selectedSpeaker, setSelectedSpeaker] = useState("");

  useEffect(() => {
    if (isOpen && identity) {
      setName(identity.display_name ?? "");
      setStatusType(identity.status_type ?? "online");
      setStatusMessage(identity.status_message ?? "");
      setCopied(false);

      // Load saved device IDs
      setSelectedMic(localStorage.getItem("chatr-mic-id") ?? "");
      setSelectedCamera(localStorage.getItem("chatr-camera-id") ?? "");
      setSelectedSpeaker(localStorage.getItem("chatr-speaker-id") ?? "");

      // Enumerate available devices
      navigator.mediaDevices
        .enumerateDevices()
        .then(setDevices)
        .catch((err) => console.warn("Could not enumerate devices:", err));
    }
  }, [isOpen, identity]);

  const microphones = devices.filter((d) => d.kind === "audioinput");
  const cameras = devices.filter((d) => d.kind === "videoinput");
  const speakers = devices.filter((d) => d.kind === "audiooutput");

  const handleCopyPeerId = async () => {
    if (!identity?.peer_id) return;
    try {
      await navigator.clipboard.writeText(identity.peer_id);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for environments without clipboard API
      const textarea = document.createElement("textarea");
      textarea.value = identity.peer_id;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      const trimmedName = name.trim();
      if (trimmedName && trimmedName !== identity?.display_name) {
        await setDisplayName(trimmedName);
      }
      await setStatus(statusMessage || undefined, statusType);

      // Persist device selections
      if (selectedMic) localStorage.setItem("chatr-mic-id", selectedMic);
      else localStorage.removeItem("chatr-mic-id");
      if (selectedCamera) localStorage.setItem("chatr-camera-id", selectedCamera);
      else localStorage.removeItem("chatr-camera-id");
      if (selectedSpeaker) localStorage.setItem("chatr-speaker-id", selectedSpeaker);
      else localStorage.removeItem("chatr-speaker-id");

      onClose();
    } catch (err) {
      console.error("Failed to save profile:", err);
    } finally {
      setSaving(false);
    }
  };

  const initial = (identity?.display_name ?? "?").charAt(0).toUpperCase();

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Profile">
      <div className="flex flex-col items-center gap-4">
        {/* Avatar */}
        <div className="w-20 h-20 rounded-full bg-indigo-600 flex items-center justify-center text-white text-3xl font-bold">
          {initial}
        </div>

        {/* Peer ID */}
        <div className="w-full">
          <label className="block text-xs text-gray-400 mb-1">Peer ID</label>
          <div className="flex items-center gap-2">
            <code className="flex-1 text-xs text-gray-300 bg-gray-900 rounded px-2 py-1.5 truncate font-mono">
              {identity?.peer_id ?? ""}
            </code>
            <button
              onClick={handleCopyPeerId}
              className="px-3 py-1.5 text-xs bg-gray-700 hover:bg-gray-600 text-gray-300 rounded transition-colors shrink-0"
            >
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
        </div>

        {/* Display Name */}
        <div className="w-full">
          <label className="block text-xs text-gray-400 mb-1">Display Name</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Your display name"
            className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
          />
        </div>

        {/* Status Type */}
        <div className="w-full">
          <label className="block text-xs text-gray-400 mb-1">Status</label>
          <select
            value={statusType}
            onChange={(e) => setStatusType(e.target.value)}
            className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
          >
            {STATUS_TYPES.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </select>
        </div>

        {/* Status Message */}
        <div className="w-full">
          <label className="block text-xs text-gray-400 mb-1">Status Message</label>
          <input
            type="text"
            value={statusMessage}
            onChange={(e) => setStatusMessage(e.target.value)}
            placeholder="What's on your mind?"
            className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
          />
        </div>

        {/* Device Settings */}
        <div className="w-full border-t border-gray-700 pt-4 mt-2">
          <label className="block text-xs text-gray-400 mb-3 font-medium uppercase tracking-wide">
            Device Settings
          </label>

          {/* Microphone */}
          <div className="mb-3">
            <label className="block text-xs text-gray-400 mb-1">Microphone</label>
            <select
              value={selectedMic}
              onChange={(e) => setSelectedMic(e.target.value)}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
            >
              <option value="">Default</option>
              {microphones.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.label || `Microphone (${d.deviceId.slice(0, 8)}...)`}
                </option>
              ))}
            </select>
          </div>

          {/* Camera */}
          <div className="mb-3">
            <label className="block text-xs text-gray-400 mb-1">Camera</label>
            <select
              value={selectedCamera}
              onChange={(e) => setSelectedCamera(e.target.value)}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
            >
              <option value="">Default</option>
              {cameras.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.label || `Camera (${d.deviceId.slice(0, 8)}...)`}
                </option>
              ))}
            </select>
          </div>

          {/* Speaker */}
          <div>
            <label className="block text-xs text-gray-400 mb-1">Speaker</label>
            <select
              value={selectedSpeaker}
              onChange={(e) => setSelectedSpeaker(e.target.value)}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 outline-none text-sm"
            >
              <option value="">Default</option>
              {speakers.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.label || `Speaker (${d.deviceId.slice(0, 8)}...)`}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Save Button */}
        <button
          onClick={handleSave}
          disabled={saving}
          className="w-full mt-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded font-medium transition-colors text-sm"
        >
          {saving ? "Saving..." : "Save"}
        </button>
      </div>
    </Modal>
  );
}
