use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderInfo {
    pub name: String,
    pub encoder_type: String,
    pub codec: String,
    pub is_available: bool,
    pub notes: String,
}

pub fn enumerate_encoders() -> Vec<EncoderInfo> {
    let mut encoders = Vec::new();

    encoders.push(EncoderInfo {
        name: "Media Foundation Software H.264".into(),
        encoder_type: "software".into(),
        codec: "h264".into(),
        is_available: true,
        notes: "Software encoder - always available on Windows 10+".into(),
    });

    #[cfg(windows)]
    {
        if let Ok(hw) = detect_hardware_encoders() {
            encoders.extend(hw);
        }
    }

    encoders
}

#[cfg(windows)]
fn detect_hardware_encoders() -> Result<Vec<EncoderInfo>> {
    use windows::Win32::Media::MediaFoundation::*;

    let mut result = Vec::new();

    unsafe {
        MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET)?;

        let mut count: u32 = 0;
        let mut activates: *mut Option<IMFActivate> = std::ptr::null_mut();

        if MFTEnumEx(
            MFT_CATEGORY_VIDEO_ENCODER,
            MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER,
            None,
            None,
            &mut activates,
            &mut count,
        ).is_ok() && count > 0 {
            let activates_slice = std::slice::from_raw_parts(activates, count as usize);
            for activate in activates_slice {
                if let Some(activate) = activate {
                    let name = get_mft_friendly_name(activate).unwrap_or_else(|_| "Unknown HW Encoder".into());
                    let notes = if name.to_lowercase().contains("nvidia") || name.to_lowercase().contains("nvenc") {
                        "NVIDIA NVENC hardware encoder"
                    } else if name.to_lowercase().contains("amd") || name.to_lowercase().contains("amf") {
                        "AMD AMF hardware encoder"
                    } else if name.to_lowercase().contains("intel") || name.to_lowercase().contains("qsv") {
                        "Intel QuickSync hardware encoder"
                    } else {
                        "Hardware encoder"
                    };

                    result.push(EncoderInfo {
                        name,
                        encoder_type: "hardware".into(),
                        codec: "h264".into(),
                        is_available: true,
                        notes: notes.into(),
                    });
                }
            }
            if !activates.is_null() {
                windows::Win32::System::Com::CoTaskMemFree(Some(activates as *mut _));
            }
        }

        let _ = MFShutdown();
    }

    Ok(result)
}

#[cfg(windows)]
fn get_mft_friendly_name(activate: &windows::Win32::Media::MediaFoundation::IMFActivate) -> Result<String> {
    use windows::Win32::Media::MediaFoundation::MFT_FRIENDLY_NAME_Attribute;
    let name = unsafe {
        let len = activate.GetStringLength(&MFT_FRIENDLY_NAME_Attribute)? as usize + 1;
        let mut buf = vec![0u16; len];
        let mut actual_len = 0u32;
        activate.GetString(&MFT_FRIENDLY_NAME_Attribute, &mut buf, Some(&mut actual_len))?;
        String::from_utf16_lossy(&buf[..actual_len as usize])
    };
    Ok(name)
}
