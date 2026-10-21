// Formatting utilities
export function formatDuration(ms: number): string {
  if (!ms || ms < 0) return "0:00";
  const totalSec = Math.floor(ms / 1000);
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function formatFileSize(bytes: number, arabic = false): string {
  const locale = arabic ? "ar-EG" : "en-US";
  if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toLocaleString(locale, { maximumFractionDigits: 1 })} GB`;
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toLocaleString(locale, { maximumFractionDigits: 1 })} MB`;
  return `${(bytes / 1024).toLocaleString(locale, { maximumFractionDigits: 0 })} KB`;
}

export function formatRelativeTime(isoDate: string, arabic = false): string {
  const diff = Date.now() - new Date(isoDate).getTime();
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return arabic ? "الآن" : "just now";
  const min = Math.floor(sec / 60);
  if (min < 60) return arabic ? `منذ ${min} د` : `${min}m ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return arabic ? `منذ ${hr} س` : `${hr}h ago`;
  const day = Math.floor(hr / 24);
  return arabic ? `منذ ${day} ي` : `${day}d ago`;
}

export function formatBytes(bytes: number): string {
  return formatFileSize(bytes, false);
}

export function msToTime(ms: number): { h: number; m: number; s: number; ms: number } {
  const totalSec = Math.floor(ms / 1000);
  return {
    h: Math.floor(totalSec / 3600),
    m: Math.floor((totalSec % 3600) / 60),
    s: totalSec % 60,
    ms: ms % 1000,
  };
}
