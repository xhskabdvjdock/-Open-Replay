import React from "react";

type BadgeVariant = "active" | "inactive" | "warning" | "error" | "info";

interface BadgeProps {
  variant?: BadgeVariant;
  children: React.ReactNode;
  dot?: boolean;
}

export const Badge: React.FC<BadgeProps> = ({ variant = "info", children, dot = false }) => (
  <span className={`badge badge-${variant}`}>
    {dot && <span className="badge-dot" />}
    {children}
  </span>
);
