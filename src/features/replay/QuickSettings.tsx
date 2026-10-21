import React from "react";
import { useTranslation } from "react-i18next";
import type { ReplayStatus } from "../../types";

interface QuickSettingsProps {
  status: ReplayStatus;
  shortcut: string;
}

interface MetaItemProps { label: string; value: string; active?: boolean }
const MetaItem: React.FC<MetaItemProps> = ({ label, value, active }) => (
  <div className="meta-item">
    <span className="meta-label">{label}</span>
    <span className={`meta-value${active ? " meta-value-accent" : ""}`}>{value}</span>
  </div>
);

export const QuickSettings: React.FC<QuickSettingsProps> = ({ status, shortcut }) => {
  const { t } = useTranslation();

  return (
    <div className="quick-settings-grid">
      <MetaItem label={t("home.monitor")} value={status.monitorName || "—"} />
      <MetaItem label={t("home.quality")} value={status.resolution || "—"} />
      <MetaItem label={t("home.fps")} value={status.targetFps ? `${status.targetFps}` : "—"} />
      <MetaItem
        label={t("home.systemAudio")}
        value={status.hasAudio ? t("common.on") : t("common.off")}
        active={status.hasAudio}
      />
      <MetaItem
        label={t("home.microphone")}
        value={status.hasMicrophone ? t("common.on") : t("common.off")}
        active={status.hasMicrophone}
      />
      <MetaItem label={t("home.shortcut")} value={shortcut || "—"} />
    </div>
  );
};
