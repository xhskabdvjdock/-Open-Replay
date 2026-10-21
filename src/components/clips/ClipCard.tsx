import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Film, MoreHorizontal, Trash2, FolderOpen, Edit2, Download } from "lucide-react";
import type { Clip } from "../../types";
import { formatDuration, formatFileSize, formatRelativeTime } from "../../utils/format";

interface ClipCardProps {
  clip: Clip;
  onOpen: () => void;
  onRename: () => void;
  onDelete: () => void;
  onOpenFolder: () => void;
  onExport: () => void;
}

export const ClipCard: React.FC<ClipCardProps> = ({
  clip, onOpen, onRename, onDelete, onOpenFolder, onExport,
}) => {
  const { t, i18n } = useTranslation();
  const [menuOpen, setMenuOpen] = useState(false);
  const isAr = i18n.language === "ar";

  return (
    <div className="clip-card" onClick={onOpen}>
      {/* Thumbnail */}
      <div className="clip-thumb">
        {clip.thumbnailPath ? (
          <img src={`asset://${clip.thumbnailPath}`} alt={clip.name} className="clip-thumb-img" />
        ) : (
          <div className="clip-thumb-placeholder">
            <Film size={24} />
          </div>
        )}
        <span className="clip-duration">{formatDuration(clip.durationMs)}</span>
      </div>

      {/* Info */}
      <div className="clip-card-body">
        <p className="clip-name" title={clip.name}>{clip.name}</p>
        <p className="clip-meta">
          {formatRelativeTime(clip.createdAt, isAr)} · {formatFileSize(clip.fileSize, isAr)}
        </p>
      </div>

      {/* Actions menu */}
      <div className="clip-card-actions" onClick={(e) => e.stopPropagation()}>
        <button
          className="icon-btn"
          onClick={() => setMenuOpen((p) => !p)}
          aria-label="More actions"
        >
          <MoreHorizontal size={16} />
        </button>
        {menuOpen && (
          <div className="clip-menu" onMouseLeave={() => setMenuOpen(false)}>
            <button className="clip-menu-item" onClick={() => { onRename(); setMenuOpen(false); }}>
              <Edit2 size={14} /> {t("library.rename")}
            </button>
            <button className="clip-menu-item" onClick={() => { onOpenFolder(); setMenuOpen(false); }}>
              <FolderOpen size={14} /> {t("library.openFolder")}
            </button>
            <button className="clip-menu-item" onClick={() => { onExport(); setMenuOpen(false); }}>
              <Download size={14} /> {t("library.export")}
            </button>
            <div className="clip-menu-divider" />
            <button className="clip-menu-item clip-menu-danger" onClick={() => { onDelete(); setMenuOpen(false); }}>
              <Trash2 size={14} /> {t("library.delete")}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
