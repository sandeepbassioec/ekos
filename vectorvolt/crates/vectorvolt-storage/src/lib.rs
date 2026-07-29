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

pub mod column;
pub mod format;
pub mod partition;
pub mod reader;
pub mod writer;

pub use column::ColumnValues;
pub use partition::MicroPartition;
pub use reader::PartitionReader;
pub use writer::write_partition;

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
