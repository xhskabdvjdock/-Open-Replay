use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use parking_lot::Mutex;
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    pub clip_id: String,
    pub input_path: String,
    pub output_path: String,
    pub file_name: String,
    pub format: String,
    pub codec: String,
    pub quality: String,
    pub resolution: String,
    pub fps: u32,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub percentage: f32,
    pub current_time_ms: i64,
    pub total_time_ms: i64,
    pub status: String,
    pub error_message: Option<String>,
}

pub struct ExportManager {
    progress: Arc<Mutex<ExportProgress>>,
    cancel_flag: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
}

impl ExportManager {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(ExportProgress {
                percentage: 0.0,
                current_time_ms: 0,
                total_time_ms: 0,
                status: "idle".into(),
                error_message: None,
            })),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, config: ExportConfig) -> Result<()> {
        if self.is_running.load(Ordering::SeqCst) {
            anyhow::bail!("An export is already running");
        }

        let output_full = PathBuf::from(&config.output_path).join(&config.file_name);
        if output_full.exists() {
            anyhow::bail!("Output file already exists: {:?}", output_full);
        }
        if let Some(parent) = output_full.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.cancel_flag.store(false, Ordering::SeqCst);
        self.is_running.store(true, Ordering::SeqCst);

        {
            let mut p = self.progress.lock();
            *p = ExportProgress {
                percentage: 0.0,
                current_time_ms: 0,
                total_time_ms: config.end_ms - config.start_ms,
                status: "exporting".into(),
                error_message: None,
            };
        }

        let progress = Arc::clone(&self.progress);
        let cancel = Arc::clone(&self.cancel_flag);
        let is_running = Arc::clone(&self.is_running);

        std::thread::Builder::new()
            .name("export".into())
            .spawn(move || {
                let result = run_export(&config, &progress, &cancel);
                is_running.store(false, Ordering::SeqCst);
                let mut p = progress.lock();
                match result {
                    Ok(()) => {
                        if cancel.load(Ordering::SeqCst) {
                            p.status = "cancelled".into();
                            p.percentage = 0.0;
                        } else {
                            p.status = "complete".into();
                            p.percentage = 100.0;
                        }
                    }
                    Err(e) => {
                        error!("Export failed: {}", e);
                        p.status = "error".into();
                        p.error_message = Some(e.to_string());
                    }
                }
            })?;

        Ok(())
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().clone()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

impl Default for ExportManager {
    fn default() -> Self { Self::new() }
}

fn run_export(
    config: &ExportConfig,
    progress: &Arc<Mutex<ExportProgress>>,
    cancel: &Arc<AtomicBool>,
) -> Result<()> {
    let input = PathBuf::from(&config.input_path);
    let output = PathBuf::from(&config.output_path).join(&config.file_name);

    if !input.exists() {
        anyhow::bail!("Input file not found: {:?}", input);
    }

    let duration_ms = config.end_ms - config.start_ms;
    if duration_ms <= 0 {
        anyhow::bail!("Invalid trim range");
    }

    let needs_reencode = config.start_ms > 0 || config.end_ms > 0;

    if !needs_reencode {
        info!("Export: direct copy {:?} -> {:?}", input, output);
        std::fs::copy(&input, &output)?;
    } else {
        info!("Export: trim/transcode {:?} -> {:?}", input, output);
        export_with_trim(&input, &output, config, progress, cancel)?;
    }

    if !output.exists() || std::fs::metadata(&output)?.len() == 0 {
        anyhow::bail!("Export produced empty or missing file");
    }

    info!("Export complete: {:?}", output);
    Ok(())
}

fn export_with_trim(
    input: &PathBuf,
    output: &PathBuf,
    config: &ExportConfig,
    progress: &Arc<Mutex<ExportProgress>>,
    cancel: &Arc<AtomicBool>,
) -> Result<()> {
    #[cfg(windows)]
    {
        use windows::Win32::Media::MediaFoundation::*;
        use windows::core::HSTRING;

        unsafe {
            MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET)?;

            let input_url = HSTRING::from(input.to_string_lossy().as_ref());
            let output_url = HSTRING::from(output.to_string_lossy().as_ref());

            // windows 0.62: MFCreateSourceReaderFromURL returns IMFSourceReader directly (2 args)
            let reader: IMFSourceReader = MFCreateSourceReaderFromURL(&input_url, None::<&IMFAttributes>)?;

            reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
            reader.SetStreamSelection(0, true)?;

            // MFCreateMediaType returns IMFMediaType directly in 0.62
            let decode_type: IMFMediaType = MFCreateMediaType()?;
            decode_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            decode_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
            reader.SetCurrentMediaType(0, None, &decode_type)?;

            let native_type = reader.GetCurrentMediaType(0)?;
            let frame_size = native_type.GetUINT64(&MF_MT_FRAME_SIZE)?;
            let width = (frame_size >> 32) as u32;
            let height = (frame_size & 0xFFFF_FFFF) as u32;
            let frame_rate_val = native_type.GetUINT64(&MF_MT_FRAME_RATE).unwrap_or((30u64 << 32) | 1);
            let fps_num = (frame_rate_val >> 32) as f64;
            let fps_den = (frame_rate_val & 0xFFFF_FFFF) as f64;

            // Sink writer - MFCreateAttributes still uses out-param in 0.62
            let mut attrs_opt: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attrs_opt, 2)?;
            let attrs = attrs_opt.ok_or_else(|| anyhow::anyhow!("MFCreateAttributes failed"))?;
            attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;

            // MFCreateSinkWriterFromURL returns IMFSinkWriter directly (3 args) in 0.62
            let writer: IMFSinkWriter = MFCreateSinkWriterFromURL(&output_url, None::<&IMFByteStream>, &attrs)?;

            let video_out: IMFMediaType = MFCreateMediaType()?;
            video_out.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            video_out.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
            video_out.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            let fs = ((width as u64) << 32) | (height as u64);
            video_out.SetUINT64(&MF_MT_FRAME_SIZE, fs)?;
            let fr = ((fps_num as u64) << 32) | (fps_den as u64);
            video_out.SetUINT64(&MF_MT_FRAME_RATE, fr)?;
            video_out.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1u64)?;
            video_out.SetUINT32(&MF_MT_AVG_BITRATE, 8_000_000)?;
            let stream_idx = writer.AddStream(&video_out)?;

            let video_in: IMFMediaType = MFCreateMediaType()?;
            video_in.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            video_in.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
            video_in.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
            video_in.SetUINT64(&MF_MT_FRAME_SIZE, fs)?;
            video_in.SetUINT64(&MF_MT_FRAME_RATE, fr)?;
            video_in.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1u64)?;
            video_in.SetUINT32(&MF_MT_DEFAULT_STRIDE, width * 4)?;
            writer.SetInputMediaType(stream_idx, &video_in, None::<&IMFAttributes>)?;
            writer.BeginWriting()?;

            let duration_ms = config.end_ms - config.start_ms;
            let mut written_ms: i64 = 0;

            loop {
                if cancel.load(Ordering::SeqCst) { break; }

                let mut flags: u32 = 0;
                let mut timestamp: i64 = 0;
                let mut actual_stream: u32 = 0;
                let mut sample: Option<IMFSample> = None;

                if reader.ReadSample(
                    0, 0,
                    Some(&mut actual_stream),
                    Some(&mut flags),
                    Some(&mut timestamp),
                    Some(&mut sample),
                ).is_err() {
                    break;
                }

                if flags & 4 != 0 { break; } // MF_SOURCE_READERF_ENDOFSTREAM

                let sample = match sample { Some(s) => s, None => continue };
                let ts_ms = timestamp / 10_000;

                if config.end_ms > 0 && ts_ms > config.end_ms { break; }
                if ts_ms < config.start_ms { continue; }

                let adjusted_ts_ms = ts_ms - config.start_ms;
                sample.SetSampleTime(adjusted_ts_ms * 10_000)?;
                writer.WriteSample(stream_idx, &sample)?;
                written_ms = adjusted_ts_ms;

                let pct = (written_ms as f32 / duration_ms as f32 * 100.0).clamp(0.0, 99.0);
                let mut p = progress.lock();
                p.percentage = pct;
                p.current_time_ms = written_ms;
            }

            writer.Finalize()?;
            MFShutdown()?;
        }
    }

    #[cfg(not(windows))]
    {
        std::fs::copy(input, output)?;
    }

    Ok(())
}
