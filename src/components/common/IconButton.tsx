interface IconButtonProps {
  onClick: () => void;
  title?: string;
  className?: string;
  children: React.ReactNode;
}

export default function IconButton({
  onClick,
  title,
  className = "",
  children,
}: IconButtonProps) {
  return (
    <button
      onClick={onClick}
      title={title}
      className={`p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-600 transition-colors ${className}`}
    >
      {children}
    </button>
  );
}
