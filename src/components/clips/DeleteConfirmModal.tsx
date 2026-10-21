import React from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle } from "lucide-react";
import { Modal } from "../common/Modal";
import { Button } from "../common/Button";

interface DeleteConfirmModalProps {
  open: boolean;
  clipName: string;
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

export const DeleteConfirmModal: React.FC<DeleteConfirmModalProps> = ({
  open, clipName, onConfirm, onCancel, loading = false,
}) => {
  const { t } = useTranslation();
  return (
    <Modal
      open={open}
      onClose={onCancel}
      title={t("library.deleteConfirm")}
      actions={
        <>
          <Button variant="ghost" onClick={onCancel} disabled={loading}>{t("common.cancel")}</Button>
          <Button variant="danger" onClick={onConfirm} loading={loading}>{t("library.delete")}</Button>
        </>
      }
    >
      <div className="delete-confirm-body">
        <AlertTriangle size={36} className="delete-warn-icon" />
        <p className="delete-clip-name">«{clipName}»</p>
        <p className="delete-message">{t("library.deleteMessage")}</p>
      </div>
    </Modal>
  );
};
