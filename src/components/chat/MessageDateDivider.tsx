interface MessageDateDividerProps {
  date: string;
}

export default function MessageDateDivider({ date }: MessageDateDividerProps) {
  return (
    <div className="flex items-center gap-2 px-4 py-2">
      <div className="flex-1 h-px bg-gray-600" />
      <span className="text-xs text-gray-400 font-medium">{date}</span>
      <div className="flex-1 h-px bg-gray-600" />
    </div>
  );
}
