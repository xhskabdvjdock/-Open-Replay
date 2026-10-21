use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Central application state shared across all Tauri commands
pub struct AppState {
    pub replay_engine: Arc<Mutex<crate::replay_buffer::engine::ReplayEngine>>,
    pub library: Arc<Mutex<crate::library::manager::Library>>,
    pub settings: Arc<Mutex<crate::settings::config::Settings>>,
    pub export_manager: Arc<Mutex<crate::export::exporter::ExportManager>>,
    pub event_tx: broadcast::Sender<AppEvent>,
}

/// Events emitted by the backend to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    ReplayStatusChanged(crate::replay_buffer::engine::ReplayStatus),
    ClipSaved(crate::library::manager::ClipInfo),
    ExportProgress(crate::export::exporter::ExportProgress),
    AudioLevels(AudioLevels),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevels {
    pub system_rms: f32,
    pub mic_rms: f32,
}

impl AppState {
    pub fn new(
        replay_engine: crate::replay_buffer::engine::ReplayEngine,
        library: crate::library::manager::Library,
        settings: crate::settings::config::Settings,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        Self {
            replay_engine: Arc::new(Mutex::new(replay_engine)),
            library: Arc::new(Mutex::new(library)),
            settings: Arc::new(Mutex::new(settings)),
            export_manager: Arc::new(Mutex::new(crate::export::exporter::ExportManager::new())),
            event_tx,
        }
    }
}
