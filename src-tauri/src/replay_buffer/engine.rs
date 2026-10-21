use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use tracing::{info, warn, error};

use super::buffer::{ReplayBuffer, EncodedPacket, StreamType};
use crate::settings::config::Settings;
use crate::capture::monitor::MonitorInfo;

/// Current status of the replay engine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayStatus {
    pub is_recording: bool,
    pub buffer_duration_ms: i64,
    pub buffer_fill_percent: f32,
    pub actual_fps: f32,
    pub target_fps: u32,
    pub monitor_name: String,
    pub resolution: String,
    pub has_audio: bool,
    pub has_microphone: bool,
    pub error: Option<String>,
}

/// The core replay engine.
/// Manages capture threads, the replay buffer, and saving clips.
pub struct ReplayEngine {
    buffer: Arc<ReplayBuffer>,
    is_recording: Arc<AtomicBool>,
    settings: Arc<Mutex<Settings>>,
    status: Arc<Mutex<ReplayStatus>>,
    stop_tx: Option<std::sync::mpsc::Sender<()>>,
    frame_counter: Arc<Mutex<FrameCounter>>,
}

struct FrameCounter {
    count: u64,
    window_start: Instant,
    fps: f32,
}

impl FrameCounter {
    fn new() -> Self {
        Self { count: 0, window_start: Instant::now(), fps: 0.0 }
    }

    fn tick(&mut self) {
        self.count += 1;
        let elapsed = self.window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.count as f32 / elapsed.as_secs_f32();
            self.count = 0;
            self.window_start = Instant::now();
        }
    }
}

