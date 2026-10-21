import React from "react";
import { Volume2, VolumeX, Mic, MicOff } from "lucide-react";

interface AudioIndicatorProps {
  label: string;
  level: number; // 0-1 RMS level (simulated from status)
  muted: boolean;
  onMuteToggle: () => void;
  type: "system" | "mic";
}

const BARS = 8;

export const AudioIndicator: React.FC<AudioIndicatorProps> = ({
  label, level, muted, onMuteToggle, type,
}) => {
  const activeCount = muted ? 0 : Math.round(level * BARS);
  const Icon = type === "mic" ? (muted ? MicOff : Mic) : (muted ? VolumeX : Volume2);

  return (
    <div className="audio-indicator">
      <button className="icon-btn" onClick={onMuteToggle} title={muted ? "Unmute" : "Mute"}>
        <Icon size={14} />
      </button>
      <span className="audio-label">{label}</span>
      <div className="vu-meter">
        {Array.from({ length: BARS }, (_, i) => {
          const isActive = i < activeCount;
          const isHigh = i >= BARS * 0.75;
          const isMid = i >= BARS * 0.5;
          const barClass = isActive
            ? isHigh ? "vu-bar vu-bar-high" : isMid ? "vu-bar vu-bar-mid" : "vu-bar vu-bar-low"
            : "vu-bar vu-bar-off";
          return <div key={i} className={barClass} />;
        })}
      </div>
    </div>
  );
};
