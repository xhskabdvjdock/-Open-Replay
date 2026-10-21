// ── Monitor ──────────────────────────────────────────────────────────────────

export interface MonitorInfo {
  id: string;
  name: string;
  width: number;
  height: number;
  x: number;
  y: number;
  scaleFactor: number;
  isPrimary: boolean;
}

// ── Audio ─────────────────────────────────────────────────────────────────────

export interface AudioDevice {
  id: string;
  name: string;
  deviceType: "input" | "output";
  isDefault: boolean;
}

// ── Encoder ──────────────────────────────────────────────────────────────────

export interface EncoderInfo {
  name: string;
  encoderType: "hardware" | "software";
  codec: string;
  isAvailable: boolean;
}

// ── Replay ───────────────────────────────────────────────────────────────────

export interface ReplayStatus {
  isRecording: boolean;
  bufferDurationMs: number;
  bufferFillPercent: number;
  actualFps: number;
  targetFps: number;
  monitorName: string;
  resolution: string;
  hasAudio: boolean;
  hasMicrophone: boolean;
}

// ── Clip ─────────────────────────────────────────────────────────────────────

export interface Clip {
  id: string;
  name: string;
  filePath: string;
  thumbnailPath: string | null;
  createdAt: string;
  durationMs: number;
  width: number;
  height: number;
  fps: number;
  fileSize: number;
  hasAudio: boolean;
  codec: string;
  notes: string | null;
}

export interface ClipInfo {
  durationMs: number;
  width: number;
  height: number;
  fps: number;
  codec: string;
  hasAudio: boolean;
  audioCodec: string | null;
}

export type ClipSortField = "date" | "name" | "duration" | "size";
export type ClipSortOrder = "asc" | "desc";

// ── Export ───────────────────────────────────────────────────────────────────

export interface ExportConfig {
  clipId: string;
  inputPath: string;
  outputPath: string;
  fileName: string;
  format: string;
  codec: string;
  quality: string;
  resolution: string;
  fps: number;
  startMs: number;
  endMs: number;
}

export interface ExportProgress {
  percentage: number;
  currentTimeMs: number;
  totalTimeMs: number;
  status: "idle" | "exporting" | "complete" | "cancelled" | "error";
  errorMessage: string | null;
}

// ── Settings ─────────────────────────────────────────────────────────────────

export interface ReplaySettings {
  enabled: boolean;
  bufferDurationSec: number;
  autoStart: boolean;
}

export interface CaptureSettings {
  monitorId: string;
  fps: number;
  resolution: string;
  captureCursor: boolean;
}

export interface VideoSettings {
  codec: string;
  encoder: string;
  quality: string;
  bitrate: number;
}

export interface AudioSettings {
  systemAudioEnabled: boolean;
  systemAudioDevice: string;
  microphoneEnabled: boolean;
  microphoneDevice: string;
  microphoneMuted: boolean;
  systemVolume: number;
  micVolume: number;
}

export interface StorageSettings {
  clipsFolder: string;
  tempFolder: string;
  maxStorageGb: number;
}

export interface ShortcutSettings {
  saveReplay: string;
  toggleReplay: string;
  showWindow: string;
}

export interface AppearanceSettings {
  theme: "dark" | "light";
  language: "ar" | "en";
}

export interface Settings {
  replay: ReplaySettings;
  capture: CaptureSettings;
  video: VideoSettings;
  audio: AudioSettings;
  storage: StorageSettings;
  shortcuts: ShortcutSettings;
  appearance: AppearanceSettings;
}

// ── Storage / System ─────────────────────────────────────────────────────────

export interface StorageInfo {
  totalBytes: number;
  freeBytes: number;
  usedByClipsBytes: number;
  tempBytes: number;
}

export interface SystemInfo {
  os: string;
  gpuName: string;
  totalRamMb: number;
  cpuName: string;
}