impl ReplayEngine {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        let initial_duration = {
            let s = settings.lock();
            s.replay.buffer_duration_sec as i64 * 1000
        };
        let buffer = Arc::new(ReplayBuffer::new(initial_duration));
        let status = Arc::new(Mutex::new(ReplayStatus {
            is_recording: false,
            buffer_duration_ms: initial_duration,
            buffer_fill_percent: 0.0,
            actual_fps: 0.0,
            target_fps: 60,
            monitor_name: String::from("None"),
            resolution: String::from("-"),
            has_audio: false,
            has_microphone: false,
            error: None,
        }));
        Self {
            buffer,
            is_recording: Arc::new(AtomicBool::new(false)),
            settings,
            status,
            stop_tx: None,
            frame_counter: Arc::new(Mutex::new(FrameCounter::new())),
        }
    }

    /// Start the capture loop in a background thread
    pub fn start(&mut self) -> Result<()> {
        if self.is_recording.load(Ordering::SeqCst) {
            return Ok(()); // Already recording
        }

        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        self.stop_tx = Some(stop_tx);

        let buffer = Arc::clone(&self.buffer);
        let is_recording = Arc::clone(&self.is_recording);
        let settings = Arc::clone(&self.settings);
        let status = Arc::clone(&self.status);
        let frame_counter = Arc::clone(&self.frame_counter);

        is_recording.store(true, Ordering::SeqCst);
        {
            let mut s = status.lock();
            s.is_recording = true;
            s.error = None;
        }

        std::thread::Builder::new()
            .name("replay-capture".into())
            .spawn(move || {
                info!("Replay capture thread started");
                if let Err(e) = capture_loop(
                    buffer,
                    is_recording.clone(),
                    settings,
                    status.clone(),
                    frame_counter,
                    stop_rx,
                ) {
                    error!("Capture loop error: {}", e);
                    is_recording.store(false, Ordering::SeqCst);
                    let mut s = status.lock();
                    s.is_recording = false;
                    s.error = Some(format!("Capture error: {}", e));
                }
                info!("Replay capture thread stopped");
            })?;

        info!("Replay engine started");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self.is_recording.store(false, Ordering::SeqCst);
        let mut s = self.status.lock();
        s.is_recording = false;
        info!("Replay engine stopped");
    }

    /// Extract the current replay buffer content and write it to a file.
    /// Returns the path of the saved file and its duration.
    pub fn save_replay(&self, output_path: &PathBuf) -> Result<(PathBuf, i64, i32, i32, f64, bool)> {
        let settings = self.settings.lock().clone();
        let duration_ms = settings.replay.buffer_duration_sec as i64 * 1000;
        let packets = self.buffer.extract_last(duration_ms);

        if packets.is_empty() {
            anyhow::bail!("No data in replay buffer");
        }

        let video_packets: Vec<_> = packets.iter().filter(|p| p.stream == StreamType::Video).collect();
        let audio_packets: Vec<_> = packets.iter().filter(|p| p.stream == StreamType::Audio).collect();
        let has_audio = !audio_packets.is_empty();

        // Calculate actual duration from packet timestamps
        let first_pts = packets.first().map(|p| p.pts_ms).unwrap_or(0);
        let last_pts = packets.last().map(|p| p.pts_ms).unwrap_or(0);
        let actual_duration_ms = last_pts - first_pts;

        if actual_duration_ms < 500 {
            anyhow::bail!("Buffer too short to save ({} ms)", actual_duration_ms);
        }

        let status = self.status.lock();
        let width = parse_resolution_width(&status.resolution);
        let height = parse_resolution_height(&status.resolution);
        let fps = settings.capture.fps as f64;
        drop(status);

        // Write to MP4 using Media Foundation Sink Writer (Windows-native)
        write_packets_to_mp4(
            output_path,
            &video_packets,
            &audio_packets,
            width,
            height,
            fps,
            settings.capture.fps,
        )?;

        info!("Replay saved: {:?} ({} ms)", output_path, actual_duration_ms);
        Ok((output_path.clone(), actual_duration_ms, width as i32, height as i32, fps, has_audio))
    }

    pub fn get_status(&self) -> ReplayStatus {
        let mut s = self.status.lock().clone();
        s.buffer_fill_percent = self.buffer.fill_fraction(s.buffer_duration_ms) * 100.0;
        s
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn update_settings(&self, new_settings: &Settings) {
        let duration_ms = new_settings.replay.buffer_duration_sec as i64 * 1000;
        self.buffer.set_max_duration_ms(duration_ms);
        let mut s = self.status.lock();
        s.buffer_duration_ms = duration_ms;
        s.target_fps = new_settings.capture.fps;
    }
}

