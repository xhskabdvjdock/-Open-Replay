import React, { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Save, Play, Square, ChevronLeft, Film } from "lucide-react";

import { useReplayStatus } from "../../hooks/useReplayStatus";
import { useSettings } from "../../hooks/useSettings";
import { useClips } from "../../hooks/useClips";
import { useToast } from "../../components/common/Toast";
import { Button } from "../../components/common/Button";
import { StatusCard } from "./StatusCard";
import { BufferIndicator } from "./BufferIndicator";
import { QuickSettings } from "./QuickSettings";

import * as api from "../../services/api";
import { formatDuration, formatRelativeTime, formatFileSize } from "../../utils/format";

export const HomePage: React.FC = () => {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();
  const { status } = useReplayStatus(1000);
  const { settings } = useSettings();
  const { clips, refresh } = useClips("date", "desc");
  const [toggling, setToggling] = useState(false);
  const [saving, setSaving] = useState(false);

  const recentClips = clips.slice(0, 5);

  const handleToggleReplay = useCallback(async () => {
    setToggling(true);
    try {
      if (status.isRecording) {
        await api.stopReplay();
        toast.info(t("home.replayInactive"));
      } else {
        await api.startReplay();
        toast.success(t("home.replayActive"));
      }
    } catch (e) {
      toast.error(String(e));
    } finally {
      setToggling(false);
    }
  }, [status.isRecording, t, toast]);

  const handleSaveClip = useCallback(async () => {
    if (!status.isRecording) return;
    setSaving(true);
    try {
      await api.saveReplay();
      toast.success(t("library.empty.title") === "No clips yet" ? "Clip saved!" : "تم حفظ المقطع!");
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSaving(false);
    }
  }, [status.isRecording, t, toast, refresh]);

  const isAr = i18n.language === "ar";

  return (
    <div className="page-content">
      <div className="page-header">
        <h1 className="page-title">{t("home.title")}</h1>
      </div>

      {/* Status + Controls */}
      <div className="home-top">
        <StatusCard
          isRecording={status.isRecording}
          fps={status.actualFps}
          resolution={status.resolution}
          monitorName={status.monitorName}
        />

        <div className="home-controls">
          <Button
            variant={status.isRecording ? "danger" : "primary"}
            size="lg"
            loading={toggling}
            icon={status.isRecording ? <Square size={18} /> : <Play size={18} />}
            onClick={handleToggleReplay}
          >
            {status.isRecording ? t("home.stopReplay") : t("home.startReplay")}
          </Button>

          <Button
            variant="primary"
            size="lg"
            loading={saving}
            disabled={!status.isRecording}
            icon={<Save size={18} />}
            onClick={handleSaveClip}
            title={settings?.shortcuts.saveReplay ?? "Ctrl+Shift+F10"}
          >
            {t("home.saveClip")}
          </Button>
        </div>
      </div>

      {/* Buffer indicator */}
      <section className="home-section">
        <BufferIndicator
          fillPercent={status.bufferFillPercent}
          durationMs={status.bufferDurationMs}
          isRecording={status.isRecording}
        />
      </section>

      {/* Quick settings grid */}
      <section className="home-section">
        <QuickSettings
          status={status}
          shortcut={settings?.shortcuts.saveReplay ?? "Ctrl+Shift+F10"}
        />
      </section>

      {/* Recent clips */}
      <section className="home-section">
        <div className="section-header">
          <h2 className="section-title">{t("home.recentClips")}</h2>
          {clips.length > 0 && (
            <button className="link-btn" onClick={() => navigate("/library")}>
              {t("home.viewAll")}
              <ChevronLeft size={14} style={{ transform: isAr ? "none" : "scaleX(-1)" }} />
            </button>
          )}
        </div>

        {recentClips.length === 0 ? (
          <div className="recent-empty">
            <Film size={24} className="recent-empty-icon" />
            <span>{t("home.noRecentClips")}</span>
          </div>
        ) : (
          <div className="recent-list">
            {recentClips.map((clip) => (
              <button
                key={clip.id}
                className="recent-item"
                onClick={() => navigate(`/editor/${clip.id}`)}
              >
                <div className="recent-thumb">
                  <Film size={16} />
                </div>
                <div className="recent-info">
                  <span className="recent-name">{clip.name}</span>
                  <span className="recent-meta">
                    {formatDuration(clip.durationMs)} ·{" "}
                    {formatRelativeTime(clip.createdAt, isAr)} ·{" "}
                    {formatFileSize(clip.fileSize, isAr)}
                  </span>
                </div>
              </button>
            ))}
          </div>
        )}
      </section>
    </div>
  );
};
