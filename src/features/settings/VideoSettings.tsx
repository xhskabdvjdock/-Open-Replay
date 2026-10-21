import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Select } from "../../components/common/Select";
import { getEncoderCapabilities } from "../../services/api";
import type { Settings, EncoderInfo } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

export const VideoSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const [encoders, setEncoders] = useState<EncoderInfo[]>([]);
  const v = settings.video;

  useEffect(() => {
    getEncoderCapabilities()
      .then(setEncoders)
      .catch((e) => toast.error(String(e)));
  }, []);

  const save = async (patch: Partial<Settings["video"]>) => {
    try {
      await onSave({ video: { ...v, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  const encoderOptions = encoders
    .filter((e) => e.isAvailable)
    .map((e) => ({ value: e.name, label: `${e.name} (${e.encoderType})` }));

  const qualityOptions = [
    { value: "low",    label: t("export.low") },
    { value: "medium", label: t("export.medium") },
    { value: "high",   label: t("export.high") },
    { value: "ultra",  label: "Ultra" },
  ];

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.video")}</h2>

      <div className="settings-group">
        <div className="settings-info-row">
          <span className="input-label">{t("settings.codec")}</span>
          <span className="settings-info-value">H.264 / MP4</span>
        </div>

        <Select
          label={t("settings.encoder")}
          options={encoderOptions.length ? encoderOptions : [{ value: "software", label: "Software (H.264)" }]}
          value={v.encoder}
          onChange={(e) => save({ encoder: e.target.value })}
        />

        <Select
          label={t("settings.quality")}
          options={qualityOptions}
          value={v.quality}
          onChange={(e) => save({ quality: e.target.value })}
        />
      </div>
    </div>
  );
};
