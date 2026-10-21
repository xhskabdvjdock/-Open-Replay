import React from "react";
import { useTranslation } from "react-i18next";
import { Select } from "../../components/common/Select";
import type { Settings } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

export const AppearanceSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t, i18n } = useTranslation();
  const a = settings.appearance;

  const save = async (patch: Partial<Settings["appearance"]>) => {
    try {
      await onSave({ appearance: { ...a, ...patch } });
      toast.success(t("settings.saved"));
      if (patch.language) {
        i18n.changeLanguage(patch.language);
        document.documentElement.dir = patch.language === "ar" ? "rtl" : "ltr";
        document.documentElement.lang = patch.language;
      }
    } catch (e) { toast.error(String(e)); }
  };

  const themeOptions = [
    { value: "dark",  label: t("settings.dark") },
    { value: "light", label: t("settings.light") },
  ];

  const langOptions = [
    { value: "ar", label: "العربية" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.appearance")}</h2>
      <div className="settings-group">
        <Select
          label={t("settings.theme")}
          options={themeOptions}
          value={a.theme}
          onChange={(e) => save({ theme: e.target.value as "dark" | "light" })}
        />
        <Select
          label={t("settings.language")}
          options={langOptions}
          value={a.language}
          onChange={(e) => save({ language: e.target.value as "ar" | "en" })}
        />
      </div>
    </div>
  );
};
