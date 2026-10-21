import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Keyboard } from "lucide-react";
import type { Settings } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

interface ShortcutRowProps {
  label: string;
  value: string;
  onChange: (v: string) => void;
}

const ShortcutRow: React.FC<ShortcutRowProps> = ({ label, value, onChange }) => {
  const { t } = useTranslation();
  const [listening, setListening] = useState(false);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.preventDefault();
    if (e.key === "Escape") { setListening(false); return; }
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
    if (!["Control", "Shift", "Alt", "Meta"].includes(e.key)) parts.push(key);
    if (parts.length >= 2) {
      onChange(parts.join("+"));
      setListening(false);
    }
  };

  return (
    <div className="shortcut-row">
      <span className="input-label">{label}</span>
      <div className="shortcut-input-wrap">
        {listening ? (
          <input
            className="input-field shortcut-input listening"
            autoFocus
            readOnly
            placeholder={t("settings.pressKeys")}
            onKeyDown={handleKeyDown}
            onBlur={() => setListening(false)}
          />
        ) : (
          <div className="shortcut-display">
            {value.split("+").map((k) => (
              <kbd key={k} className="kbd">{k}</kbd>
            ))}
          </div>
        )}
        <button
          className="btn btn-ghost btn-sm"
          onClick={() => setListening(true)}
          disabled={listening}
        >
          <Keyboard size={14} />
          <span className="btn-label">{t("settings.changeShortcut")}</span>
        </button>
      </div>
    </div>
  );
};

export const ShortcutSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const sh = settings.shortcuts;

  const save = async (patch: Partial<Settings["shortcuts"]>) => {
    try {
      await onSave({ shortcuts: { ...sh, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.shortcuts")}</h2>
      <div className="settings-group">
        <ShortcutRow
          label={t("settings.saveShortcut")}
          value={sh.saveReplay}
          onChange={(v) => save({ saveReplay: v })}
        />
        <ShortcutRow
          label={t("settings.toggleShortcut")}
          value={sh.toggleReplay}
          onChange={(v) => save({ toggleReplay: v })}
        />
        <ShortcutRow
          label={t("settings.showWindowShortcut")}
          value={sh.showWindow}
          onChange={(v) => save({ showWindow: v })}
        />
      </div>
    </div>
  );
};
