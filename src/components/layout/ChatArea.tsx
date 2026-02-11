import RoomHeader from "../rooms/RoomHeader";
import MessageList from "../chat/MessageList";
import MessageInput from "../chat/MessageInput";
import VideoGrid from "../voice/VideoGrid";

export default function ChatArea() {
  return (
    <div className="flex-1 flex flex-col bg-gray-700 min-w-0 relative">
      <RoomHeader />
      <MessageList />
      <MessageInput />
      <VideoGrid />
    </div>
  );
}
