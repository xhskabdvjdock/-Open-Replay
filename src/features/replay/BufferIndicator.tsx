import React from "react";
import { useTranslation } from "react-i18next";
import { ProgressBar } from "../../components/common/ProgressBar";

interface BufferIndicatorProps {
  fillPercent: number;
  durationMs: number;
  isRecording: boolean;
}

export const BufferIndicator: React.FC<BufferIndicatorProps> = ({
  fillPercent, durationMs, isRecording,
}) => {
  const { t, i18n } = useTranslation();
  const isAr = i18n.language === "ar";
  const durationSec = Math.round(durationMs / 1000);

  const label =
    fillPercent >= 99
      ? t("home.bufferFull")
      : fillPercent > 0
      ? t("home.bufferFilling")
      : t("home.replayInactive");

  return (
    <div className="buffer-indicator">
      <div className="buffer-header">
        <span className="buffer-label">{label}</span>
        <span className="buffer-duration">
          {durationSec.toLocaleString(isAr ? "ar-EG" : "en-US")}
          {t("common.seconds")}
        </span>
      </div>
      <ProgressBar
        value={fillPercent}
        variant={fillPercent >= 99 ? "success" : "default"}
        animated={isRecording && fillPercent < 99}
      />
    </div>
  );
};
