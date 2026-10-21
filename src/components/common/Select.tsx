import React from "react";

interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: SelectOption[];
  placeholder?: string;
  error?: string;
}

export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, options, placeholder, error, className = "", id, ...rest }, ref) => {
    const selectId = id ?? `select-${Math.random().toString(36).slice(2)}`;
    return (
      <div className={`select-wrapper${error ? " select-error" : ""}${className ? " " + className : ""}`}>
        {label && <label htmlFor={selectId} className="input-label">{label}</label>}
        <select ref={ref} id={selectId} className="select-field" {...rest}>
          {placeholder && <option value="" disabled>{placeholder}</option>}
          {options.map((o) => (
            <option key={o.value} value={o.value} disabled={o.disabled}>{o.label}</option>
          ))}
        </select>
        {error && <p className="input-error-msg">{error}</p>}
      </div>
    );
  }
);
Select.displayName = "Select";
