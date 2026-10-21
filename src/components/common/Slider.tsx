import React from "react";

interface SliderProps {
  label?: string;
  min: number;
  max: number;
  step?: number;
  value: number;
  onChange: (value: number) => void;
  disabled?: boolean;
  formatValue?: (v: number) => string;
}

export const Slider: React.FC<SliderProps> = ({
  label, min, max, step = 1, value, onChange, disabled = false, formatValue,
}) => {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className={`slider-wrapper${disabled ? " slider-disabled" : ""}`}>
      <div className="slider-header">
        {label && <span className="input-label">{label}</span>}
        <span className="slider-value">{formatValue ? formatValue(value) : value}</span>
      </div>
      <input
        type="range"
        className="slider-input"
        min={min}
        max={max}
        step={step}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(Number(e.target.value))}
        style={{ "--slider-pct": `${pct}%` } as React.CSSProperties}
      />
    </div>
  );
};
