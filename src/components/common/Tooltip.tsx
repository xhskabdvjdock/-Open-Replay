import React, { useState, useRef } from "react";

interface TooltipProps {
  content: string;
  children: React.ReactElement;
  placement?: "top" | "bottom" | "left" | "right";
}

export const Tooltip: React.FC<TooltipProps> = ({ content, children, placement = "top" }) => {
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  return (
    <div
      ref={ref}
      className="tooltip-wrapper"
      onMouseEnter={() => setVisible(true)}
      onMouseLeave={() => setVisible(false)}
    >
      {children}
      {visible && (
        <div className={`tooltip tooltip-${placement}`} role="tooltip">
          {content}
        </div>
      )}
    </div>
  );
};
