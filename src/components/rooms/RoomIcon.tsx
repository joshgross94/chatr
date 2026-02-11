import Tooltip from "../common/Tooltip";

interface RoomIconProps {
  name: string;
  isActive: boolean;
  onClick: () => void;
}

export default function RoomIcon({ name, isActive, onClick }: RoomIconProps) {
  const initials = name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

  return (
    <Tooltip text={name}>
      <button
        onClick={onClick}
        className={`w-12 h-12 rounded-2xl flex items-center justify-center text-sm font-bold transition-all duration-200 ${
          isActive
            ? "bg-indigo-600 text-white rounded-xl"
            : "bg-gray-700 text-gray-300 hover:bg-indigo-500 hover:text-white hover:rounded-xl"
        }`}
      >
        {initials}
      </button>
    </Tooltip>
  );
}
