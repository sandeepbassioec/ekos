use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupt frame at segment {segment} offset {offset}: {reason}")]
    CorruptFrame { segment: u64, offset: u64, reason: &'static str },

    #[error("Hash chain broken at sequence {sequence}: expected {expected}, got {actual}")]
    ChainBroken { sequence: u64, expected: String, actual: String },

    #[error("CRC32 mismatch at sequence {sequence}")]
    Crc32Mismatch { sequence: u64 },

    #[error("Invalid segment magic at {path}")]
    InvalidSegmentMagic { path: String },

    #[error("Unsupported WAL version {version} (max supported: {max})")]
    UnsupportedVersion { version: u32, max: u32 },

    #[error("WAL is closed")]
    Closed,

    #[error("Sequence {0} not found in any segment")]
    SequenceNotFound(u64),

    #[error("Segment {0} not found")]
    SegmentNotFound(u64),

    #[error("Payload too large: {size} bytes (max {max})")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("GroupCommit channel closed")]
    ChannelClosed,
}

pub type WalResult<T> = Result<T, WalError>;
