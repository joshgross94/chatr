import { useState } from "react";

interface TooltipProps {
  text: string;
  children: React.ReactNode;
  position?: "right" | "top" | "bottom";
}

export default function Tooltip({ text, children, position = "right" }: TooltipProps) {
  const [show, setShow] = useState(false);

  const positionClasses = {
    right: "left-full ml-2 top-1/2 -translate-y-1/2",
    top: "bottom-full mb-2 left-1/2 -translate-x-1/2",
    bottom: "top-full mt-2 left-1/2 -translate-x-1/2",
  };

  return (
    <div
      className="relative"
      onMouseEnter={() => setShow(true)}
      onMouseLeave={() => setShow(false)}
    >
      {children}
      {show && (
        <div
          className={`absolute ${positionClasses[position]} bg-gray-900 text-white text-sm px-2 py-1 rounded shadow-lg whitespace-nowrap z-50 pointer-events-none`}
        >
          {text}
        </div>
      )}
    </div>
  );
}
