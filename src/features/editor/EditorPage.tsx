import React, { useState, useEffect, useCallback } from "react";
import { useParams, useNavigate, useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { ChevronLeft, Download } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";

import { getClip } from "../../services/api";
import { useToast } from "../../components/common/Toast";
import { VideoPlayer } from "../../components/player/VideoPlayer";
import { Timeline } from "../../components/player/Timeline";
import { TrimControls } from "../../components/player/TrimControls";
import { ExportDialog } from "../export/ExportDialog";
import type { Clip } from "../../types";

export const EditorPage: React.FC = () => {
  const { clipId } = useParams<{ clipId: string }>();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { t } = useTranslation();
  const toast = useToast();

  const [clip, setClip] = useState<Clip | null>(null);
  const [loading, setLoading] = useState(true);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [volume, setVolume] = useState(0.8);
  const [muted, setMuted] = useState(false);
  const [trimStart, setTrimStart] = useState(0);
  const [trimEnd, setTrimEnd] = useState(0);
  const [showExport, setShowExport] = useState(searchParams.get("export") === "1");

  useEffect(() => {
    if (!clipId) return;
    setLoading(true);
    getClip(clipId)
      .then((c) => {
        setClip(c);
        setDuration(c.durationMs / 1000);
        setTrimEnd(c.durationMs / 1000);
        setLoading(false);
      })
      .catch((e) => {
        toast.error(String(e));
        setLoading(false);
      });
  }, [clipId]);

  const handleReset = useCallback(() => {
    setTrimStart(0);
    setTrimEnd(duration);
  }, [duration]);

  if (loading) return <div className="page-content"><div className="loading-state">{t("common.loading")}</div></div>;
  if (!clip) return <div className="page-content"><div className="error-state">{t("common.error")}</div></div>;

  const videoSrc = convertFileSrc(clip.filePath);

  return (
    <div className="page-content editor-page">
      {/* Top bar */}
      <div className="editor-topbar">
        <button className="link-btn" onClick={() => navigate(-1)}>
          <ChevronLeft size={16} />
          {t("editor.backToLibrary")}
        </button>
        <h1 className="page-title">{clip.name}</h1>
        <button
          className="btn btn-primary btn-md"
          onClick={() => setShowExport(true)}
        >
          <Download size={16} />
          <span className="btn-label">{t("editor.export")}</span>
        </button>
      </div>

      {/* Video player */}
      <VideoPlayer
        src={videoSrc}
        currentTime={currentTime}
        duration={duration}
        playing={playing}
        volume={volume}
        muted={muted}
        onTimeUpdate={setCurrentTime}
        onPlayPause={setPlaying}
        onVolumeChange={setVolume}
        onMuteToggle={() => setMuted((m) => !m)}
        onDurationChange={setDuration}
        onEnded={() => setPlaying(false)}
      />

      {/* Timeline */}
      <Timeline
        duration={duration}
        currentTime={currentTime}
        trimStart={trimStart}
        trimEnd={trimEnd}
        onSeek={setCurrentTime}
        onTrimStartChange={setTrimStart}
        onTrimEndChange={setTrimEnd}
      />

      {/* Trim controls */}
      <TrimControls
        duration={duration}
        trimStart={trimStart}
        trimEnd={trimEnd}
        onTrimStartChange={setTrimStart}
        onTrimEndChange={setTrimEnd}
        onReset={handleReset}
      />

      {/* Export dialog */}
      {showExport && (
        <ExportDialog
          clip={clip}
          trimStartMs={trimStart * 1000}
          trimEndMs={trimEnd * 1000}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
};
