import React, { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { X, FolderOpen, FileVideo } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

import { useExport } from "../../hooks/useExport";
import { useToast } from "../../components/common/Toast";
import { Select } from "../../components/common/Select";
import { Input } from "../../components/common/Input";
import { ProgressBar } from "../../components/common/ProgressBar";
import { Button } from "../../components/common/Button";
import { openClipFolder } from "../../services/api";
import type { Clip } from "../../types";

interface ExportDialogProps {
  clip: Clip;
  trimStartMs: number;
  trimEndMs: number;
  onClose: () => void;
}

export const ExportDialog: React.FC<ExportDialogProps> = ({
  clip, trimStartMs, trimEndMs, onClose,
}) => {
  const { t } = useTranslation();
  const toast = useToast();
  const { progress, start, cancel } = useExport();

  const [outputPath, setOutputPath] = useState("");
  const [fileName, setFileName] = useState(
    clip.name.replace(/[^a-z0-9_\-\.]/gi, "_") + "_export.mp4"
  );
  const [quality, setQuality] = useState("high");
  const [resolution, setResolution] = useState("original");
  const [fps, setFps] = useState(0); // 0 = original

  const isExporting = progress.status === "exporting";
  const isDone = progress.status === "complete";
  const isFailed = progress.status === "error";

  const handleBrowse = useCallback(async () => {
    try {
      const dir = await openDialog({ directory: true, title: t("export.outputPath") });
      if (typeof dir === "string") setOutputPath(dir);
    } catch (e) {
      toast.error(String(e));
    }
  }, [t, toast]);

  const handleStart = useCallback(async () => {
    if (!outputPath) { toast.error(t("export.outputPath") + " is required"); return; }
    if (!fileName) { toast.error(t("export.fileName") + " is required"); return; }
    try {
      await start({
        clipId: clip.id,
        inputPath: clip.filePath,
        outputPath,
        fileName,
        format: "mp4",
        codec: "h264",
        quality,
        resolution,
        fps: fps || clip.fps,
        startMs: trimStartMs,
        endMs: trimEndMs,
      });
    } catch (e) {
      toast.error(String(e));
    }
  }, [outputPath, fileName, quality, resolution, fps, clip, trimStartMs, trimEndMs, start, t, toast]);

  const qualityOptions = [
    { value: "low", label: t("export.low") },
    { value: "medium", label: t("export.medium") },
    { value: "high", label: t("export.high") },
    { value: "original", label: t("export.original") },
  ];

  const resOptions = [
    { value: "original", label: t("export.original") },
    { value: "1080p", label: "1080p" },
    { value: "720p", label: "720p" },
    { value: "480p", label: "480p" },
  ];

  const fpsOptions = [
    { value: "0", label: t("export.original") },
    { value: "60", label: "60 fps" },
    { value: "30", label: "30 fps" },
  ];

  return (
    <div className="modal-backdrop">
      <div className="modal-panel" style={{ maxWidth: 520 }} onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2 className="modal-title">{t("export.title")}</h2>
          {!isExporting && (
            <Button variant="ghost" size="sm" onClick={onClose}><X size={16} /></Button>
          )}
        </div>

        <div className="modal-body export-form">
          {!isDone && !isFailed && (
            <>
              <div className="form-row">
                <Select label={t("export.quality")} options={qualityOptions} value={quality} onChange={(e) => setQuality(e.target.value)} />
                <Select label={t("export.resolution")} options={resOptions} value={resolution} onChange={(e) => setResolution(e.target.value)} />
                <Select label={t("export.fps")} options={fpsOptions} value={String(fps)} onChange={(e) => setFps(Number(e.target.value))} />
              </div>

              <div className="form-field">
                <label className="input-label">{t("export.outputPath")}</label>
                <div className="input-inner">
                  <input className="input-field" value={outputPath} onChange={(e) => setOutputPath(e.target.value)} placeholder="/path/to/folder" readOnly />
                  <Button variant="ghost" size="sm" onClick={handleBrowse} icon={<FolderOpen size={14} />}>{t("common.browse")}</Button>
                </div>
              </div>

              <Input label={t("export.fileName")} value={fileName} onChange={(e) => setFileName(e.target.value)} />
            </>
          )}

          {isExporting && (
            <div className="export-progress">
              <ProgressBar value={progress.percentage} showValue animated />
              <p className="export-status-text">{t("export.progress")}… {Math.round(progress.percentage)}%</p>
            </div>
          )}

          {isDone && (
            <div className="export-success">
              <FileVideo size={40} className="export-success-icon" />
              <p>{t("export.complete")}</p>
              <div className="export-success-actions">
                <Button variant="secondary" size="sm" icon={<FolderOpen size={14} />} onClick={() => openClipFolder(clip.id)}>
                  {t("export.openFolder")}
                </Button>
              </div>
            </div>
          )}

          {isFailed && (
            <div className="export-error">
              <p className="error-text">{t("export.failed")}</p>
              <p className="error-detail">{progress.errorMessage}</p>
            </div>
          )}
        </div>

        <div className="modal-footer">
          {!isExporting && !isDone && (
            <>
              <Button variant="ghost" onClick={onClose}>{t("common.cancel")}</Button>
              <Button variant="primary" onClick={handleStart}>{t("export.start")}</Button>
            </>
          )}
          {isExporting && (
            <Button variant="danger" onClick={cancel}>{t("export.cancel")}</Button>
          )}
          {(isDone || isFailed) && (
            <Button variant="secondary" onClick={onClose}>{t("common.close")}</Button>
          )}
        </div>
      </div>
    </div>
  );
};
