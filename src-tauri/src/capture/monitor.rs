use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f32,
    pub is_primary: bool,
    pub refresh_rate: u32,
}

pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    #[cfg(windows)] { enumerate_monitors_windows() }
    #[cfg(not(windows))] {
        vec![MonitorInfo {
            id: "primary".into(), name: "Primary Monitor".into(),
            width: 1920, height: 1080, x: 0, y: 0,
            scale_factor: 1.0, is_primary: true, refresh_rate: 60,
        }]
    }
}

#[cfg(windows)]
fn enumerate_monitors_windows() -> Vec<MonitorInfo> {
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFO, HDC, HMONITOR,
    };
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

    static LIST: Mutex<Vec<MonitorInfo>> = Mutex::new(Vec::new());
    LIST.lock().unwrap().clear();

    unsafe extern "system" fn cb(
        hm: HMONITOR, _: HDC, _: *mut RECT, _: LPARAM,
    ) -> windows_core::BOOL {
        // In windows 0.62, MONITORINFOEXW::szDevice is [u16; 32]
        let mut mi = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            szDevice: [0u16; 32],
        };
        if GetMonitorInfoW(hm, &mut mi.monitorInfo).as_bool() {
            let r = mi.monitorInfo.rcMonitor;
            let is_primary = (mi.monitorInfo.dwFlags & 1) != 0;
            let name_end = mi.szDevice.iter().position(|&c| c == 0).unwrap_or(32);
            let name = String::from_utf16_lossy(&mi.szDevice[..name_end])
                .trim_matches('\0').to_string();

            let mut dpi_x: u32 = 96;
            let mut dpi_y: u32 = 96;
            let _ = GetDpiForMonitor(hm, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);

            let monitor = MonitorInfo {
                id: format!("monitor_{}", hm.0 as usize),
                name,
                width: (r.right - r.left) as u32,
                height: (r.bottom - r.top) as u32,
                x: r.left,
                y: r.top,
                scale_factor: dpi_x as f32 / 96.0,
                is_primary,
                refresh_rate: 60,
            };

            if let Ok(mut lock) = LIST.lock() {
                lock.push(monitor);
            }
        }
        windows_core::BOOL(1)
    }

    // In windows 0.62, hdc param is Option<HDC>
    unsafe { let _ = EnumDisplayMonitors(Some(HDC::default()), None, Some(cb), LPARAM(0)); }
    LIST.lock().unwrap().clone()
}
