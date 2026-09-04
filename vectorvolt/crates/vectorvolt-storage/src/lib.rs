//! `vectorvolt-storage` — micro-partition file layer for VectorVolt.
//!
//! ## Core types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`ColumnValues`] | In-memory column data: `Raw(Vec<u64>)` or `DictEncoded` |
//! | [`MicroPartition`] | A table slice: file_id + partition_id + columns |
//! | [`write_partition`] | Serialize a partition to a `.vvsst` binary file |
//! | [`PartitionReader`] | mmap-backed reader: verify CRC, parse metadata, read columns |
//!
//! ## File format
//!
//! See [`format`] for the complete binary layout.  All integers are little-endian.
//! Two CRC32 checksums protect the header and the entire file.

pub mod bitpack;
pub mod direct_io;
pub mod column;
pub mod compaction;
pub mod format;
pub mod lifecycle;
pub mod manifest;
pub mod partition;
pub mod prefetch;
pub mod reader;
pub mod tier;
pub mod tiered_writer;
pub mod writer;

pub use column::ColumnValues;
pub use compaction::{
    CompactionOptions, CompactionOutcome, KeepAll, Liveness, RowLocation, execute_job,
};
pub use lifecycle::{
    CompactionJob, CompactionReason, CompactionStrategy, PartitionMeta, RetentionAction,
    RetentionPlan, RetentionPolicy, RetentionRule, RetentionVeto, VerifyFinding, VerifyReport,
    verify_partitions,
};
pub use manifest::{Manifest, ManifestEdit, ManifestRecovery};
pub use partition::MicroPartition;
pub use prefetch::{AccessPattern, PrefetchAdvisor};
pub use reader::PartitionReader;
pub use tier::{StorageTier, TierConfig, TierStats, TieredStorageConfig};
pub use tiered_writer::TieredWriter;
pub use writer::{Compression, WriteOptions, write_partition, write_partition_with};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid magic bytes — not a .vvsst file")]
    InvalidMagic,

    #[error("unsupported version {0}")]
    UnsupportedVersion(u32),

    #[error("header CRC mismatch (expected {expected:#010x}, stored {stored:#010x})")]
    HeaderCrcMismatch { expected: u32, stored: u32 },

    #[error("file CRC mismatch (expected {expected:#010x}, stored {stored:#010x})")]
    FileCrcMismatch { expected: u32, stored: u32 },

    #[error("column index {0} out of range (partition has {1} columns)")]
    ColumnOutOfRange(usize, u32),

    /// A row offset pointed past the end of the partition.
    ///
    /// Reachable from a stale [`crate::PartitionReader`] coordinate — for example a
    /// `RowDescriptor` cached from before a compaction rewrote the partition. Must
    /// be an error rather than a panic: a database that aborts on a stale index
    /// entry is worse than one that reports it.
    #[error("row_offset {row_offset} out of bounds (partition has {row_count} rows)")]
    RowOffsetOutOfBounds { row_offset: usize, row_count: u32 },

    /// A dictionary index read from the file pointed outside the dictionary.
    ///
    /// Only reachable from corrupt on-disk data, since the index is file content
    /// rather than a caller argument — which is exactly why it must be validated
    /// instead of trusted.
    #[error(
        "dictionary index {index} out of bounds (dictionary holds {size} entries) — \
         partition is corrupt"
    )]
    DictIndexOutOfBounds { index: usize, size: usize },

    /// A computed byte offset fell outside the mapped file.
    #[error("byte range {start}..{end} outside mapped file of {len} bytes — partition is corrupt")]
    OffsetOutsideFile {
        start: usize,
        end: usize,
        len: usize,
    },

    #[error("unknown encoding byte {0:#04x}")]
    UnknownEncoding(u8),

    #[error("row count mismatch: partition has {expected} rows, new column has {got}")]
    RowCountMismatch { expected: usize, got: usize },

    #[error("partition is empty — at least one column required")]
    EmptyPartition,

    #[error("invalid data in file: {0}")]
    InvalidData(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
