import { useState } from "react";
import { useChatStore } from "../../stores/chatStore";
import { useViewStore } from "../../stores/viewStore";
import RoomIcon from "../rooms/RoomIcon";
import CreateRoomModal from "../rooms/CreateRoomModal";
import JoinRoomModal from "../rooms/JoinRoomModal";
import Tooltip from "../common/Tooltip";

export default function RoomSidebar() {
  const { rooms, selectedRoomId, selectRoom } = useChatStore();
  const { mode, setMode } = useViewStore();
  const [showCreate, setShowCreate] = useState(false);
  const [showJoin, setShowJoin] = useState(false);

  return (
    <>
      <div className="w-[72px] bg-gray-950 flex flex-col items-center py-3 gap-2 shrink-0 overflow-y-auto">
        {/* App icon / home */}
        <div className="w-12 h-12 rounded-2xl bg-indigo-600 flex items-center justify-center text-white font-bold text-lg mb-1">
          C
        </div>
        <div className="w-8 h-0.5 bg-gray-700 rounded-full mb-1" />

        {/* DM button */}
        <Tooltip text="Direct Messages">
          <button
            onClick={() => setMode("dms")}
            className={`w-12 h-12 rounded-2xl flex items-center justify-center text-2xl font-light transition-all duration-200 ${
              mode === "dms"
                ? "bg-indigo-600 text-white rounded-xl"
                : "bg-gray-700 text-indigo-400 hover:bg-indigo-500 hover:text-white hover:rounded-xl"
            }`}
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
          </button>
        </Tooltip>

        {/* Friends button */}
        <Tooltip text="Friends">
          <button
            onClick={() => setMode("friends")}
            className={`w-12 h-12 rounded-2xl flex items-center justify-center text-2xl font-light transition-all duration-200 ${
              mode === "friends"
                ? "bg-indigo-600 text-white rounded-xl"
                : "bg-gray-700 text-green-400 hover:bg-green-500 hover:text-white hover:rounded-xl"
            }`}
          >
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
          </button>
        </Tooltip>

        <div className="w-8 h-0.5 bg-gray-700 rounded-full my-1" />

        {/* Room icons */}
        {rooms.map((room) => (
          <RoomIcon
            key={room.id}
            name={room.name}
            isActive={room.id === selectedRoomId && mode === "rooms"}
            onClick={() => {
              selectRoom(room.id);
              setMode("rooms");
            }}
          />
        ))}

        {/* Add room button */}
        <Tooltip text="Create Room">
          <button
            onClick={() => setShowCreate(true)}
            className="w-12 h-12 rounded-2xl bg-gray-700 text-green-500 hover:bg-green-500 hover:text-white hover:rounded-xl flex items-center justify-center text-2xl font-light transition-all duration-200"
          >
            +
          </button>
        </Tooltip>

        <Tooltip text="Join Room">
          <button
            onClick={() => setShowJoin(true)}
            className="w-12 h-12 rounded-2xl bg-gray-700 text-indigo-400 hover:bg-indigo-500 hover:text-white hover:rounded-xl flex items-center justify-center transition-all duration-200"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
            </svg>
          </button>
        </Tooltip>
      </div>

      <CreateRoomModal isOpen={showCreate} onClose={() => setShowCreate(false)} />
      <JoinRoomModal isOpen={showJoin} onClose={() => setShowJoin(false)} />
    </>
  );
}
