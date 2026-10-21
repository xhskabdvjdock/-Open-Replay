import React from "react";

interface ToggleProps {
  label?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  description?: string;
}

export const Toggle: React.FC<ToggleProps> = ({ label, checked, onChange, disabled = false, description }) => {
  return (
    <label className={`toggle-wrapper${disabled ? " toggle-disabled" : ""}`}>
      <div className="toggle-content">
        {label && <span className="toggle-label">{label}</span>}
        {description && <span className="toggle-description">{description}</span>}
      </div>
      <button
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        className={`toggle-track${checked ? " toggle-checked" : ""}`}
        onClick={() => !disabled && onChange(!checked)}
        type="button"
      >
        <span className="toggle-thumb" />
      </button>
    </label>
  );
};
