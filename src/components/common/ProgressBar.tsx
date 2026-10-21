import React from "react";

interface ProgressBarProps {
  value: number; // 0-100
  label?: string;
  showValue?: boolean;
  variant?: "default" | "success" | "warning" | "error";
  animated?: boolean;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value, label, showValue = false, variant = "default", animated = false,
}) => {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <div className="progress-wrapper">
      {(label || showValue) && (
        <div className="progress-header">
          {label && <span className="progress-label">{label}</span>}
          {showValue && <span className="progress-value">{Math.round(clamped)}%</span>}
        </div>
      )}
      <div className="progress-track">
        <div
          className={`progress-fill progress-${variant}${animated ? " progress-animated" : ""}`}
          style={{ width: `${clamped}%` }}
        />
      </div>
    </div>
  );
};
