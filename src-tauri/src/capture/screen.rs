/// Screen capture using Windows Graphics Capture API.
use anyhow::Result;
use tracing::info;
use crate::capture::monitor::{MonitorInfo, enumerate_monitors};

pub struct ScreenCapture {
    monitor_info: MonitorInfo,
    #[cfg(windows)]
    inner: windows_impl::WgcCapture,
    #[cfg(not(windows))]
    _frame_count: u64,
}

impl ScreenCapture {
    pub fn new(monitor_id: &str) -> Result<Self> {
        let monitors = enumerate_monitors();
        let monitor = if monitor_id == "primary" {
            monitors.into_iter().find(|m| m.is_primary)
                .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?
        } else {
            monitors.into_iter().find(|m| m.id == monitor_id)
                .ok_or_else(|| anyhow::anyhow!("Monitor not found: {}", monitor_id))?
        };

        info!("Initializing screen capture: {} ({}x{})", monitor.name, monitor.width, monitor.height);

        #[cfg(windows)]
        {
            let inner = windows_impl::WgcCapture::new(&monitor)?;
            return Ok(Self { monitor_info: monitor, inner });
        }
        #[cfg(not(windows))]
        Ok(Self { monitor_info: monitor, _frame_count: 0 })
    }

    pub fn capture_frame(&mut self) -> Result<Option<Vec<u8>>> {
        #[cfg(windows)]
        { return self.inner.capture_frame(); }
        #[cfg(not(windows))]
        {
            self._frame_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(16));
            Ok(Some(vec![0u8; 16]))
        }
    }

    pub fn monitor_info(&self) -> &MonitorInfo { &self.monitor_info }
}

#[cfg(windows)]
mod windows_impl {
    use anyhow::Result;
    use tracing::info;
    use std::sync::Arc;
    use parking_lot::Mutex;

    use windows::Win32::Graphics::Direct3D11::*;
    use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
    use windows::Win32::Graphics::Dxgi::*;
    use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
    use windows::Graphics::Capture::*;
    use windows::Graphics::DirectX::DirectXPixelFormat;
    use windows::Foundation::TypedEventHandler;
    use windows::core::Interface;

    use crate::capture::monitor::MonitorInfo;

    pub struct WgcCapture {
        _item: GraphicsCaptureItem,
        _session: GraphicsCaptureSession,
        frame_pool: Direct3D11CaptureFramePool,
        d3d_device: ID3D11Device,
        d3d_context: ID3D11DeviceContext,
        pending_frame: Arc<Mutex<Option<Direct3D11CaptureFrame>>>,
        width: u32,
        height: u32,
    }

    impl WgcCapture {
        pub fn new(monitor: &MonitorInfo) -> Result<Self> {
            let (d3d_device, d3d_context) = create_d3d11_device()?;
            let winrt_device = create_winrt_device(&d3d_device)?;

            let item = create_capture_item(monitor)?;
            let item_size = item.Size()?;
            let width = item_size.Width as u32;
            let height = item_size.Height as u32;

            let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
                &winrt_device,
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                2,
                item_size,
            )?;

            let pending_frame: Arc<Mutex<Option<Direct3D11CaptureFrame>>> = Arc::new(Mutex::new(None));
            let pending_clone = Arc::clone(&pending_frame);

            // TypedEventHandler in windows 0.62: Ref<T> is Option-like, use as_ref()
            frame_pool.FrameArrived(&TypedEventHandler::new(
                move |pool: windows_core::Ref<Direct3D11CaptureFramePool>, _: windows_core::Ref<windows::core::IInspectable>| {
                    if let Some(p) = pool.as_ref() {
                        if let Ok(frame) = p.TryGetNextFrame() {
                            *pending_clone.lock() = Some(frame);
                        }
                    }
                    Ok(())
                },
            ))?;

            let session = frame_pool.CreateCaptureSession(&item)?;
            let _ = session.SetIsCursorCaptureEnabled(true);
            session.StartCapture()?;

            info!("WGC capture started {}x{}", width, height);
            Ok(Self { _item: item, _session: session, frame_pool, d3d_device, d3d_context, pending_frame, width, height })
        }

