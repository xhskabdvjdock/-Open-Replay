/// MP4 muxer using Windows Media Foundation Sink Writer.
use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

pub struct Mp4Muxer {
    output_path: PathBuf,
    #[cfg(windows)]
    inner: Option<mf::SinkWriter>,
}

impl Mp4Muxer {
    pub fn new(output_path: &PathBuf, width: u32, height: u32, fps: f64) -> Result<Self> {
        #[cfg(windows)]
        {
            let inner = mf::SinkWriter::new(output_path, width, height, fps)?;
            return Ok(Self { output_path: output_path.clone(), inner: Some(inner) });
        }
        #[cfg(not(windows))]
        Ok(Self { output_path: output_path.clone() })
    }

    pub fn write_video_packet(&mut self, data: &[u8], pts_ms: i64, duration_ms: i64, is_keyframe: bool) -> Result<()> {
        #[cfg(windows)]
        if let Some(inner) = &mut self.inner {
            inner.write_video(data, pts_ms, duration_ms, is_keyframe)?;
        }
        Ok(())
    }

    pub fn finalize(mut self) -> Result<()> {
        #[cfg(windows)]
        if let Some(inner) = self.inner.take() {
            inner.finalize()?;
        }
        info!("Muxer finalized: {:?}", self.output_path);
        Ok(())
    }
}

#[cfg(windows)]
mod mf {
    use anyhow::Result;
    use std::path::PathBuf;
    use windows::Win32::Media::MediaFoundation::*;
    use windows::core::HSTRING;
    use tracing::info;

    pub struct SinkWriter {
        writer: IMFSinkWriter,
        video_idx: u32,
    }

    impl SinkWriter {
        pub fn new(path: &PathBuf, width: u32, height: u32, fps: f64) -> Result<Self> {
            unsafe {
                MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET)?;
                let url = HSTRING::from(path.to_string_lossy().as_ref());

                // windows 0.62: MFCreateAttributes uses out-param, but most others return directly
                let mut attrs_opt: Option<IMFAttributes> = None;
                MFCreateAttributes(&mut attrs_opt, 2)?;
                let attrs = attrs_opt.ok_or_else(|| anyhow::anyhow!("MFCreateAttributes failed"))?;
                attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;

                // MFCreateSinkWriterFromURL returns IMFSinkWriter directly in 0.62
                let writer: IMFSinkWriter = MFCreateSinkWriterFromURL(&url, None::<&IMFByteStream>, &attrs)?;

                // MFCreateMediaType returns IMFMediaType directly in 0.62
                let out_type: IMFMediaType = MFCreateMediaType()?;
                out_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                out_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_H264)?;
                out_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
                let fs = ((width as u64) << 32) | height as u64;
                out_type.SetUINT64(&MF_MT_FRAME_SIZE, fs)?;
                let fps_n = (fps * 1000.0) as u64;
                out_type.SetUINT64(&MF_MT_FRAME_RATE, (fps_n << 32) | 1000)?;
                out_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)?;
                out_type.SetUINT32(&MF_MT_AVG_BITRATE, 8_000_000)?;

                let video_idx = writer.AddStream(&out_type)?;

                let in_type: IMFMediaType = MFCreateMediaType()?;
                in_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
                in_type.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_ARGB32)?;
                in_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
                in_type.SetUINT64(&MF_MT_FRAME_SIZE, fs)?;
                in_type.SetUINT64(&MF_MT_FRAME_RATE, (fps_n << 32) | 1000)?;
                in_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1u64 << 32) | 1)?;
                in_type.SetUINT32(&MF_MT_DEFAULT_STRIDE, width * 4)?;
                writer.SetInputMediaType(video_idx, &in_type, None::<&IMFAttributes>)?;
                writer.BeginWriting()?;

                info!("MF Sink Writer: {}x{} @ {}fps", width, height, fps);
                Ok(Self { writer, video_idx })
            }
        }

        pub fn write_video(&mut self, data: &[u8], pts_ms: i64, dur_ms: i64, keyframe: bool) -> Result<()> {
            unsafe {
                // MFCreateMemoryBuffer, MFCreateSample return directly in 0.62
                let buf: IMFMediaBuffer = MFCreateMemoryBuffer(data.len() as u32)?;
                let mut ptr: *mut u8 = std::ptr::null_mut();
                buf.Lock(&mut ptr, None, None)?;
                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
                buf.Unlock()?;
                buf.SetCurrentLength(data.len() as u32)?;

                let sample: IMFSample = MFCreateSample()?;
                sample.AddBuffer(&buf)?;
                sample.SetSampleTime(pts_ms * 10_000)?;
                sample.SetSampleDuration(dur_ms * 10_000)?;
                if keyframe { sample.SetUINT32(&MFSampleExtension_CleanPoint, 1)?; }
                self.writer.WriteSample(self.video_idx, &sample)?;
            }
            Ok(())
        }

        pub fn finalize(self) -> Result<()> {
            unsafe {
                self.writer.Finalize()?;
                MFShutdown()?;
            }
            Ok(())
        }
    }
}
