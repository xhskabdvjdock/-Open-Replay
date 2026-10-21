import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import { Select } from "../../components/common/Select";
import { Toggle } from "../../components/common/Toggle";
import { getAvailableMonitors } from "../../services/api";
import type { Settings, MonitorInfo } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

export const CaptureSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const c = settings.capture;

  const loadMonitors = async () => {
    setLoading(true);
    try { setMonitors(await getAvailableMonitors()); }
    catch (e) { toast.error(String(e)); }
    finally { setLoading(false); }
  };

  useEffect(() => { loadMonitors(); }, []);

  const save = async (patch: Partial<Settings["capture"]>) => {
    try {
      await onSave({ capture: { ...c, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  const monitorOptions = monitors.map((m) => ({
    value: m.id,
    label: `${m.name} (${m.width}×${m.height})${m.isPrimary ? " ★" : ""}`,
  }));

  const fpsOptions = [
    { value: "30", label: "30 fps" },
    { value: "60", label: "60 fps" },
    { value: "120", label: "120 fps" },
  ];

  const resOptions = [
    { value: "native", label: t("export.original") },
    { value: "1920x1080", label: "1080p" },
    { value: "1280x720", label: "720p" },
  ];

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.capture")}</h2>

      <div className="settings-group">
        <div className="settings-row">
          <Select
            label={t("settings.monitor")}
            options={monitorOptions.length ? monitorOptions : [{ value: "", label: t("common.loading") }]}
            value={c.monitorId}
            onChange={(e) => save({ monitorId: e.target.value })}
          />
          <button className="btn btn-ghost btn-sm" onClick={loadMonitors} disabled={loading}>
            <RefreshCw size={14} className={loading ? "spin" : ""} />
            <span className="btn-label">{t("settings.refreshMonitors")}</span>
          </button>
        </div>

        <Select
          label={t("settings.fps")}
          options={fpsOptions}
          value={String(c.fps)}
          onChange={(e) => save({ fps: Number(e.target.value) })}
        />

        <Select
          label={t("settings.resolution")}
          options={resOptions}
          value={c.resolution}
          onChange={(e) => save({ resolution: e.target.value })}
        />

        <Toggle
          label={t("settings.cursor")}
          checked={c.captureCursor}
          onChange={(v) => save({ captureCursor: v })}
        />
      </div>
    </div>
  );
};
