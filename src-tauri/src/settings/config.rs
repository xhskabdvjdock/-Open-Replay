use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Context, Result};
use tracing::{info, warn};

/// Complete application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub replay: ReplaySettings,
    pub capture: CaptureSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub storage: StorageSettings,
    pub shortcuts: ShortcutSettings,
    pub appearance: AppearanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySettings {
    pub enabled: bool,
    #[serde(alias = "buffer_duration_sec")]
    pub buffer_duration_sec: u32,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSettings {
    #[serde(alias = "monitor_id")]
    pub monitor_id: String,
    pub fps: u32,
    pub resolution: String,
    #[serde(alias = "capture_cursor")]
    pub capture_cursor: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    pub codec: String,
    pub encoder: String,
    pub quality: String,
    #[serde(alias = "bitrate_kbps")]
    pub bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    #[serde(alias = "system_audio_enabled")]
    pub system_audio_enabled: bool,
    #[serde(alias = "system_audio_device")]
    pub system_audio_device: String,
    #[serde(alias = "microphone_enabled")]
    pub microphone_enabled: bool,
    #[serde(alias = "microphone_device")]
    pub microphone_device: String,
    #[serde(alias = "microphone_muted")]
    pub microphone_muted: bool,
    #[serde(alias = "system_volume")]
    pub system_volume: f32,
    #[serde(alias = "mic_volume")]
    pub mic_volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSettings {
    #[serde(alias = "clips_folder")]
    pub clips_folder: String,
    #[serde(alias = "temp_folder")]
    pub temp_folder: String,
    #[serde(alias = "max_storage_gb")]
    pub max_storage_gb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettings {
    #[serde(alias = "save_replay")]
    pub save_replay: String,
    #[serde(alias = "toggle_replay")]
    pub toggle_replay: String,
    #[serde(alias = "show_window")]
    pub show_window: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        let app_data = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.openreplay.app");

        Self {
            replay: ReplaySettings {
                enabled: true,
                buffer_duration_sec: 30,
                auto_start: true,
            },
            capture: CaptureSettings {
                monitor_id: String::from("primary"),
                fps: 60,
                resolution: String::from("native"),
                capture_cursor: true,
            },
            video: VideoSettings {
                codec: String::from("h264"),
                encoder: String::from("auto"),
                quality: String::from("high"),
                bitrate_kbps: 8000,
            },
            audio: AudioSettings {
                system_audio_enabled: true,
                system_audio_device: String::from("default"),
                microphone_enabled: false,
                microphone_device: String::from("default"),
                microphone_muted: false,
                system_volume: 1.0,
                mic_volume: 1.0,
            },
            storage: StorageSettings {
                clips_folder: app_data.join("clips").to_string_lossy().into_owned(),
                temp_folder: app_data.join("temp").to_string_lossy().into_owned(),
                max_storage_gb: 10.0,
            },
            shortcuts: ShortcutSettings {
                save_replay: String::from("Ctrl+Shift+F10"),
                toggle_replay: String::from("Ctrl+Shift+F9"),
                show_window: String::from("Ctrl+Shift+F8"),
            },
            appearance: AppearanceSettings {
                theme: String::from("dark"),
                language: String::from("ar"),
            },
        }
    }
}

impl Settings {
    /// Load settings from disk, falling back to defaults on error
    pub fn load(config_dir: &PathBuf) -> Self {
        let path = config_dir.join("settings.toml");
        match Self::load_from_path(&path) {
            Ok(settings) => {
                info!("Settings loaded from {:?}", path);
                settings
            }
            Err(e) => {
                warn!("Failed to load settings: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Reading settings file {:?}", path))?;
        toml::from_str(&contents)
            .with_context(|| "Parsing settings TOML")
    }

    /// Save settings to disk
    pub fn save(&self, config_dir: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(config_dir)?;
        let path = config_dir.join("settings.toml");
        let contents = toml::to_string_pretty(self)
            .with_context(|| "Serializing settings")?;
        std::fs::write(&path, contents)
            .with_context(|| format!("Writing settings to {:?}", path))?;
        info!("Settings saved to {:?}", path);
        Ok(())
    }

    /// Validate settings and clamp to valid ranges
    pub fn validate(&mut self) {
        self.replay.buffer_duration_sec = self.replay.buffer_duration_sec.clamp(5, 600);
        self.capture.fps = match self.capture.fps {
            f if f <= 30 => 30,
            f if f <= 60 => 60,
            f if f <= 120 => 120,
            f if f <= 144 => 144,
            _ => 240,
        };
        self.audio.system_volume = self.audio.system_volume.clamp(0.0, 1.0);
        self.audio.mic_volume = self.audio.mic_volume.clamp(0.0, 1.0);
        self.video.bitrate_kbps = self.video.bitrate_kbps.clamp(500, 100_000);
        self.storage.max_storage_gb = self.storage.max_storage_gb.clamp(0.5, 1000.0);
    }
}
