import React, { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Search, Grid, List, Film } from "lucide-react";

import { useClips } from "../../hooks/useClips";
import { useToast } from "../../components/common/Toast";
import { Input } from "../../components/common/Input";
import { Select } from "../../components/common/Select";
import { EmptyState } from "../../components/common/EmptyState";
import { ClipCard } from "../../components/clips/ClipCard";
import { ClipRow } from "../../components/clips/ClipRow";
import { DeleteConfirmModal } from "../../components/clips/DeleteConfirmModal";
import * as api from "../../services/api";
import type { ClipSortField, ClipSortOrder } from "../../types";

export const LibraryPage: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const toast = useToast();

  const [search, setSearch] = useState("");
  const [sortField, setSortField] = useState<ClipSortField>("date");
  const [sortOrder] = useState<ClipSortOrder>("desc");
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; name: string } | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [renaming, setRenaming] = useState<{ id: string; name: string } | null>(null);
  const [newName, setNewName] = useState("");

  const { clips, loading, error, remove, rename, refresh } = useClips(sortField, sortOrder, search);

  const handleDelete = useCallback(async () => {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await remove(deleteTarget.id);
      toast.success(`${deleteTarget.name} — deleted`);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setDeleting(false);
      setDeleteTarget(null);
    }
  }, [deleteTarget, remove, toast]);

  const handleRename = useCallback(async () => {
    if (!renaming || !newName.trim()) return;
    try {
      await rename(renaming.id, newName.trim());
      toast.success(t("settings.saved"));
    } catch (e) {
      toast.error(String(e));
    } finally {
      setRenaming(null);
      setNewName("");
    }
  }, [renaming, newName, rename, t, toast]);

  const handleOpenFolder = useCallback(async (id: string) => {
    try { await api.openClipFolder(id); }
    catch (e) { toast.error(String(e)); }
  }, [toast]);

  const sortOptions = [
    { value: "date", label: t("library.date") },
    { value: "name", label: t("library.name") },
    { value: "duration", label: t("library.duration") },
    { value: "size", label: t("library.size") },
  ];

  const renderContent = () => {
    if (loading) return <div className="loading-state">{t("common.loading")}</div>;
    if (error) return <div className="error-state">{error}</div>;
    if (clips.length === 0) {
      return (
        <EmptyState
          icon={<Film size={48} />}
          title={t("library.empty.title")}
          description={t("library.empty.description")}
        />
      );
    }

    if (viewMode === "grid") {
      return (
        <div className="clips-grid">
          {clips.map((clip) => (
            <ClipCard
              key={clip.id}
              clip={clip}
              onOpen={() => navigate(`/editor/${clip.id}`)}
              onRename={() => { setRenaming({ id: clip.id, name: clip.name }); setNewName(clip.name); }}
              onDelete={() => setDeleteTarget({ id: clip.id, name: clip.name })}
              onOpenFolder={() => handleOpenFolder(clip.id)}
              onExport={() => navigate(`/editor/${clip.id}?export=1`)}
            />
          ))}
        </div>
      );
    }

    return (
      <div className="clips-list">
        <div className="clips-list-header">
          <span>{t("library.name")}</span>
          <span>{t("library.duration")}</span>
          <span>{t("library.resolution", "Res")}</span>
          <span>FPS</span>
          <span>{t("library.date")}</span>
          <span>{t("library.size")}</span>
          <span />
        </div>
        {clips.map((clip) => (
          <ClipRow
            key={clip.id}
            clip={clip}
            onOpen={() => navigate(`/editor/${clip.id}`)}
            onRename={() => { setRenaming({ id: clip.id, name: clip.name }); setNewName(clip.name); }}
            onDelete={() => setDeleteTarget({ id: clip.id, name: clip.name })}
            onOpenFolder={() => handleOpenFolder(clip.id)}
            onExport={() => navigate(`/editor/${clip.id}?export=1`)}
          />
        ))}
      </div>
    );
  };

  return (
    <div className="page-content">
      <div className="page-header">
        <h1 className="page-title">{t("library.title")}</h1>
        <div className="page-header-actions">
          <Input
            icon={<Search size={14} />}
            placeholder={t("library.search")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <Select
            options={sortOptions}
            value={sortField}
            onChange={(e) => setSortField(e.target.value as ClipSortField)}
          />
          <div className="view-toggle">
            <button
              className={`icon-btn${viewMode === "grid" ? " active" : ""}`}
              onClick={() => setViewMode("grid")}
              title={t("library.gridView")}
            ><Grid size={16} /></button>
            <button
              className={`icon-btn${viewMode === "list" ? " active" : ""}`}
              onClick={() => setViewMode("list")}
              title={t("library.listView")}
            ><List size={16} /></button>
          </div>
        </div>
      </div>

      {renderContent()}

      {/* Delete confirmation */}
      <DeleteConfirmModal
        open={!!deleteTarget}
        clipName={deleteTarget?.name ?? ""}
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
        loading={deleting}
      />

      {/* Rename dialog */}
      {renaming && (
        <div className="modal-backdrop" onClick={() => setRenaming(null)}>
          <div className="modal-panel" style={{ maxWidth: 400 }} onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2 className="modal-title">{t("library.rename")}</h2>
            </div>
            <div className="modal-body">
              <Input
                label={t("library.name")}
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleRename(); }}
                autoFocus
              />
            </div>
            <div className="modal-footer">
              <button className="btn btn-ghost btn-md" onClick={() => setRenaming(null)}>{t("common.cancel")}</button>
              <button className="btn btn-primary btn-md" onClick={handleRename}>{t("common.save")}</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
