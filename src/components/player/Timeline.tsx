import React, { useRef, useEffect, useCallback, useState } from "react";
import { formatDuration } from "../../utils/format";

interface TimelineProps {
  duration: number;        // seconds
  currentTime: number;     // seconds
  trimStart: number;       // seconds
  trimEnd: number;         // seconds
  onSeek: (t: number) => void;
  onTrimStartChange: (t: number) => void;
  onTrimEndChange: (t: number) => void;
}

type DragTarget = "playhead" | "trimStart" | "trimEnd" | null;

export const Timeline: React.FC<TimelineProps> = ({
  duration, currentTime, trimStart, trimEnd, onSeek, onTrimStartChange, onTrimEndChange,
}) => {
  const trackRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<DragTarget>(null);
  const [tooltip, setTooltip] = useState<{ x: number; time: number } | null>(null);

  const getTimeFromX = useCallback((clientX: number): number => {
    const el = trackRef.current;
    if (!el || duration <= 0) return 0;
    const { left, width } = el.getBoundingClientRect();
    return Math.max(0, Math.min(duration, ((clientX - left) / width) * duration));
  }, [duration]);

  const pct = (t: number) => `${(t / duration) * 100}%`;

  const handleMouseDown = useCallback((target: DragTarget, e: React.MouseEvent) => {
    e.stopPropagation();
    dragRef.current = target;
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    const t = getTimeFromX(e.clientX);
    switch (dragRef.current) {
      case "playhead": onSeek(t); break;
      case "trimStart": onTrimStartChange(Math.min(t, trimEnd - 0.5)); break;
      case "trimEnd": onTrimEndChange(Math.max(t, trimStart + 0.5)); break;
    }
  }, [getTimeFromX, trimStart, trimEnd, onSeek, onTrimStartChange, onTrimEndChange]);

  const handleMouseUp = useCallback(() => {
    dragRef.current = null;
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }, [handleMouseMove]);

  useEffect(() => () => {
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }, []);

  const handleTrackClick = useCallback((e: React.MouseEvent) => {
    if (dragRef.current) return;
    onSeek(getTimeFromX(e.clientX));
  }, [getTimeFromX, onSeek]);

  const handleMouseMoveTrack = (e: React.MouseEvent) => {
    const t = getTimeFromX(e.clientX);
    const { left } = e.currentTarget.getBoundingClientRect();
    setTooltip({ x: e.clientX - left, time: t });
  };

  // Tick marks
  const ticks: number[] = [];
  const interval = duration > 120 ? 30 : duration > 30 ? 10 : 5;
  for (let i = 0; i <= duration; i += interval) ticks.push(i);

  return (
    <div className="timeline">
      <div
        ref={trackRef}
        className="timeline-track"
        onClick={handleTrackClick}
        onMouseMove={handleMouseMoveTrack}
        onMouseLeave={() => setTooltip(null)}
      >
        {/* Excluded region (before trim start) */}
        <div className="timeline-excluded" style={{ left: 0, width: pct(trimStart) }} />
        {/* Highlighted trim region */}
        <div className="timeline-trimmed" style={{ left: pct(trimStart), width: pct(trimEnd - trimStart) }} />
        {/* Excluded region (after trim end) */}
        <div className="timeline-excluded" style={{ left: pct(trimEnd), right: 0 }} />

        {/* Playhead */}
        <div
          className="timeline-playhead"
          style={{ left: pct(currentTime) }}
          onMouseDown={(e) => handleMouseDown("playhead", e)}
        >
          <div className="playhead-line" />
          <div className="playhead-head" />
        </div>

        {/* Trim start handle */}
        <div
          className="trim-handle trim-handle-start"
          style={{ left: pct(trimStart) }}
          onMouseDown={(e) => handleMouseDown("trimStart", e)}
        />
        {/* Trim end handle */}
        <div
          className="trim-handle trim-handle-end"
          style={{ left: pct(trimEnd) }}
          onMouseDown={(e) => handleMouseDown("trimEnd", e)}
        />

        {/* Tick marks */}
        {ticks.map((t) => (
          <div key={t} className="timeline-tick" style={{ left: pct(t) }}>
            <div className="tick-line" />
            <span className="tick-label">{formatDuration(t * 1000)}</span>
          </div>
        ))}

        {/* Hover tooltip */}
        {tooltip && (
          <div className="timeline-tooltip" style={{ left: tooltip.x }}>
            {formatDuration(tooltip.time * 1000)}
          </div>
        )}
      </div>
    </div>
  );
};