/// The main capture loop running in a background thread.
/// On Windows, uses Windows Graphics Capture API.
fn capture_loop(
    buffer: Arc<ReplayBuffer>,
    is_recording: Arc<AtomicBool>,
    settings: Arc<Mutex<Settings>>,
    status: Arc<Mutex<ReplayStatus>>,
    frame_counter: Arc<Mutex<FrameCounter>>,
    stop_rx: std::sync::mpsc::Receiver<()>,
) -> Result<()> {
    #[cfg(windows)]
    {
        use crate::capture::screen::ScreenCapture;

        let (monitor_id, fps, has_audio, has_mic) = {
            let s = settings.lock();
            (
                s.capture.monitor_id.clone(),
                s.capture.fps,
                s.audio.system_audio_enabled,
                s.audio.microphone_enabled,
            )
        };

        {
            let mut s = status.lock();
            s.target_fps = fps;
            s.has_audio = has_audio;
            s.has_microphone = has_mic;
        }

        let frame_interval = Duration::from_secs_f64(1.0 / fps as f64);
        let mut capture = ScreenCapture::new(&monitor_id)?;

        // Update status with monitor info
        {
            let info = capture.monitor_info();
            let mut s = status.lock();
            s.monitor_name = info.name.clone();
            s.resolution = format!("{}x{}", info.width, info.height);
        }

        let start_time = Instant::now();
        let mut pts_ms: i64 = 0;
        loop {
            // Check stop signal (non-blocking)
            if stop_rx.try_recv().is_ok() || !is_recording.load(Ordering::SeqCst) {
                break;
            }

            let frame_start = Instant::now();

            match capture.capture_frame() {
                Ok(Some(frame_data)) => {
                    pts_ms = start_time.elapsed().as_millis() as i64;
                    let packet = EncodedPacket {
                        // ScreenCapture returns an independently decodable JPEG.
                        data: frame_data.into(),
                        pts_ms,
                        duration_ms: (1000 / fps as i64).max(1),
                        is_keyframe: true,
                        stream: StreamType::Video,
                    };
                    buffer.push(packet);
                    frame_counter.lock().tick();
                }
                Ok(None) => {
                    // No new frame yet, continue
                }
                Err(e) => {
                    warn!("Frame capture error: {}", e);
                    // Don't stop on transient errors, keep trying
                    std::thread::sleep(Duration::from_millis(100));
                }
            }

            // Update FPS in status
            {
                let fc = frame_counter.lock();
                let mut s = status.lock();
                s.actual_fps = fc.fps;
                s.buffer_fill_percent = buffer.fill_fraction(s.buffer_duration_ms) * 100.0;
            }

            // Sleep remaining frame time
            let elapsed = frame_start.elapsed();
            if elapsed < frame_interval {
                std::thread::sleep(frame_interval - elapsed);
            }
        }
    }

    #[cfg(not(windows))]
    {
        // Non-Windows stub: generate synthetic packets for testing
        let fps = settings.lock().capture.fps;
        let frame_interval = Duration::from_secs_f64(1.0 / fps as f64);
        let start_time = Instant::now();
        let mut frame_num: u64 = 0;

        loop {
            if stop_rx.try_recv().is_ok() || !is_recording.load(Ordering::SeqCst) {
                break;
            }
            let pts_ms = start_time.elapsed().as_millis() as i64;
            let is_keyframe = frame_num % (fps as u64 * 2) == 0;
            // Synthetic "frame" data
            let packet = EncodedPacket {
                data: vec![0u8; 1024].into(),
                pts_ms,
                duration_ms: 1000 / fps as i64,
                is_keyframe,
                stream: StreamType::Video,
            };
            buffer.push(packet);
            frame_counter.lock().tick();
            frame_num += 1;

            let mut s = status.lock();
            s.actual_fps = frame_counter.lock().fps;
            s.buffer_fill_percent = buffer.fill_fraction(s.buffer_duration_ms) * 100.0;
            drop(s);

            std::thread::sleep(frame_interval);
        }
    }

    Ok(())
}

fn write_packets_to_mp4(
    output_path: &PathBuf,
    video_packets: &[&EncodedPacket],
    _audio_packets: &[&EncodedPacket],
    width: u32,
    height: u32,
    fps: f64,
    _fps_u32: u32,
) -> Result<()> {
    #[cfg(windows)]
    {
        use crate::encoding::muxer::Mp4Muxer;
        let mut muxer = Mp4Muxer::new(output_path, width, height, fps)?;
        for pkt in video_packets {
            // The ring stores JPEG-compressed frames to keep RAM bounded.
            // Decode one frame at a time for Media Foundation's BGRA input.
            let rgb = image::load_from_memory(&pkt.data)?.to_rgb8();
            let mut bgra = Vec::with_capacity(rgb.as_raw().len() / 3 * 4);
            for pixel in rgb.pixels() {
                bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
            }
            muxer.write_video_packet(&bgra, pkt.pts_ms, pkt.duration_ms, pkt.is_keyframe)?;
        }
        muxer.finalize()?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        // On non-Windows: write a placeholder file for compilation testing
        std::fs::write(output_path, b"PLACEHOLDER_MP4")?;
        Ok(())
    }
}

fn parse_resolution_width(res: &str) -> u32 {
    res.split('x')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1920)
}

fn parse_resolution_height(res: &str) -> u32 {
    res.split('x')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1080)
}
