import { useState, useEffect, useRef } from "react";
import { getReplayStatus } from "../services/api";
import type { ReplayStatus } from "../types";

const DEFAULT_STATUS: ReplayStatus = {
  isRecording: false,
  bufferDurationMs: 0,
  bufferFillPercent: 0,
  actualFps: 0,
  targetFps: 30,
  monitorName: "—",
  resolution: "—",
  hasAudio: false,
  hasMicrophone: false,
};

export function useReplayStatus(pollIntervalMs = 1000) {
  const [status, setStatus] = useState<ReplayStatus>(DEFAULT_STATUS);
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    let active = true;

    const poll = async () => {
      try {
        const s = await getReplayStatus();
        if (active) {
          setStatus(s);
          setError(null);
        }
      } catch (e) {
        if (active) setError(String(e));
      }
    };

    poll();
    intervalRef.current = setInterval(poll, pollIntervalMs);

    return () => {
      active = false;
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [pollIntervalMs]);

  return { status, error };
}
