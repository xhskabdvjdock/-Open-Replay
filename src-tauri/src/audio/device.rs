use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: String, // "input" | "output"
    pub is_default: bool,
}

/// Enumerate audio devices via WASAPI
pub fn enumerate_audio_devices() -> Result<Vec<AudioDevice>> {
    #[cfg(windows)]
    {
        enumerate_audio_devices_windows()
    }
    #[cfg(not(windows))]
    {
        Ok(vec![
            AudioDevice {
                id: "default-output".into(),
                name: "Default Output".into(),
                device_type: "output".into(),
                is_default: true,
            },
            AudioDevice {
                id: "default-input".into(),
                name: "Default Input".into(),
                device_type: "input".into(),
                is_default: true,
            },
        ])
    }
}

#[cfg(windows)]
fn enumerate_audio_devices_windows() -> Result<Vec<AudioDevice>> {
    use windows::Win32::Media::Audio::{
        IMMDeviceEnumerator, MMDeviceEnumerator, eAll, eCapture, eRender,
        DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, CoInitializeEx, COINIT_MULTITHREADED};
    use windows::core::Interface;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;

        let mut devices = Vec::new();

        let collection = enumerator.EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;
        let count = collection.GetCount()?;

        let default_render = enumerator.GetDefaultAudioEndpoint(eRender, windows::Win32::Media::Audio::eConsole).ok();
        let default_capture = enumerator.GetDefaultAudioEndpoint(eCapture, windows::Win32::Media::Audio::eConsole).ok();

        let default_render_id = default_render.as_ref().and_then(|d| get_device_id(d).ok());
        let default_capture_id = default_capture.as_ref().and_then(|d| get_device_id(d).ok());

        for i in 0..count {
            let device = collection.Item(i)?;
            let id = match get_device_id(&device) {
                Ok(id) => id,
                Err(_) => continue,
            };
            let name = get_device_name(&device).unwrap_or_else(|_| format!("Device {}", i));

            let endpoint: windows::Win32::Media::Audio::IMMEndpoint = device.cast()?;
            let dataflow = endpoint.GetDataFlow()?;
            let device_type = if dataflow == eRender { "output" } else { "input" }.to_string();
            let is_default = if dataflow == eRender {
                default_render_id.as_deref() == Some(&id)
            } else {
                default_capture_id.as_deref() == Some(&id)
            };

            devices.push(AudioDevice { id, name, device_type, is_default });
        }

        Ok(devices)
    }
}

#[cfg(windows)]
fn get_device_id(device: &windows::Win32::Media::Audio::IMMDevice) -> Result<String> {
    unsafe {
        let id_pwstr = device.GetId()?;
        let id = id_pwstr.to_string()?;
        windows::Win32::System::Com::CoTaskMemFree(Some(id_pwstr.0 as *mut _));
        Ok(id)
    }
}

#[cfg(windows)]
fn get_device_name(device: &windows::Win32::Media::Audio::IMMDevice) -> Result<String> {
    // Simplified: use device ID as name when property store is unavailable
    // Full implementation needs Win32_System_Variant feature for PROPVARIANT
    match get_device_id(device) {
        Ok(id) => {
            // Strip the {GUID}.X suffix to get a cleaner name
            let clean = id.trim_matches('{').trim_matches('}');
            Ok(format!("Audio Device ({})", &clean[..clean.len().min(8)]))
        }
        Err(e) => Err(e),
    }
}
