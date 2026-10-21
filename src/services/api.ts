import { invoke } from "@tauri-apps/api/core";
import type {
  MonitorInfo,
  AudioDevice,
  EncoderInfo,
  ReplayStatus,
  Clip,
  ClipSortField,
  ClipSortOrder,
  ClipInfo,
  ExportConfig,
  ExportProgress,
  Settings,
  StorageInfo,
  SystemInfo,
} from "../types";

// ── Replay ──────────────────────────────────────────────────────────────────

export async function startReplay(): Promise<void> {
  await invoke("start_replay");
}

export async function stopReplay(): Promise<void> {
  await invoke("stop_replay");
}

export async function saveReplay(): Promise<Clip> {
  return await invoke<Clip>("save_replay");
}

export async function getReplayStatus(): Promise<ReplayStatus> {
  return await invoke<ReplayStatus>("get_replay_status");
}

// ── Capture ──────────────────────────────────────────────────────────────────

export async function getAvailableMonitors(): Promise<MonitorInfo[]> {
  return await invoke<MonitorInfo[]>("get_available_monitors");
}

export async function getAvailableAudioDevices(): Promise<AudioDevice[]> {
  return await invoke<AudioDevice[]>("get_available_audio_devices");
}

export async function getEncoderCapabilities(): Promise<EncoderInfo[]> {
  return await invoke<EncoderInfo[]>("get_encoder_capabilities");
}

// ── Library ──────────────────────────────────────────────────────────────────

export async function getClips(
  sortField?: ClipSortField,
  sortOrder?: ClipSortOrder,
  search?: string
): Promise<Clip[]> {
  return await invoke<Clip[]>("get_clips", {
    sortField: sortField ?? "date",
    sortOrder: sortOrder ?? "desc",
    search: search ?? "",
  });
}

export async function getClip(id: string): Promise<Clip> {
  return await invoke<Clip>("get_clip", { id });
}

export async function deleteClip(id: string): Promise<void> {
  await invoke("delete_clip", { id });
}

export async function renameClip(id: string, newName: string): Promise<void> {
  await invoke("rename_clip", { id, newName });
}

export async function openClipFolder(id: string): Promise<void> {
  await invoke("open_clip_folder", { id });
}

export async function getClipThumbnail(id: string): Promise<string> {
  return await invoke<string>("get_clip_thumbnail", { id });
}

// ── Editor ───────────────────────────────────────────────────────────────────

export async function getClipInfo(clipId: string): Promise<ClipInfo> {
  return await invoke<ClipInfo>("get_clip_info", { clipId });
}

export async function trimClip(
  clipId: string,
  startMs: number,
  endMs: number
): Promise<Clip> {
  return await invoke<Clip>("trim_clip", { clipId, startMs, endMs });
}

export async function previewFrame(
  clipId: string,
  timestampMs: number
): Promise<string> {
  return await invoke<string>("preview_frame", { clipId, timestampMs });
}

// ── Export ───────────────────────────────────────────────────────────────────

export async function startExport(config: ExportConfig): Promise<void> {
  await invoke("start_export", { config });
}

export async function cancelExport(): Promise<void> {
  await invoke("cancel_export");
}

export async function getExportProgress(): Promise<ExportProgress> {
  return await invoke<ExportProgress>("get_export_progress");
}

// ── Settings ─────────────────────────────────────────────────────────────────

export async function getSettings(): Promise<Settings> {
  return await invoke<Settings>("get_settings");
}

export async function updateSettings(settings: Partial<Settings>): Promise<void> {
  await invoke("update_settings", { settings });
}

export async function getStorageInfo(): Promise<StorageInfo> {
  return await invoke<StorageInfo>("get_storage_info");
}

// ── System ───────────────────────────────────────────────────────────────────

export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke<SystemInfo>("get_system_info");
}

export async function openLogsFolder(): Promise<void> {
  await invoke("open_logs_folder");
}