        pub fn capture_frame(&mut self) -> Result<Option<Vec<u8>>> {
            let frame = { self.pending_frame.lock().take() };
            let frame = match frame { Some(f) => f, None => return Ok(None) };

            let surface = frame.Surface()?;
            let texture: ID3D11Texture2D = get_texture_from_surface(&surface)?;

            let mut desc = D3D11_TEXTURE2D_DESC::default();
            unsafe { texture.GetDesc(&mut desc); }

            // In windows 0.62, BindFlags/CPUAccessFlags/MiscFlags are u32
            let staging_desc = D3D11_TEXTURE2D_DESC {
                Width: desc.Width,
                Height: desc.Height,
                MipLevels: 1,
                ArraySize: 1,
                Format: desc.Format,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0u32,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0u32,
            };

            // CreateTexture2D in 0.62: (desc, init_data, out_texture) - out_texture is Option<*mut Option<T>>
            let mut staging_opt: Option<ID3D11Texture2D> = None;
            unsafe { self.d3d_device.CreateTexture2D(&staging_desc, None, Some(&mut staging_opt))? };
            let staging = staging_opt.ok_or_else(|| anyhow::anyhow!("CreateTexture2D returned None"))?;
            unsafe { self.d3d_context.CopyResource(&staging, &texture); }

            // Map in 0.62: (resource, subresource, map_type, map_flags, mapped_resource)
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            unsafe { self.d3d_context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))? };

            let row_pitch = mapped.RowPitch as usize;
            let h = desc.Height as usize;
            let w = desc.Width as usize;
            // Keep compressed frames in the replay ring. Retaining the raw
            // BGRA image at 1080p costs ~8 MB per frame (nearly 15 GB for
            // 30 seconds at 60 FPS).
            let mut rgb = Vec::with_capacity(w * h * 3);
            let src = mapped.pData as *const u8;
            for row in 0..h {
                let row_data = unsafe { std::slice::from_raw_parts(src.add(row * row_pitch), w * 4) };
                for pixel in row_data.chunks_exact(4) {
                    // WGC supplies BGRA; JPEG expects RGB.
                    rgb.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
                }
            }

            unsafe { self.d3d_context.Unmap(&staging, 0); }
            let mut jpeg = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 78)
                .encode(&rgb, desc.Width, desc.Height, image::ExtendedColorType::Rgb8)?;
            Ok(Some(jpeg))
        }
    }

    fn create_d3d11_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
        unsafe {
            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            // In windows 0.62, software param is HMODULE (not Option)
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                windows::Win32::Foundation::HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
            Ok((device.unwrap(), context.unwrap()))
        }
    }

    fn create_winrt_device(d3d: &ID3D11Device) -> Result<windows::Graphics::DirectX::Direct3D11::IDirect3DDevice> {
        use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
        let dxgi: IDXGIDevice = d3d.cast()?;
        let insp = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? };
        Ok(insp.cast()?)
    }

    fn create_capture_item(monitor: &MonitorInfo) -> Result<GraphicsCaptureItem> {
        use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
        let interop: IGraphicsCaptureItemInterop =
            windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        let hmonitor = find_hmonitor(&monitor.name, monitor.is_primary)
            .ok_or_else(|| anyhow::anyhow!("Cannot find HMONITOR for '{}'", monitor.name))?;
        let item: GraphicsCaptureItem = unsafe { interop.CreateForMonitor(hmonitor)? };
        Ok(item)
    }

    fn find_hmonitor(target_name: &str, want_primary: bool) -> Option<windows::Win32::Graphics::Gdi::HMONITOR> {
        use windows::Win32::Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFO, HDC, HMONITOR,
        };
        use windows::Win32::Foundation::{LPARAM, RECT};

        struct State { name: String, primary: bool, result: Option<HMONITOR> }
        let mut state = State { name: target_name.to_string(), primary: want_primary, result: None };

        // In windows 0.62, szDevice is [u16; 32]
        unsafe extern "system" fn cb(hm: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> windows_core::BOOL {
            let s = &mut *(lp.0 as *mut State);
            let mut mi = MONITORINFOEXW {
                monitorInfo: MONITORINFO { cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32, ..Default::default() },
                szDevice: [0u16; 32],
            };
            if GetMonitorInfoW(hm, &mut mi.monitorInfo).as_bool() {
                let is_primary = (mi.monitorInfo.dwFlags & 1) != 0;
                let name_end = mi.szDevice.iter().position(|&c| c == 0).unwrap_or(32);
                let name = String::from_utf16_lossy(&mi.szDevice[..name_end]);
                let name = name.trim_matches('\0');
                if (s.primary && is_primary) || name == s.name {
                    s.result = Some(hm);
                    return windows_core::BOOL(0);
                }
            }
            windows_core::BOOL(1)
        }

        unsafe {
            // In windows 0.62, hdc param is Option<HDC>
            let _ = EnumDisplayMonitors(Some(HDC::default()), None, Some(cb), LPARAM(&mut state as *mut _ as isize));
        }
        state.result
    }

    fn get_texture_from_surface(surface: &windows::Graphics::DirectX::Direct3D11::IDirect3DSurface) -> Result<ID3D11Texture2D> {
        use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
        let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };
        Ok(texture)
    }
}
