import React, { useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Play, Pause, Volume2, VolumeX, Maximize } from "lucide-react";
import { formatDuration } from "../../utils/format";

interface VideoPlayerProps {
  src: string;
  currentTime: number;
  duration: number;
  playing: boolean;
  volume: number;
  muted: boolean;
  onTimeUpdate: (t: number) => void;
  onPlayPause: (playing: boolean) => void;
  onVolumeChange: (v: number) => void;
  onMuteToggle: () => void;
  onDurationChange: (d: number) => void;
  onEnded: () => void;
}

export const VideoPlayer: React.FC<VideoPlayerProps> = ({
  src, currentTime, duration, playing, volume, muted,
  onTimeUpdate, onPlayPause, onVolumeChange, onMuteToggle, onDurationChange, onEnded,
}) => {
  const { t } = useTranslation();
  const videoRef = useRef<HTMLVideoElement>(null);

  // Sync external play state
  useEffect(() => {
    const v = videoRef.current;
    if (!v) return;
    if (playing) { v.play().catch(() => onPlayPause(false)); }
    else v.pause();
  }, [playing]);

  // Sync external seek
  useEffect(() => {
    const v = videoRef.current;
    if (!v) return;
    if (Math.abs(v.currentTime - currentTime) > 0.5) v.currentTime = currentTime;
  }, [currentTime]);

  useEffect(() => {
    const v = videoRef.current;
    if (v) v.volume = volume;
  }, [volume]);

  useEffect(() => {
    const v = videoRef.current;
    if (v) v.muted = muted;
  }, [muted]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case " ": e.preventDefault(); onPlayPause(!playing); break;
      case "ArrowLeft": { const v = videoRef.current; if (v) onTimeUpdate(Math.max(0, v.currentTime - 5)); break; }
      case "ArrowRight": { const v = videoRef.current; if (v) onTimeUpdate(Math.min(duration, v.currentTime + 5)); break; }
      case "m": case "M": onMuteToggle(); break;
    }
  }, [playing, duration, onPlayPause, onTimeUpdate, onMuteToggle]);

  const handleFullscreen = () => {
    const v = videoRef.current;
    if (v) {
      if (document.fullscreenElement) document.exitFullscreen();
      else v.requestFullscreen();
    }
  };

  return (
    <div className="video-player" tabIndex={0} onKeyDown={handleKeyDown}>
      <video
        ref={videoRef}
        src={src}
        className="video-el"
        onClick={() => onPlayPause(!playing)}
        onTimeUpdate={() => { const v = videoRef.current; if (v) onTimeUpdate(v.currentTime); }}
        onDurationChange={() => { const v = videoRef.current; if (v) onDurationChange(v.duration); }}
        onEnded={onEnded}
        onPlay={() => onPlayPause(true)}
        onPause={() => onPlayPause(false)}
        preload="metadata"
      />

      <div className="video-controls">
        <button className="icon-btn" onClick={() => onPlayPause(!playing)} aria-label={playing ? t("editor.pause") : t("editor.play")}>
          {playing ? <Pause size={20} /> : <Play size={20} />}
        </button>

        <span className="video-time">
          {formatDuration(currentTime * 1000)} / {formatDuration(duration * 1000)}
        </span>

        <div className="video-volume">
          <button className="icon-btn" onClick={onMuteToggle}>
            {muted || volume === 0 ? <VolumeX size={16} /> : <Volume2 size={16} />}
          </button>
          <input
            type="range" min={0} max={1} step={0.05} value={muted ? 0 : volume}
            onChange={(e) => onVolumeChange(Number(e.target.value))}
            className="slider-input volume-slider"
          />
        </div>

        <button className="icon-btn ms-auto" onClick={handleFullscreen} aria-label={t("editor.fullscreen")}>
          <Maximize size={16} />
        </button>
      </div>
    </div>
  );
};
