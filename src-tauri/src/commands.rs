use tauri::Manager;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::settings::config::Settings;
use crate::library::manager::ClipInfo;
use crate::export::exporter::{ExportConfig, ExportProgress};
use crate::capture::monitor::MonitorInfo;
use crate::audio::device::AudioDevice;
use crate::encoding::encoder::EncoderInfo;
use crate::replay_buffer::engine::ReplayStatus;
use crate::diagnostics::SystemInfo;

// ─── Replay Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_replay(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.replay_engine.lock().unwrap();
    engine.start().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_replay(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.replay_engine.lock().unwrap();
    engine.stop();
    Ok(())
}

#[tauri::command]
pub async fn save_replay(state: State<'_, AppState>) -> Result<ClipInfo, String> {
    let output_path = {
        let lib = state.library.lock().unwrap();
        lib.next_clip_path()
    };

    let result = {
        let engine = state.replay_engine.lock().unwrap();
        engine.save_replay(&output_path).map_err(|e| e.to_string())?
    };

    let (path, duration_ms, width, height, fps, has_audio) = result;

    let clip = {
        let lib = state.library.lock().unwrap();
        lib.add_clip(&path, duration_ms, width, height, fps, has_audio)
            .map_err(|e| e.to_string())?
    };

    Ok(clip)
}

#[tauri::command]
pub async fn get_replay_status(state: State<'_, AppState>) -> Result<ReplayStatus, String> {
    let engine = state.replay_engine.lock().unwrap();
    Ok(engine.get_status())
}

// ─── Capture Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_available_monitors() -> Result<Vec<MonitorInfo>, String> {
    Ok(crate::capture::monitor::enumerate_monitors())
}

#[tauri::command]
pub async fn get_available_audio_devices() -> Result<Vec<AudioDevice>, String> {
    crate::audio::device::enumerate_audio_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_encoder_capabilities() -> Result<Vec<EncoderInfo>, String> {
    Ok(crate::encoding::encoder::enumerate_encoders())
}

// ─── Library Commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_clips(
    state: State<'_, AppState>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    search: Option<String>,
) -> Result<Vec<ClipInfo>, String> {
    let lib = state.library.lock().unwrap();
    lib.get_clips(
        sort_field.as_deref().unwrap_or("date"),
        sort_order.as_deref().unwrap_or("desc"),
        search.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_clip(state: State<'_, AppState>, id: String) -> Result<Option<ClipInfo>, String> {
    let lib = state.library.lock().unwrap();
    lib.get_clip(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_clip(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let lib = state.library.lock().unwrap();
    lib.delete_clip(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_clip(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    let lib = state.library.lock().unwrap();
    lib.rename_clip(&id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_clip_folder(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let path = {
        let lib = state.library.lock().unwrap();
        lib.get_clip(&id)
            .map_err(|e| e.to_string())?
            .map(|c| c.file_path)
            .ok_or_else(|| format!("Clip not found: {}", id))?
    };

    let folder = std::path::PathBuf::from(&path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path);

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(&folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_clip_thumbnail(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let lib = state.library.lock().unwrap();
    lib.get_clip_thumbnail_b64(&id).map_err(|e| e.to_string())
}

// ─── Settings Commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: serde_json::Value,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let current = state.settings.lock().unwrap().clone();
    let mut merged = serde_json::to_value(&current).map_err(|e| e.to_string())?;
    merge_settings(&mut merged, settings.clone());
    let mut s: Settings = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    s.validate();

    let data_dir = {
        use tauri::Manager;
        app.path().app_data_dir().map_err(|e| e.to_string())?
    };

    s.save(&data_dir).map_err(|e| e.to_string())?;

    {
        let mut current = state.settings.lock().unwrap();
        *current = s.clone();
    }

    {
        let engine = state.replay_engine.lock().unwrap();
        engine.update_settings(&s);
    }

    if settings.get("shortcuts").is_some() && s.shortcuts != current.shortcuts {
        crate::register_shortcuts(&app, &s.shortcuts).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn merge_settings(target: &mut serde_json::Value, patch: serde_json::Value) {
    match (target, patch) {
        (serde_json::Value::Object(target), serde_json::Value::Object(patch)) => {
            for (key, value) in patch {
                merge_settings(target.entry(key).or_insert(serde_json::Value::Null), value);
            }
        }
        (target, patch) => *target = patch,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_by_clips_bytes: u64,
    pub temp_bytes: u64,
}

#[tauri::command]
pub async fn get_storage_info(state: State<'_, AppState>) -> Result<StorageInfo, String> {
    let lib = state.library.lock().unwrap();
    let (used_by_clips, _) = lib.get_disk_usage().map_err(|e| e.to_string())?;
    let clips_dir = lib.clips_dir().clone();
    drop(lib);

    let (total, free) = crate::diagnostics::get_storage_info(&clips_dir);

    Ok(StorageInfo {
        total_bytes: total,
        free_bytes: free,
        used_by_clips_bytes: used_by_clips,
        temp_bytes: 0,
    })
}

// ─── Export Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_export(
    state: State<'_, AppState>,
    config: ExportConfig,
) -> Result<(), String> {
    let em = state.export_manager.lock().unwrap();
    em.start(config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_export(state: State<'_, AppState>) -> Result<(), String> {
    let em = state.export_manager.lock().unwrap();
    em.cancel();
    Ok(())
}

#[tauri::command]
pub async fn get_export_progress(state: State<'_, AppState>) -> Result<ExportProgress, String> {
    let em = state.export_manager.lock().unwrap();
    Ok(em.get_progress())
}

// ─── System Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(crate::diagnostics::get_system_info())
}

#[tauri::command]
pub async fn open_logs_folder(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let logs_dir = data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(logs_dir.to_string_lossy().as_ref())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
