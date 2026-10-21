import React from "react";
import { useTranslation } from "react-i18next";
import { Badge } from "../../components/common/Badge";

interface StatusCardProps {
  isRecording: boolean;
  fps: number;
  resolution: string;
  monitorName: string;
}

export const StatusCard: React.FC<StatusCardProps> = ({
  isRecording, fps, resolution, monitorName,
}) => {
  const { t } = useTranslation();

  return (
    <div className={`status-card${isRecording ? " status-card-active" : ""}`}>
      <div className="status-dot-wrap">
        <span className={`status-dot${isRecording ? " status-dot-active" : ""}`} />
      </div>
      <div className="status-info">
        <div className="status-state">
          <Badge variant={isRecording ? "active" : "inactive"} dot>
            {isRecording ? t("home.replayActive") : t("home.replayInactive")}
          </Badge>
        </div>
        {isRecording && (
          <p className="status-meta">
            {monitorName} · {resolution} · {fps} fps
          </p>
        )}
      </div>
    </div>
  );
};
