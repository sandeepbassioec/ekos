use std::path::PathBuf;
use std::time::Duration;

/// All tunables for a WAL instance.
#[derive(Debug, Clone)]
pub struct WalConfig {
    /// Directory where segment files are stored.
    pub dir: PathBuf,

    /// Maximum segment file size before rotation (default: 64 MiB).
    pub segment_max_bytes: u64,

    /// GroupCommit: flush to disk after this many pending bytes.
    pub group_commit_max_bytes: usize,

    /// GroupCommit: flush to disk after this interval even if batch is not full.
    pub group_commit_interval: Duration,

    /// GroupCommit: channel capacity (number of pending writes before backpressure).
    pub group_commit_channel_capacity: usize,

    /// Maximum payload size per frame (default: 64 MiB).
    pub max_payload_bytes: usize,

    /// Whether to call fsync after each group commit (disable only in tests).
    pub fsync: bool,

    /// Adaptive batch window: minimum flush interval in nanoseconds.
    ///
    /// When both `adaptive_window_min_ns` and `adaptive_window_max_ns` are
    /// non-zero, the batcher adjusts its flush interval after every batch
    /// based on the EMA flush latency.  Set both to 0 (the default) to
    /// disable adaptive tuning and use the static `group_commit_interval`.
    pub adaptive_window_min_ns: u64,

    /// Adaptive batch window: maximum flush interval in nanoseconds.
    pub adaptive_window_max_ns: u64,

    /// Fingerprint (`vectorvolt_crypto::KeyStore::key_id`) of the key mutation payloads
    /// are encrypted with, or `None` for a plaintext WAL. Written into every
    /// new segment header so replay can refuse a wrong or missing key before
    /// it reads a frame (PP-52). Readers ignore it.
    pub encryption_key_id: Option<crate::segment::KeyId>,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("./wal"),
            segment_max_bytes: 64 * 1024 * 1024,
            group_commit_max_bytes: 4 * 1024 * 1024,
            group_commit_interval: Duration::from_micros(500),
            group_commit_channel_capacity: 8_192,
            max_payload_bytes: 64 * 1024 * 1024,
            fsync: true,
            adaptive_window_min_ns: 0,
            adaptive_window_max_ns: 0,
            encryption_key_id: None,
        }
    }
}

impl WalConfig {
    pub fn with_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir = dir.into();
        self
    }

    pub fn with_fsync(mut self, fsync: bool) -> Self {
        self.fsync = fsync;
        self
    }

    pub fn with_segment_max_bytes(mut self, bytes: u64) -> Self {
        self.segment_max_bytes = bytes;
        self
    }

    pub fn with_group_commit_interval(mut self, d: Duration) -> Self {
        self.group_commit_interval = d;
        self
    }

    /// Enable adaptive batch-window tuning.
    ///
    /// After every flush the batcher adjusts its next sleep duration to
    /// `clamp(ema_latency_us × 1.2 × 1_000, min_ns, max_ns)`.  Set both
    /// to the same value for a fixed override (bypasses the static
    /// `group_commit_interval`).
    pub fn with_adaptive_window(mut self, min_ns: u64, max_ns: u64) -> Self {
        self.adaptive_window_min_ns = min_ns;
        self.adaptive_window_max_ns = max_ns;
        self
    }

    /// Mark new segments as encrypted under the key with this fingerprint.
    pub fn with_encryption_key_id(mut self, key_id: Option<crate::segment::KeyId>) -> Self {
        self.encryption_key_id = key_id;
        self
    }
}
