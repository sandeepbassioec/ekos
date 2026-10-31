use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The WAL no longer matches its keyed tail seal (PP-38): it was
    /// truncated, rewritten, or the seal was forged or made with another key.
    #[error("WAL tail seal check failed: {reason}")]
    SealMismatch { reason: String },

    #[error("Corrupt frame at segment {segment} offset {offset}: {reason}")]
    CorruptFrame {
        segment: u64,
        offset: u64,
        reason: &'static str,
    },

    /// Frame `sequence` does not link to the frame before it.
    ///
    /// `expected` is the preceding frame's `frame_hash` — what this frame's
    /// `prev_hash` should have contained. `actual` is what it does contain.
    ///
    /// **Forensic note:** the break is reported at the frame that *fails to link*,
    /// which means the tampering is at or before `sequence - 1`. A frame can be
    /// rewritten with a valid self-hash and CRC (both computable from the frame
    /// alone), but doing so changes its `frame_hash`, so the *next* frame is where
    /// the chain visibly breaks. That is precisely what makes the log
    /// tamper-evident rather than merely checksummed.
    #[error(
        "Hash chain broken at sequence {sequence}: prev_hash is {actual}, expected {expected} \
         (tampering is at or before sequence {})", sequence.saturating_sub(1)
    )]
    ChainBroken {
        sequence: u64,
        expected: String,
        actual: String,
    },

    /// A frame's stored `frame_hash` does not match a hash recomputed from its
    /// own contents.
    ///
    /// Normally unreachable from [`crate::WalReader::verify`], because
    /// [`crate::Frame::decode`] performs the same check and rejects the frame
    /// first. Retained as defence in depth: if `decode` is ever changed to skip
    /// hash validation for speed, `verify` must not silently become a no-op.
    #[error(
        "Frame self-hash mismatch at sequence {sequence}: stored {actual}, computed {expected}"
    )]
    FrameHashMismatch {
        sequence: u64,
        expected: String,
        actual: String,
    },

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

    #[error("GroupCommit channel full — backpressure")]
    Backpressure,

    #[error("snapshot decode error: {0}")]
    SnapshotDecodeError(String),
}

pub type WalResult<T> = Result<T, WalError>;
