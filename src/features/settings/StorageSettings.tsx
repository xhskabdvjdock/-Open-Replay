import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FolderOpen, Trash2 } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { getStorageInfo } from "../../services/api";
import { formatFileSize } from "../../utils/format";
import type { Settings, StorageInfo } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void; info: (m: string) => void };
}

export const StorageSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const [info, setInfo] = useState<StorageInfo | null>(null);
  const s = settings.storage;

  useEffect(() => {
    getStorageInfo()
      .then(setInfo)
      .catch((e) => toast.error(String(e)));
  }, []);

  const save = async (patch: Partial<Settings["storage"]>) => {
    try {
      await onSave({ storage: { ...s, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  const browseFolder = async (key: "clipsFolder" | "tempFolder") => {
    try {
      const dir = await openDialog({ directory: true });
      if (typeof dir === "string") save({ [key]: dir });
    } catch (e) { toast.error(String(e)); }
  };

  const usedPct = info ? Math.round((info.usedByClipsBytes / info.totalBytes) * 100) : 0;

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.storage")}</h2>

      <div className="settings-group">
        {/* Clips folder */}
        <div className="form-field">
          <label className="input-label">{t("settings.savePath")}</label>
          <div className="input-inner">
            <input className="input-field" value={s.clipsFolder} readOnly />
            <button className="btn btn-ghost btn-sm" onClick={() => browseFolder("clipsFolder")}>
              <FolderOpen size={14} />
            </button>
          </div>
        </div>

        {/* Temp folder */}
        <div className="form-field">
          <label className="input-label">{t("settings.tempFolder")}</label>
          <div className="input-inner">
            <input className="input-field" value={s.tempFolder} readOnly />
            <button className="btn btn-ghost btn-sm" onClick={() => browseFolder("tempFolder")}>
              <FolderOpen size={14} />
            </button>
          </div>
        </div>

        {/* Disk info */}
        {info && (
          <div className="storage-info-card">
            <div className="storage-row">
              <span className="meta-label">{t("settings.diskSpace")}</span>
              <span className="meta-value">
                {formatFileSize(info.usedByClipsBytes)} / {formatFileSize(info.totalBytes)} ({usedPct}%)
              </span>
            </div>
            <div className="progress-track">
              <div className="progress-fill" style={{ width: `${usedPct}%` }} />
            </div>
            <div className="storage-row">
              <span className="meta-label">Free</span>
              <span className="meta-value">{formatFileSize(info.freeBytes)}</span>
            </div>
          </div>
        )}

        <button className="btn btn-ghost btn-sm" onClick={() => toast.info(t("settings.cleanTemp"))}>
          <Trash2 size={14} />
          <span className="btn-label">{t("settings.cleanTemp")}</span>
        </button>
      </div>
    </div>
  );
};
