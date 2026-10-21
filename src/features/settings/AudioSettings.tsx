import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw } from "lucide-react";
import { Toggle } from "../../components/common/Toggle";
import { Select } from "../../components/common/Select";
import { Slider } from "../../components/common/Slider";
import { getAvailableAudioDevices } from "../../services/api";
import type { Settings, AudioDevice } from "../../types";

interface Props {
  settings: Settings;
  onSave: (p: Partial<Settings>) => Promise<void>;
  toast: { success: (m: string) => void; error: (m: string) => void };
}

export const AudioSettingsSection: React.FC<Props> = ({ settings, onSave, toast }) => {
  const { t } = useTranslation();
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [loading, setLoading] = useState(false);
  const a = settings.audio;

  const loadDevices = async () => {
    setLoading(true);
    try { setDevices(await getAvailableAudioDevices()); }
    catch (e) { toast.error(String(e)); }
    finally { setLoading(false); }
  };

  useEffect(() => { loadDevices(); }, []);

  const save = async (patch: Partial<Settings["audio"]>) => {
    try {
      await onSave({ audio: { ...a, ...patch } });
      toast.success(t("settings.saved"));
    } catch (e) { toast.error(String(e)); }
  };

  const outputDevices = devices.filter((d) => d.deviceType === "output");
  const inputDevices = devices.filter((d) => d.deviceType === "input");

  const toOptions = (devs: AudioDevice[]) =>
    devs.map((d) => ({ value: d.id, label: d.name + (d.isDefault ? " ★" : "") }));

  return (
    <div className="settings-section">
      <h2 className="settings-section-title">{t("settings.audio")}</h2>

      <div className="settings-group">
        {/* Refresh */}
        <div className="settings-row-end">
          <button className="btn btn-ghost btn-sm" onClick={loadDevices} disabled={loading}>
            <RefreshCw size={14} className={loading ? "spin" : ""} />
            <span className="btn-label">{t("settings.refreshDevices")}</span>
          </button>
        </div>

        {/* System audio */}
        <Toggle
          label={t("settings.systemAudio")}
          checked={a.systemAudioEnabled}
          onChange={(v) => save({ systemAudioEnabled: v })}
        />
        {a.systemAudioEnabled && (
          <>
            <Select
              label={t("settings.audioDevice")}
              options={toOptions(outputDevices).length ? toOptions(outputDevices) : [{ value: "", label: t("common.unknown") }]}
              value={a.systemAudioDevice}
              onChange={(e) => save({ systemAudioDevice: e.target.value })}
            />
            <Slider
              label={t("settings.volume")}
              min={0} max={100} step={1}
              value={a.systemVolume}
              onChange={(v) => save({ systemVolume: v })}
              formatValue={(v) => `${v}%`}
            />
          </>
        )}

        <div className="settings-divider" />

        {/* Microphone */}
        <Toggle
          label={t("settings.microphone")}
          checked={a.microphoneEnabled}
          onChange={(v) => save({ microphoneEnabled: v })}
        />
        {a.microphoneEnabled && (
          <>
            <Select
              label={t("settings.micDevice")}
              options={toOptions(inputDevices).length ? toOptions(inputDevices) : [{ value: "", label: t("common.unknown") }]}
              value={a.microphoneDevice}
              onChange={(e) => save({ microphoneDevice: e.target.value })}
            />
            <Toggle
              label={t("settings.mute")}
              checked={a.microphoneMuted}
              onChange={(v) => save({ microphoneMuted: v })}
            />
            {!a.microphoneMuted && (
              <Slider
                label={t("settings.volume")}
                min={0} max={100} step={1}
                value={a.micVolume}
                onChange={(v) => save({ micVolume: v })}
                formatValue={(v) => `${v}%`}
              />
            )}
          </>
        )}
      </div>
    </div>
  );
};
