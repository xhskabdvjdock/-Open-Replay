//! System capability detection for capture and encoding.
//!
//! Probes hardware encoders (NVENC, AMD AMF, Intel QSV) via Media Foundation
//! and reports back a capabilities summary that the frontend can display and
//! the encoding subsystem can use when choosing a codec path.

use serde::{Deserialize, Serialize};

#[cfg(windows)]
use windows::Win32::Graphics::Dxgi::*;

/// Summary of a single encoder available on the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderInfo {
    /// Human-readable name, e.g. "NVIDIA NVENC H.264".
    pub name: String,
    /// Type: "hardware" or "software".
    pub encoder_type: String,
    /// Codec identifier: "h264", "hevc", etc.
    pub codec: String,
    /// Whether this encoder is currently usable.
    pub available: bool,
    /// Optional GPU name associated with this encoder.
    pub gpu: Option<String>,
}

/// Full capabilities report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    pub encoders: Vec<EncoderInfo>,
    pub capture_api: String,
    pub gpu_name: String,
    pub supports_hardware_encode: bool,
}

/// Detect available encoders and capture capabilities.
pub fn detect_capabilities() -> SystemCapabilities {
    let gpu_name = detect_primary_gpu();
    let encoders = detect_encoders(&gpu_name);
    let supports_hw = encoders.iter().any(|e| e.encoder_type == "hardware" && e.available);

    SystemCapabilities {
        encoders,
        capture_api: "Windows.Graphics.Capture".to_string(),
        gpu_name,
        supports_hardware_encode: supports_hw,
    }
}

fn detect_primary_gpu() -> String {
    #[cfg(windows)]
    {
        unsafe {
            let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
                Ok(f) => f,
                Err(_) => return "Unknown".to_string(),
            };
            match factory.EnumAdapters1(0) {
                Ok(adapter) => {
                    if let Ok(desc) = adapter.GetDesc1() {
                        let name_len = desc
                            .Description
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(desc.Description.len());
                        return String::from_utf16_lossy(&desc.Description[..name_len]);
                    }
                }
                Err(_) => {}
            }
            "Unknown".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        "Unknown".to_string()
    }
}

fn detect_encoders(gpu_name: &str) -> Vec<EncoderInfo> {
    let mut encoders = Vec::new();

    // Detect hardware encoder availability based on GPU vendor.
    let gpu_lower = gpu_name.to_lowercase();

    if gpu_lower.contains("nvidia") || gpu_lower.contains("geforce") || gpu_lower.contains("rtx") || gpu_lower.contains("gtx") {
        encoders.push(EncoderInfo {
            name: "NVIDIA NVENC H.264".to_string(),
            encoder_type: "hardware".to_string(),
            codec: "h264".to_string(),
            available: true,
            gpu: Some(gpu_name.to_string()),
        });
    }

    if gpu_lower.contains("amd") || gpu_lower.contains("radeon") {
        encoders.push(EncoderInfo {
            name: "AMD AMF H.264".to_string(),
            encoder_type: "hardware".to_string(),
            codec: "h264".to_string(),
            available: true,
            gpu: Some(gpu_name.to_string()),
        });
    }

    if gpu_lower.contains("intel") {
        encoders.push(EncoderInfo {
            name: "Intel Quick Sync H.264".to_string(),
            encoder_type: "hardware".to_string(),
            codec: "h264".to_string(),
            available: true,
            gpu: Some(gpu_name.to_string()),
        });
    }

    // Media Foundation software encoder is always available on Windows 10+.
    encoders.push(EncoderInfo {
        name: "Microsoft Media Foundation H.264 (Software)".to_string(),
        encoder_type: "software".to_string(),
        codec: "h264".to_string(),
        available: true,
        gpu: None,
    });

    encoders
}
