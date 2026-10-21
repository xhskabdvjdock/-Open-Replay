use tracing::info;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use std::path::Path;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Initialize structured logging with file rotation
pub fn init_logging(log_dir: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = rolling::daily(log_dir, "open-replay.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // Keep guard alive for the duration of the program
    Box::leak(Box::new(guard));

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true),
        )
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Logging initialized in {:?}", log_dir);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub gpu_name: String,
    pub total_ram_mb: u64,
    pub cpu_name: String,
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: get_os_version(),
        gpu_name: get_gpu_name(),
        total_ram_mb: get_ram_mb(),
        cpu_name: get_cpu_name(),
    }
}

fn get_os_version() -> String {
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::GetVersionExW;
        use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
        unsafe {
            let mut info = OSVERSIONINFOW::default();
            info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
            let _ = GetVersionExW(&mut info);
            format!("Windows {}.{} Build {}",
                info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber)
        }
    }
    #[cfg(not(windows))]
    { std::env::consts::OS.to_string() }
}

fn get_gpu_name() -> String {
    #[cfg(windows)]
    {
        use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                if let Ok(adapter) = factory.EnumAdapters1(0) {
                    if let Ok(desc) = adapter.GetDesc1() {
                        let name_end = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
                        return String::from_utf16_lossy(&desc.Description[..name_end]);
                    }
                }
            }
        }
        "Unknown GPU".into()
    }
    #[cfg(not(windows))]
    { "Unknown GPU".into() }
}

fn get_ram_mb() -> u64 {
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        unsafe {
            let mut status = MEMORYSTATUSEX::default();
            status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut status).is_ok() {
                return status.ullTotalPhys / 1024 / 1024;
            }
        }
        0
    }
    #[cfg(not(windows))]
    { 0 }
}

fn get_cpu_name() -> String {
    format!("{} processor", std::env::consts::ARCH)
}

pub fn get_storage_info(path: &Path) -> (u64, u64) {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            let mut free: u64 = 0;
            let mut total: u64 = 0;
            if windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                windows::core::PCWSTR(wide.as_ptr()),
                Some(&mut free),
                Some(&mut total),
                None,
            ).is_ok() {
                return (total, free);
            }
        }
        (0, 0)
    }
    #[cfg(not(windows))]
    { (0, 0) }
}
