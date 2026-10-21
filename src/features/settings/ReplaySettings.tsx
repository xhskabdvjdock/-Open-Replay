import React from "react";
import { useTranslation } from "react-i18next";
import { Toggle } from "../../components/common/Toggle";
import { Select } from "../../components/common/Select";
import type { Settings } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

export const ReplaySettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const r = settings.replay;

  const save = async (patch: Partial<Settings["replay"]>) => {
    try {
      await onSave({ replay: { ...r, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  const durationOptions = [
    { value: "15",  label: `15 ${t("common.seconds")}` },
    { value: "30",  label: `30 ${t("common.seconds")}` },
    { value: "60",  label: `60 ${t("common.seconds")}` },
    { value: "120", label: `2 ${t("common.minutes")}` },
    { value: "300", label: `5 ${t("common.minutes")}` },
  ];

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.replay")}</h2>

      <div className="settings-group">
        <Toggle
          label={t("settings.replay")}
          description={t("home.replayStatus")}
          checked={r.enabled}
          onChange={(v) => save({ enabled: v })}
        />

        <Select
          label={t("settings.replayDuration")}
          options={durationOptions}
          value={String(r.bufferDurationSec)}
          onChange={(e) => save({ bufferDurationSec: Number(e.target.value) })}
        />

        <Toggle
          label={t("settings.autoStart")}
          checked={r.autoStart}
          onChange={(v) => save({ autoStart: v })}
        />
      </div>
    </div>
  );
};
