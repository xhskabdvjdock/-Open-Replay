import React from "react";
import { useTranslation } from "react-i18next";
import { Film, Trash2, FolderOpen, Edit2, Download } from "lucide-react";
import type { Clip } from "../../types";
import { formatDuration, formatFileSize, formatRelativeTime } from "../../utils/format";

interface ClipRowProps {
  clip: Clip;
  onOpen: () => void;
  onRename: () => void;
  onDelete: () => void;
  onOpenFolder: () => void;
  onExport: () => void;
}

export const ClipRow: React.FC<ClipRowProps> = ({
  clip, onOpen, onRename, onDelete, onOpenFolder, onExport,
}) => {
  const { t, i18n } = useTranslation();
  const isAr = i18n.language === "ar";

  return (
    <div className="clip-row" onClick={onOpen}>
      <div className="clip-row-thumb">
        {clip.thumbnailPath ? (
          <img src={`asset://${clip.thumbnailPath}`} alt={clip.name} className="clip-row-img" />
        ) : (
          <div className="clip-row-placeholder"><Film size={16} /></div>
        )}
      </div>

      <div className="clip-row-name">
        <span className="clip-name">{clip.name}</span>
      </div>

      <span className="clip-row-cell">{formatDuration(clip.durationMs)}</span>
      <span className="clip-row-cell">{clip.width}×{clip.height}</span>
      <span className="clip-row-cell">{clip.fps.toFixed(0)} fps</span>
      <span className="clip-row-cell">{formatRelativeTime(clip.createdAt, isAr)}</span>
      <span className="clip-row-cell">{formatFileSize(clip.fileSize, isAr)}</span>

      <div className="clip-row-actions" onClick={(e) => e.stopPropagation()}>
        <button className="icon-btn" onClick={onRename} title={t("library.rename")}><Edit2 size={14} /></button>
        <button className="icon-btn" onClick={onOpenFolder} title={t("library.openFolder")}><FolderOpen size={14} /></button>
        <button className="icon-btn" onClick={onExport} title={t("library.export")}><Download size={14} /></button>
        <button className="icon-btn icon-btn-danger" onClick={onDelete} title={t("library.delete")}><Trash2 size={14} /></button>
      </div>
    </div>
  );
};
