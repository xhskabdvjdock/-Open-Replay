import React from "react";
import { Button } from "./Button";

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  action?: { label: string; onClick: () => void };
}

export const EmptyState: React.FC<EmptyStateProps> = ({ icon, title, description, action }) => (
  <div className="empty-state">
    {icon && <div className="empty-icon">{icon}</div>}
    <h3 className="empty-title">{title}</h3>
    {description && <p className="empty-description">{description}</p>}
    {action && (
      <Button variant="primary" size="sm" onClick={action.onClick} style={{ marginTop: "1rem" }}>
        {action.label}
      </Button>
    )}
  </div>
);
