import { useState, useEffect, useCallback } from "react";
import { getSettings, updateSettings } from "../services/api";
import type { Settings } from "../types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let active = true;
    setLoading(true);
    getSettings()
      .then((s) => { if (active) { setSettings(s); setLoading(false); } })
      .catch((e) => { if (active) { setError(String(e)); setLoading(false); } });
    return () => { active = false; };
  }, []);

  const save = useCallback(async (partial: Partial<Settings>) => {
    setSaving(true);
    try {
      await updateSettings(partial);
      const fresh = await getSettings();
      setError(null);
      setSettings(fresh);
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setSaving(false);
    }
  }, []);

  return { settings, loading, error, saving, save };
}
