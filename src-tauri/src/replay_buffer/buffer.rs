use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::Mutex;

/// Hard ceiling for in-memory replay data. The duration setting may request
/// more data, but the replay buffer must never grow without a memory bound.
const MAX_BUFFER_BYTES: usize = 384 * 1024 * 1024;

/// A single compressed video frame or audio frame.
#[derive(Debug, Clone)]
pub struct EncodedPacket {
    /// Compressed payload bytes.
    /// Shared so extracting a save snapshot does not duplicate frame payloads.
    pub data: Arc<[u8]>,
    /// Presentation timestamp in milliseconds
    pub pts_ms: i64,
    /// Duration of this packet in milliseconds
    pub duration_ms: i64,
    /// True if this is a keyframe (IDR for H.264)
    pub is_keyframe: bool,
    /// Which stream this packet belongs to
    pub stream: StreamType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    Video,
    Audio,
}

/// Thread-safe circular replay buffer.
/// Stores compressed packets for the last `max_duration_ms` milliseconds,
/// subject to a hard memory ceiling.
pub struct ReplayBuffer {
    inner: Arc<Mutex<ReplayBufferInner>>,
}

struct ReplayBufferInner {
    packets: VecDeque<EncodedPacket>,
    max_duration_ms: i64,
    total_bytes: usize,
}

impl ReplayBuffer {
    pub fn new(max_duration_ms: i64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReplayBufferInner {
                packets: VecDeque::new(),
                max_duration_ms,
                total_bytes: 0,
            })),
        }
    }

    /// Push a new encoded packet into the buffer.
    /// Old packets older than max_duration_ms are evicted.
    pub fn push(&self, packet: EncodedPacket) {
        let mut inner = self.inner.lock();
        if packet.data.len() > MAX_BUFFER_BYTES {
            return;
        }
        inner.total_bytes += packet.data.len();
        inner.packets.push_back(packet);
        inner.evict_old();
    }

    /// Extract packets for the last `duration_ms` milliseconds.
    /// Ensures the first returned packet is a keyframe.
    /// Returns a snapshot of the packets (cloned).
    pub fn extract_last(&self, duration_ms: i64) -> Vec<EncodedPacket> {
        let inner = self.inner.lock();
        if inner.packets.is_empty() {
            return Vec::new();
        }

        let latest_pts = inner.packets.back().map(|p| p.pts_ms).unwrap_or(0);
        let cutoff = latest_pts - duration_ms;

        // Find start index: first video keyframe at or after cutoff
        let start_idx = inner
            .packets
            .iter()
            .position(|p| p.stream == StreamType::Video && p.is_keyframe && p.pts_ms >= cutoff)
            .unwrap_or(0);

        inner.packets.range(start_idx..).cloned().collect()
    }

    /// How full the buffer is, as a fraction [0.0, 1.0]
    pub fn fill_fraction(&self, target_duration_ms: i64) -> f32 {
        let inner = self.inner.lock();
        if inner.packets.is_empty() {
            return 0.0;
        }
        let first = inner.packets.front().map(|p| p.pts_ms).unwrap_or(0);
        let last = inner.packets.back().map(|p| p.pts_ms).unwrap_or(0);
        let current_span = (last - first) as f32;
        (current_span / target_duration_ms as f32).clamp(0.0, 1.0)
    }

    /// Duration currently held in the buffer (ms)
    pub fn held_duration_ms(&self) -> i64 {
        let inner = self.inner.lock();
        if inner.packets.len() < 2 {
            return 0;
        }
        let first = inner.packets.front().map(|p| p.pts_ms).unwrap_or(0);
        let last = inner.packets.back().map(|p| p.pts_ms).unwrap_or(0);
        last - first
    }

    /// Update max duration (e.g. when user changes settings)
    pub fn set_max_duration_ms(&self, ms: i64) {
        let mut inner = self.inner.lock();
        inner.max_duration_ms = ms;
        inner.evict_old();
    }

    pub fn packet_count(&self) -> usize {
        self.inner.lock().packets.len()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.packets.clear();
        inner.total_bytes = 0;
    }
}

impl ReplayBufferInner {
    fn evict_old(&mut self) {
        if self.packets.is_empty() {
            return;
        }
        let latest_pts = self.packets.back().map(|p| p.pts_ms).unwrap_or(0);
        let cutoff = latest_pts - self.max_duration_ms;

        // Captured JPEG frames are independently decodable, so eviction can
        // follow timestamps and the memory ceiling without keyframe concerns.
        while self.packets.len() > 1
            && (self.total_bytes > MAX_BUFFER_BYTES
                || self.packets.front().is_some_and(|packet| packet.pts_ms < cutoff))
        {
            if let Some(packet) = self.packets.pop_front() {
                self.total_bytes = self.total_bytes.saturating_sub(packet.data.len());
            }
        }
    }
}

/// Buffer statistics returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferStats {
    pub packet_count: usize,
    pub held_duration_ms: i64,
    pub fill_fraction: f32,
}
