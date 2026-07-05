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
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("./wal"),
            segment_max_bytes: 64 * 1024 * 1024,    // 64 MiB
            group_commit_max_bytes: 4 * 1024 * 1024, // 4 MiB
            group_commit_interval: Duration::from_micros(500),
            group_commit_channel_capacity: 8_192,
            max_payload_bytes: 64 * 1024 * 1024,
            fsync: true,
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
}
