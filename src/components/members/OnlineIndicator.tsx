interface OnlineIndicatorProps {
  isOnline: boolean;
}

export default function OnlineIndicator({ isOnline }: OnlineIndicatorProps) {
  return (
    <span
      className={`w-2.5 h-2.5 rounded-full ${
        isOnline ? "bg-green-500" : "bg-gray-500"
      }`}
    />
  );
}
