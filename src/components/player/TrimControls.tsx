import React, { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { RotateCcw } from "lucide-react";
import { formatDuration } from "../../utils/format";

interface TrimControlsProps {
  duration: number;   // seconds
  trimStart: number;  // seconds
  trimEnd: number;    // seconds
  onTrimStartChange: (t: number) => void;
  onTrimEndChange: (t: number) => void;
  onReset: () => void;
}

export const TrimControls: React.FC<TrimControlsProps> = ({
  duration, trimStart, trimEnd, onTrimStartChange, onTrimEndChange, onReset,
}) => {
  const { t } = useTranslation();
  const trimmedDuration = trimEnd - trimStart;

  const parseTime = (val: string): number => {
    const parts = val.split(":").map(Number);
    if (parts.length === 2) return parts[0] * 60 + parts[1];
    if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];
    return 0;
  };

  return (
    <div className="trim-controls">
      <div className="trim-field">
        <label className="input-label">{t("editor.trimStart")}</label>
        <input
          type="text"
          className="input-field trim-input"
          value={formatDuration(trimStart * 1000)}
          onChange={(e) => {
            const v = parseTime(e.target.value);
            if (!isNaN(v)) onTrimStartChange(Math.max(0, Math.min(v, trimEnd - 0.5)));
          }}
        />
      </div>

      <div className="trim-duration-display">
        <span className="trim-duration-label">{t("editor.trimmedDuration")}</span>
        <span className="trim-duration-value">{formatDuration(trimmedDuration * 1000)}</span>
        <span className="trim-original-label">{t("editor.originalDuration")}: {formatDuration(duration * 1000)}</span>
      </div>

      <div className="trim-field">
        <label className="input-label">{t("editor.trimEnd")}</label>
        <input
          type="text"
          className="input-field trim-input"
          value={formatDuration(trimEnd * 1000)}
          onChange={(e) => {
            const v = parseTime(e.target.value);
            if (!isNaN(v)) onTrimEndChange(Math.min(duration, Math.max(v, trimStart + 0.5)));
          }}
        />
      </div>

      <button className="btn btn-ghost btn-sm" onClick={onReset} title={t("editor.resetTrim")}>
        <RotateCcw size={14} />
        <span className="btn-label">{t("editor.resetTrim")}</span>
      </button>
    </div>
  );
};
