import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "../../hooks/useSettings";
import { useToast } from "../../components/common/Toast";

// Sub-section components
import { ReplaySettingsSection } from "./ReplaySettings";
import { CaptureSettingsSection } from "./CaptureSettings";
import { VideoSettingsSection } from "./VideoSettings";
import { AudioSettingsSection } from "./AudioSettings";
import { StorageSettingsSection } from "./StorageSettings";
import { ShortcutSettingsSection } from "./ShortcutSettings";
import { AppearanceSettingsSection } from "./AppearanceSettings";

const SECTIONS = [
  { key: "replay",      label: "settings.replay" },
  { key: "capture",     label: "settings.capture" },
  { key: "video",       label: "settings.video" },
  { key: "audio",       label: "settings.audio" },
  { key: "storage",     label: "settings.storage" },
  { key: "shortcuts",   label: "settings.shortcuts" },
  { key: "appearance",  label: "settings.appearance" },
] as const;

type SectionKey = (typeof SECTIONS)[number]["key"];

export const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings, loading, error, save } = useSettings();
  const toast = useToast();
  const [active, setActive] = useState<SectionKey>("replay");

  if (loading) {
    return <div className="page-content"><div className="loading-state">{t("common.loading")}</div></div>;
  }

  if (error && !settings) {
    return <div className="page-content"><div className="error-state">{error}</div></div>;
  }
  if (!settings) return null;

  const sectionProps = { settings, onSave: save, toast };

  const renderSection = () => {
    switch (active) {
      case "replay":     return <ReplaySettingsSection {...sectionProps} />;
      case "capture":    return <CaptureSettingsSection {...sectionProps} />;
      case "video":      return <VideoSettingsSection {...sectionProps} />;
      case "audio":      return <AudioSettingsSection {...sectionProps} />;
      case "storage":    return <StorageSettingsSection {...sectionProps} />;
      case "shortcuts":  return <ShortcutSettingsSection {...sectionProps} />;
      case "appearance": return <AppearanceSettingsSection {...sectionProps} />;
    }
  };

  return (
    <div className="page-content settings-page">
      <div className="page-header">
        <h1 className="page-title">{t("settings.title")}</h1>
      </div>
      <div className="settings-layout">
        {/* Section tabs */}
        <nav className="settings-nav">
          {SECTIONS.map(({ key, label }) => (
            <button
              key={key}
              className={`settings-nav-item${active === key ? " active" : ""}`}
              onClick={() => setActive(key)}
            >
              {t(label)}
            </button>
          ))}
        </nav>
        {/* Content */}
        <div className="settings-content">
          {renderSection()}
        </div>
      </div>
    </div>
  );
};
