/// WASAPI audio capture for both system loopback and microphone input.
use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

pub struct AudioCapture {
    pub is_running: Arc<AtomicBool>,
}

impl AudioCapture {
    pub fn start_loopback(_device_id: &str) -> Result<Self> {
        info!("Starting WASAPI loopback capture");
        let running = Arc::new(AtomicBool::new(true));
        // WASAPI loopback implementation would go here
        // Uses IAudioClient in AUDCLNT_SHAREMODE_SHARED with loopback flag
        Ok(Self { is_running: running })
    }

    pub fn start_microphone(_device_id: &str) -> Result<Self> {
        info!("Starting WASAPI microphone capture");
        let running = Arc::new(AtomicBool::new(true));
        // WASAPI input implementation would go here
        Ok(Self { is_running: running })
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        info!("Audio capture stopped");
    }

    /// Calculate RMS level from PCM samples (for VU meter)
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }
}
