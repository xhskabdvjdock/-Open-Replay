import { useState, useEffect, useRef, useCallback } from "react";
import { startExport, cancelExport, getExportProgress } from "../services/api";
import type { ExportConfig, ExportProgress } from "../types";

export function useExport() {
  const [progress, setProgress] = useState<ExportProgress>({
    percentage: 0,
    currentTimeMs: 0,
    totalTimeMs: 0,
    status: "idle",
    errorMessage: null,
  });
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const stopPolling = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  const startPolling = () => {
    stopPolling();
    intervalRef.current = setInterval(async () => {
      try {
        const p = await getExportProgress();
        setProgress(p);
        if (p.status !== "exporting") stopPolling();
      } catch {
        stopPolling();
      }
    }, 500);
  };

  useEffect(() => () => stopPolling(), []);

  const start = useCallback(async (config: ExportConfig) => {
    await startExport(config);
    startPolling();
  }, []);

  const cancel = useCallback(async () => {
    await cancelExport();
  }, []);

  return { progress, start, cancel };
}
