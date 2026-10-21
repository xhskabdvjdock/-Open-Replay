import React from "react";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  icon?: React.ReactNode;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, icon, className = "", id, ...rest }, ref) => {
    const inputId = id ?? `input-${Math.random().toString(36).slice(2)}`;
    return (
      <div className={`input-wrapper${error ? " input-error" : ""}${className ? " " + className : ""}`}>
        {label && <label htmlFor={inputId} className="input-label">{label}</label>}
        <div className="input-inner">
          {icon && <span className="input-icon">{icon}</span>}
          <input ref={ref} id={inputId} className="input-field" {...rest} />
        </div>
        {error && <p className="input-error-msg">{error}</p>}
      </div>
    );
  }
);
Input.displayName = "Input";
