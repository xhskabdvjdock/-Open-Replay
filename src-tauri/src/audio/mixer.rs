/// Audio mixer: combines system audio and microphone into a single stream.
pub struct AudioMixer {
    pub system_volume: f32,
    pub mic_volume: f32,
    pub mic_muted: bool,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self { system_volume: 1.0, mic_volume: 1.0, mic_muted: false }
    }

    /// Mix system audio (s) and microphone (m) samples into output.
    /// Handles different counts gracefully.
    pub fn mix(&self, system: &[f32], mic: &[f32], output: &mut Vec<f32>) {
        let len = system.len().max(mic.len());
        output.clear();
        output.reserve(len);
        for i in 0..len {
            let s = system.get(i).copied().unwrap_or(0.0) * self.system_volume;
            let m = if self.mic_muted {
                0.0
            } else {
                mic.get(i).copied().unwrap_or(0.0) * self.mic_volume
            };
            // Clamp to prevent clipping
            output.push((s + m).clamp(-1.0, 1.0));
        }
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
